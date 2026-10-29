#!/usr/bin/env python3
"""Archive rejection tests; these fixtures are not model or qualification data."""
import hashlib
import json
from pathlib import Path
import stat
import tempfile
import unittest
import zipfile

import verify_controller_trial as verifier


class ArchiveRejectionTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.archive = Path(self.directory.name) / 'trial.zip'
        self.license = (Path(__file__).resolve().parents[1] / 'LICENSE').read_bytes()

    def manifest(self):
        record = {'sha256': hashlib.sha256(self.license).hexdigest(), 'bytes': len(self.license)}
        return {'format_version': 1, 'kind': 'experimental-controller-trial',
                'qualification': 'unqualified',
                'files': {name: dict(record) for name in verifier.REQUIRED_FILES}}

    def write_archive(self, manifest=None, omit=None, extra=None, altered=None):
        manifest = manifest or self.manifest()
        with zipfile.ZipFile(self.archive, 'w') as archive:
            archive.writestr(verifier.MANIFEST, json.dumps(manifest))
            for name in manifest['files']:
                if name != omit:
                    archive.writestr(name, b'changed' if name == altered else self.license)
            if extra:
                archive.writestr(extra, self.license)
        return verifier.digest_file(self.archive)

    def test_unsafe_paths(self):
        for name in ('../escape', '/root', 'a//b', './a', 'a/../b', 'C:/x',
                     'a\\b', 'nul.txt', 'a/COM1', 'a.', 'a ', 'a\x00b', ''):
            with self.subTest(name=name), self.assertRaises(ValueError):
                verifier.safe_name(name)

    def test_wrong_external_digest(self):
        self.write_archive()
        with self.assertRaisesRegex(ValueError, 'supplied expectation'):
            verifier.verify_archive(self.archive, '0' * 64)

    def test_missing_member(self):
        digest = self.write_archive(omit='rrflow.exe')
        with self.assertRaisesRegex(ValueError, 'missing or unexpected'):
            verifier.verify_archive(self.archive, digest)

    def test_extra_member(self):
        digest = self.write_archive(extra='private.env')
        with self.assertRaisesRegex(ValueError, 'missing or unexpected'):
            verifier.verify_archive(self.archive, digest)

    def test_case_collision(self):
        digest = self.write_archive(extra='RRFLOW.EXE')
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            verifier.verify_archive(self.archive, digest)

    def test_changed_member(self):
        digest = self.write_archive(altered='rrflow.exe')
        with self.assertRaisesRegex(ValueError, 'member size differs'):
            verifier.verify_archive(self.archive, digest)

    def test_wrong_member_digest(self):
        manifest = self.manifest()
        manifest['files']['rrflow.exe']['sha256'] = '0' * 64
        digest = self.write_archive(manifest)
        with self.assertRaisesRegex(ValueError, 'member digest differs'):
            verifier.verify_archive(self.archive, digest)

    def test_symlink_member(self):
        self.write_archive(omit='rrflow.exe')
        with zipfile.ZipFile(self.archive, 'a') as archive:
            info = zipfile.ZipInfo('rrflow.exe')
            info.external_attr = (stat.S_IFLNK | 0o777) << 16
            archive.writestr(info, 'elsewhere')
        with self.assertRaisesRegex(ValueError, 'special'):
            verifier.verify_archive(self.archive, verifier.digest_file(self.archive))

    def test_missing_declared_requirement(self):
        manifest = self.manifest()
        del manifest['files']['rrflow.exe']
        with self.assertRaisesRegex(ValueError, 'required artifact set'):
            verifier.parse_manifest(json.dumps(manifest).encode())

    def test_malformed_and_oversized_records(self):
        for record in (None, [], {'bytes': True, 'sha256': '0' * 64},
                       {'bytes': verifier.MAX_BYTES + 1, 'sha256': '0' * 64}):
            manifest = self.manifest()
            manifest['files']['rrflow.exe'] = record
            with self.subTest(record=record), self.assertRaises(ValueError):
                verifier.parse_manifest(json.dumps(manifest).encode())

    def test_duplicate_json_keys(self):
        with self.assertRaisesRegex(ValueError, 'duplicate JSON'):
            verifier.parse_manifest(b'{"files":{},"files":{}}')

    def test_qualified_claim_rejected(self):
        manifest = self.manifest()
        manifest['qualification'] = 'release'
        with self.assertRaisesRegex(ValueError, 'unsupported'):
            verifier.parse_manifest(json.dumps(manifest).encode())


if __name__ == '__main__':
    unittest.main()
