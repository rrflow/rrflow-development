"""Local HTTP correctness probes only; bytes here are not model or training data."""
from concurrent.futures import ThreadPoolExecutor
from copy import deepcopy
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
from installed_task_resume_proof import assert_resumed_budget, validate_resume_profiles


class FaultRelayTests(unittest.TestCase):
    def exercise(self, hold_call, drop, followup=False):
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
                                   10, 1024, 1024, hold_call + int(followup), drop, **options)

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
                    for extra in range(int(followup)):
                        self.assertEqual(invoke(b'separate capacity transport probe'), payload)
                        observed = json.loads((output / f'relay-{hold_call + extra + 1}-observation.json').read_text())
                        self.assertFalse(observed['heldBeforeEngineResponse'])
                        self.assertEqual(observed['responseSha256'], hashlib.sha256(payload).hexdigest())
                    self.assertEqual(relay.calls, hold_call + int(followup))
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

    def test_second_call_holds_then_delivers_exact_worker_bytes(self):
        self.exercise(2, False)

    def test_third_call_passes_after_the_second_call_barrier(self):
        self.exercise(2, False, followup=True)

    def test_both_resumed_calls_preserve_bytes_after_the_second_call_barrier(self):
        self.exercise(2, False, followup=2)

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

    def test_worker_fault_modes_parse_but_require_actual_inputs(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / 'must-not-exist'
            for fault in ('configuration', 'session', 'stop', 'disconnect'):
                with self.subTest(fault=fault):
                    result = subprocess.run([sys.executable, str(Path(__file__).with_name('installed_controller_proof.py')),
                                             '--worker', '--fault-case', fault, '--output', str(output)],
                                            capture_output=True, timeout=10)
                    self.assertEqual(result.returncode, 2)
                    self.assertIn(b'A new proof requires all build, profile, task and credential inputs.', result.stderr)
                    self.assertFalse(output.exists())

    def test_worker_fault_modes_reject_incompatible_inputs_before_startup(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / 'must-not-exist'
            for extra in (['--recover-proof', 'unused'], ['--admission-only'],
                          ['--native-client', 'unused', '--native-client-timeout-ms', '1000']):
                with self.subTest(extra=extra):
                    result = subprocess.run([sys.executable, str(Path(__file__).with_name('installed_controller_proof.py')),
                                             '--worker', '--fault-case', 'configuration', '--output', str(output), *extra],
                                            capture_output=True, timeout=10)
                    self.assertEqual(result.returncode, 2)
                    self.assertNotIn(b'A new proof requires all build, profile, task and credential inputs.', result.stderr)
                    self.assertFalse(output.exists())

    def test_terminal_rejection_requires_distinct_mode_and_capacity_input(self):
        combinations = [
            ['--worker-terminal-rejection', '--capacity-task-source', 'unused'],
            ['--worker', '--worker-terminal-rejection'],
            ['--capacity-task-source', 'unused'],
        ]
        base = ['--worker', '--worker-terminal-rejection', '--capacity-task-source', 'unused']
        combinations += [base + extra for extra in (
            ['--worker-interrupt'], ['--fault-case', 'stop'], ['--recover-proof', 'unused'],
            ['--admission-only'], ['--native-client', 'unused', '--native-client-timeout-ms', '1000'])]
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / 'must-not-exist'
            for arguments in combinations:
                with self.subTest(arguments=arguments):
                    result = subprocess.run([sys.executable, str(Path(__file__).with_name('installed_controller_proof.py')),
                                             '--output', str(output), *arguments], capture_output=True, timeout=10)
                    self.assertEqual(result.returncode, 2)
                    self.assertNotIn(b'A new proof requires all build, profile, task and credential inputs.', result.stderr)
                    self.assertFalse(output.exists())

    def test_terminal_rejection_parses_but_requires_real_inputs(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / 'must-not-exist'
            result = subprocess.run([sys.executable, str(Path(__file__).with_name('installed_controller_proof.py')),
                                     '--worker', '--worker-terminal-rejection', '--capacity-task-source', 'unused',
                                     '--output', str(output)], capture_output=True, timeout=10)
            self.assertEqual(result.returncode, 2)
            self.assertIn(b'A new proof requires all build, profile, task and credential inputs.', result.stderr)
            self.assertFalse(output.exists())

    def test_terminal_resume_requires_its_own_mode_and_profile_before_startup(self):
        base = ['--worker', '--worker-terminal-resume', '--resume-profile', 'unused']
        invalid = [['--worker-terminal-resume', '--resume-profile', 'unused'],
                   ['--worker', '--worker-terminal-resume'], ['--resume-profile', 'unused']]
        invalid += [base + extra for extra in (
            ['--worker-terminal-rejection', '--capacity-task-source', 'unused'],
            ['--capacity-task-source', 'unused'], ['--worker-interrupt'], ['--fault-case', 'stop'],
            ['--recover-proof', 'unused'], ['--admission-only'],
            ['--native-client', 'unused', '--native-client-timeout-ms', '1000'])]
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / 'must-not-exist'
            for arguments in invalid + [base]:
                with self.subTest(arguments=arguments):
                    result = subprocess.run([sys.executable, str(Path(__file__).with_name('installed_controller_proof.py')),
                                             '--output', str(output), *arguments], capture_output=True, timeout=10)
                    self.assertEqual(result.returncode, 2)
                    missing_inputs = b'A new proof requires all build, profile, task and credential inputs.'
                    if arguments == base:
                        self.assertIn(missing_inputs, result.stderr)
                    else:
                        self.assertNotIn(missing_inputs, result.stderr)
                    self.assertFalse(output.exists())

    def test_resume_profile_admits_only_a_larger_worker_output_allowance(self):
        # In-memory configuration-shape probes only; no model/provider data.
        original = {'selection_policy': {'controller_binding_id': 'transport-probe'},
                    'worker_policy': {'bindings': [{'binding_id': 'transport-probe',
                                      'generation': {'maximum_output_tokens': 1}}]}}
        restored = deepcopy(original)
        restored['worker_policy']['bindings'][0]['generation']['maximum_output_tokens'] = 2
        validate_resume_profiles(original, restored)
        for field in ['name', 'bindings', 'selection_policy']:
            changed = deepcopy(restored)
            changed[field] = None
            with self.subTest(field=field), self.assertRaises((AssertionError, TypeError)):
                validate_resume_profiles(original, changed)
        for value in [True, 1, 0, -1, '2']:
            changed = deepcopy(restored)
            changed['worker_policy']['bindings'][0]['generation']['maximum_output_tokens'] = value
            with self.subTest(value=value), self.assertRaises(AssertionError):
                validate_resume_profiles(original, changed)

    def test_resume_oracle_rejects_budget_resets_using_retained_control_bytes(self):
        root = Path(__file__).resolve().parents[2]
        capture = root / 'docs/evidence/change-journals/gate-g/task-worker-terminal-live-receipts/worker-authorized-terminal-worker-budget.json'
        before = json.loads(capture.read_text(encoding='utf-8'))['response']['outcome']['payload']
        # Explicit in-memory perturbation, not a new observed engine response.
        after = deepcopy(before)
        after['budget']['stopped'] = None
        after['budget']['stop_expected_sha256'] = None
        after['budget_sha256'] = 'different oracle probe, not a receipt'
        assert_resumed_budget(before, after)
        for field in ['reserved_steps', 'charged_elapsed_ms', 'deadline_unix_ms', 'started_at_unix_ms']:
            changed = deepcopy(after)
            changed['budget'][field] += 1
            with self.subTest(field=field), self.assertRaises(AssertionError):
                assert_resumed_budget(before, changed)


if __name__ == '__main__':
    unittest.main()
