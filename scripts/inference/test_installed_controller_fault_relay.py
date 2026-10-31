"""Local HTTP correctness probes only; bytes here are not model or training data."""
from concurrent.futures import ThreadPoolExecutor
from http.client import RemoteDisconnected
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import unittest
from urllib.request import Request, urlopen

from installed_controller_fault_relay import FaultRelay


class FaultRelayTests(unittest.TestCase):
    def exercise(self, hold_call, drop):
        payload = b'opaque local transport fixture\x00\xff'
        requests = []

        class Upstream(BaseHTTPRequestHandler):
            def log_message(self, *_args):
                pass

            def do_POST(self):
                requests.append(self.rfile.read(int(self.headers['Content-Length'])))
                self.send_response(200)
                self.send_header('Content-Length', str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)

        upstream = ThreadingHTTPServer(('127.0.0.1', 0), Upstream)
        thread = threading.Thread(target=upstream.serve_forever, daemon=True)
        thread.start()
        try:
            with tempfile.TemporaryDirectory() as temporary:
                output = Path(temporary)
                options = {} if hold_call == 1 else {'hold_call': hold_call}
                relay = FaultRelay(f'http://127.0.0.1:{upstream.server_port}/v1', output,
                                   10, 1024, 1024, hold_call, drop, **options)

                def invoke(body):
                    request = Request(relay.base_url + '/chat/completions', data=body, method='POST')
                    with urlopen(request, timeout=10) as response:
                        return response.read()

                try:
                    if hold_call == 2:
                        self.assertEqual(invoke(b'first transport probe'), payload)
                        self.assertFalse(relay.ready.is_set())
                    with ThreadPoolExecutor(max_workers=1) as executor:
                        pending = executor.submit(invoke, b'held transport probe')
                        try:
                            self.assertTrue(relay.ready.wait(5))
                            self.assertIsNone(relay.error)
                            self.assertFalse(pending.done(), 'response crossed the unreleased barrier')
                            self.assertEqual(len(requests), hold_call)
                            self.assertEqual((output / f'relay-{hold_call}-provider-response.json').read_bytes(), payload)
                            observed = json.loads((output / f'relay-{hold_call}-observation.json').read_text())
                            self.assertTrue(observed['heldBeforeEngineResponse'])
                            self.assertEqual(observed['responseSha256'], hashlib.sha256(payload).hexdigest())
                            if hold_call == 2:
                                first = json.loads((output / 'relay-1-observation.json').read_text())
                                self.assertFalse(first['heldBeforeEngineResponse'])
                        finally:
                            relay.release.set()
                        if drop:
                            with self.assertRaises(RemoteDisconnected):
                                pending.result(timeout=5)
                            self.assertTrue((output / 'relay-dropped-delivery.json').exists())
                        else:
                            self.assertEqual(pending.result(timeout=5), payload)
                    self.assertEqual(relay.calls, hold_call)
                    self.assertIsNone(relay.error)
                finally:
                    relay.close()
        finally:
            upstream.shutdown()
            upstream.server_close()
            thread.join(timeout=5)

    def test_default_first_call_holds_and_preserves_bytes(self):
        self.exercise(1, False)

    def test_second_call_holds_then_drops_only_its_delivery(self):
        self.exercise(2, True)

    def test_invalid_hold_ordinal_is_rejected_before_startup(self):
        for ordinal in (0, 3):
            with self.subTest(ordinal=ordinal), self.assertRaises(ValueError):
                FaultRelay('unused', Path('unused'), 1, 1, 1, 2, True, hold_call=ordinal)

    def test_incompatible_interrupt_arguments_have_no_filesystem_effect(self):
        combinations = [
            ['--worker-interrupt'],
            ['--worker', '--worker-interrupt', '--native-client', 'unused'],
            ['--worker', '--worker-interrupt', '--recover-proof', 'unused'],
            ['--worker', '--worker-interrupt', '--admission-only'],
            ['--worker', '--worker-interrupt', '--fault-case', 'stop'],
        ]
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / 'must-not-exist'
            for arguments in combinations:
                with self.subTest(arguments=arguments):
                    result = subprocess.run([sys.executable, str(Path(__file__).with_name('installed_controller_proof.py')),
                                             '--output', str(output), *arguments], capture_output=True, timeout=10)
                    self.assertEqual(result.returncode, 2)
                    self.assertFalse(output.exists())


if __name__ == '__main__':
    unittest.main()
