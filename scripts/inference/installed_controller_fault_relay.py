"""Bounded test relay of actual provider bytes; never generates model output."""
import atexit
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import threading
from urllib.error import HTTPError
from urllib.request import Request, build_opener, ProxyHandler, HTTPRedirectHandler


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


class FaultRelay:
    def __init__(self, upstream, output, timeout_seconds, request_limit, response_limit, maximum_calls, drop):
        self.ready = threading.Event()
        self.release = threading.Event()
        self.error = None
        self.calls = 0
        self._closed = False
        lock = threading.Lock()
        output = Path(output)
        relay = self

        def save(name, value):
            with (output / name).open('x', encoding='utf-8') as stream:
                json.dump(value, stream, indent=2)
                stream.write('\n')

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_args):
                pass  # Never log request headers, credentials or prompts.

            def do_POST(self):
                with lock:
                    relay.calls += 1
                    ordinal = relay.calls
                try:
                    if ordinal > maximum_calls or self.path != '/v1/chat/completions':
                        raise RuntimeError('relay request exceeds explicit trial scope')
                    self.connection.settimeout(timeout_seconds)
                    length = int(self.headers.get('Content-Length', '0'))
                    if not 0 < length <= request_limit:
                        raise RuntimeError('relay request length outside configured bounds')
                    body = self.rfile.read(length)
                    if len(body) != length:
                        raise RuntimeError('incomplete engine request')
                    save(f'relay-{ordinal}-intent.json', {
                        'requestSha256': hashlib.sha256(body).hexdigest(),
                        'requestBytes': len(body), 'upstream': upstream,
                        'actualProviderRequest': True, 'responseObserved': False})
                    headers = {'Content-Type': 'application/json'}
                    if self.headers.get('Authorization'):
                        headers['Authorization'] = self.headers['Authorization']
                    request = Request(upstream.rstrip('/') + '/chat/completions',
                                      data=body, headers=headers, method='POST')
                    opener = build_opener(ProxyHandler({}), NoRedirect())
                    try:
                        response = opener.open(request, timeout=timeout_seconds)
                    except HTTPError as error:
                        response = error
                    with response:
                        status = response.code
                        data = response.read(response_limit + 1)
                    if len(data) > response_limit:
                        raise RuntimeError('provider response exceeds configured bound')
                    with (output / f'relay-{ordinal}-provider-response.json').open('xb') as stream:
                        stream.write(data)
                    save(f'relay-{ordinal}-observation.json', {
                        'httpStatus': status, 'requestSha256': hashlib.sha256(body).hexdigest(),
                        'responseSha256': hashlib.sha256(data).hexdigest(),
                        'responseBytes': len(data), 'heldBeforeEngineResponse': ordinal == 1,
                        'generatedOrReplayedResponse': False})
                    if status != 200:
                        raise RuntimeError('provider did not succeed; fault condition not established')
                    if ordinal == 1:
                        relay.ready.set()
                        if not relay.release.wait(timeout_seconds):
                            raise TimeoutError('fault action did not release actual response')
                        if drop:
                            save('relay-dropped-delivery.json', {'actualProviderResponseRetained': True})
                            self.close_connection = True
                            return
                    self.send_response(status)
                    self.send_header('Content-Type', 'application/json')
                    self.send_header('Content-Length', str(len(data)))
                    self.end_headers()
                    self.wfile.write(data)
                except Exception as error:
                    relay.error = type(error).__name__
                    save(f'relay-{ordinal}-failure.json', {'errorType': relay.error})
                    relay.ready.set()
                    self.close_connection = True

        self.server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        self.server.daemon_threads = True
        self.base_url = f'http://127.0.0.1:{self.server.server_address[1]}/v1'
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        atexit.register(self.close)

    def close(self):
        if not self._closed:
            self._closed = True
            self.release.set()
            self.server.shutdown()
            self.server.server_close()
            self.thread.join(timeout=5)
