import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

import capture_observations as capture


class ObservationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.repo = self.root / 'repo'
        self.repo.mkdir()
        self.context = self.root / 'task.json'
        self.context.write_text(json.dumps({'schemaVersion': 1, 'issueTitle': 'Fixture task'}))
        self.output = self.root / 'observations.jsonl'
        self.git('init', '-q')
        self.git('config', 'user.name', 'Capture fixture')
        self.git('config', 'user.email', 'fixture@example.invalid')
        self.git('config', 'commit.gpgsign', 'false')
        (self.repo / 'code.py').write_text('original\n')
        self.git('add', 'code.py')
        self.git('commit', '-qm', 'Create fixture')

    def git(self, *args):
        return subprocess.run(['git', '-C', str(self.repo), *args], capture_output=True, check=True)

    def observe(self):
        return capture.capture_once(self.repo, self.context, self.output)

    def rows(self):
        return [capture.validate_record(json.loads(row)) for row in self.output.read_text().splitlines()]

    def command(self, *args):
        return [sys.executable, str(Path(capture.__file__)), '--repo', str(self.repo),
                '--context', str(self.context), '--output', str(self.output), *args]

    def test_same_path_edits_staging_and_return_to_prior_content(self):
        self.assertTrue(self.observe())
        original = self.output.read_bytes()
        self.assertFalse(self.observe())
        (self.repo / 'code.py').write_text('first edit\n')
        self.assertTrue(self.observe())
        (self.repo / 'code.py').write_text('second edit\n')
        self.assertTrue(self.observe())
        self.git('add', 'code.py')
        self.assertTrue(self.observe())
        self.git('reset', '-q', 'HEAD', '--', 'code.py')
        (self.repo / 'code.py').write_text('original\n')
        self.assertTrue(self.observe())
        rows = self.rows()
        self.assertEqual(len(rows), 5)
        self.assertTrue(self.output.read_bytes().startswith(original))
        self.assertEqual(rows[0]['snapshotSha256'], rows[-1]['snapshotSha256'])
        self.assertNotEqual(rows[0]['recordId'], rows[-1]['recordId'])
        self.assertEqual(rows[2]['previousRecordId'], rows[1]['recordId'])
        self.assertEqual(rows[1]['changedPaths'], rows[2]['changedPaths'])

    def test_restart_override_and_collector_identity(self):
        self.observe()
        result = subprocess.run(self.command('--once'), capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)['state'], 'unchanged')
        self.context.write_text(json.dumps({'schemaVersion': 1, 'issueTitle': 'Updated fixture',
            'userOverride': True, 'route': {'provider': 'fixture', 'modelId': 'fixture-model',
            'effort': 'low', 'accessMode': 'local', 'bindingId': 'fixture-binding'}}))
        self.assertTrue(self.observe())
        row = self.rows()[-1]
        self.assertTrue(row['task']['userOverride'])
        self.assertIsNone(row['task']['outcome'])
        self.assertIsNone(row['engineReceipt'])
        self.assertFalse(row['trainingAdmitted'])
        self.assertEqual(row['labelStatus'], 'unreviewed')
        with patch.object(capture, 'SOURCES', {'changedCollector': '0' * 64}):
            self.assertTrue(self.observe())
        self.assertEqual(len(self.rows()), 3)

    def test_invalid_and_changing_input_recover(self):
        self.observe()
        retained = self.output.read_bytes()
        self.context.write_text('{"schemaVersion":1,')
        with self.assertRaises(ValueError):
            self.observe()
        self.assertEqual(self.output.read_bytes(), retained)
        self.context.write_text(json.dumps({'schemaVersion': 1, 'issueTitle': 'Recovered'}))
        first = capture.observation(self.repo, self.context)
        second = (first[0], {**first[1], 'contextSha256': '0' * 64}, first[2])
        with patch.object(capture, 'observation', side_effect=[first, second]):
            with self.assertRaisesRegex(ValueError, 'inputs moved'):
                self.observe()
        self.assertTrue(self.observe())
        self.assertEqual(len(self.rows()), 2)

    def test_metadata_only_redaction_untracked_scope_and_rename(self):
        secret = 'sk-' + 'a' * 30
        self.context.write_text(json.dumps({'schemaVersion': 1, 'issueTitle': 'Remove ' + secret}))
        (self.repo / 'code.py').write_text(secret)
        self.observe()
        self.assertNotIn(secret, self.output.read_text())
        (self.repo / 'untracked.txt').write_text('outside declared scope')
        self.assertFalse(self.observe())
        self.assertEqual(self.rows()[0]['source']['scope'], 'tracked_files_only')
        self.git('add', 'code.py')
        self.git('commit', '-qm', 'Changed fixture')
        self.git('mv', 'code.py', 'renamed file.py')
        self.assertTrue(self.observe())
        self.assertEqual(set(self.rows()[-1]['changedPaths']), {'code.py', 'renamed file.py'})

    def test_busy_lock_and_partial_or_modified_log_preserved(self):
        lock = self.output.with_suffix('.jsonl.lock')
        lock.write_text('another writer')
        with self.assertRaises(FileExistsError):
            self.observe()
        self.assertEqual(lock.read_text(), 'another writer')
        lock.unlink()
        self.output.write_bytes(b'{"partial":')
        with self.assertRaisesRegex(ValueError, 'incomplete'):
            self.observe()
        self.assertEqual(self.output.read_bytes(), b'{"partial":')
        self.assertFalse(lock.exists())
        self.output.unlink()
        self.observe()
        row = self.rows()[0]
        row['task']['issueTitle'] = 'tampered'
        self.output.write_bytes(capture.canonical(row))
        retained = self.output.read_bytes()
        with self.assertRaisesRegex(ValueError, 'identity changed'):
            self.observe()
        self.assertEqual(retained, self.output.read_bytes())

    def test_reordered_chain_is_rejected_without_repair(self):
        self.observe()
        (self.repo / 'code.py').write_text('changed\n')
        self.observe()
        reordered = b''.join(reversed(self.output.read_bytes().splitlines(keepends=True)))
        self.output.write_bytes(reordered)
        with self.assertRaisesRegex(ValueError, 'chain changed'):
            self.observe()
        self.assertEqual(self.output.read_bytes(), reordered)

    def test_bounds_and_forged_authority_fail_before_publication(self):
        with self.assertRaises(ValueError):
            capture.git(self.repo, 'show', 'HEAD:code.py', limit=2)
        self.context.write_bytes(b'x' * (capture.MAX_CONTEXT + 1))
        with self.assertRaises(ValueError):
            self.observe()
        for text in ('{"schemaVersion":1,"issueTitle":"x","trainingAdmitted":true}',
                     '{"schemaVersion":1,"issueTitle":"a","issueTitle":"b"}',
                     '{"schemaVersion":true,"issueTitle":"x"}'):
            self.context.write_text(text)
            with self.assertRaises(ValueError):
                self.observe()
        self.assertFalse(self.output.exists())
        with self.assertRaises(ValueError):
            capture.capture_once(self.repo, self.context, self.repo / 'feedback.jsonl')
        result = subprocess.run(self.command('--duration-seconds', 'nan'), capture_output=True)
        self.assertNotEqual(result.returncode, 0)

    def test_running_process_observes_later_edit_and_stops_at_record_limit(self):
        process = subprocess.Popen(self.command('--interval-seconds', '.25', '--duration-seconds', '15',
                                   '--max-records', '2'), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            deadline = time.monotonic() + 10
            while not self.output.exists() or not self.output.read_bytes().endswith(b'\n'):
                self.assertIsNone(process.poll())
                self.assertLess(time.monotonic(), deadline)
                time.sleep(.05)
            (self.repo / 'code.py').write_text('edited while polling\n')
            stdout, stderr = process.communicate(timeout=15)
            self.assertEqual(process.returncode, 0, stderr)
            self.assertEqual(len(self.rows()), 2)
            self.assertEqual(json.loads(stdout.splitlines()[-1]),
                             {'state': 'stopped', 'recordsWritten': 2, 'trainingAdmitted': False})
        finally:
            if process.poll() is None:
                process.kill()
                process.communicate(timeout=5)


if __name__ == '__main__':
    unittest.main()
