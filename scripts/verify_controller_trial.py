#!/usr/bin/env python3
"""Verify a local experimental archive; hashes are not release signatures."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import stat
import zipfile

MAX_FILES = 32
MAX_BYTES = 2 * 1024 * 1024 * 1024
MAX_MANIFEST_BYTES = 128 * 1024
MANIFEST = 'trial-manifest.json'
REQUIRED_FILES = {
    'rrflow.exe', 'build-receipt.json', 'LICENSE', 'README.txt',
    'verify_controller_trial.py', 'contracts/endpoint-selection.schema.json',
    'tools/endpoint_selection.exe', 'tools/installed_controller_proof.py',
    'tools/installed_controller_fault_relay.py',
}


def digest_file(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def safe_name(name):
    path = PurePosixPath(name)
    reserved = {'con', 'prn', 'aux', 'nul', *(f'com{i}' for i in range(1, 10)),
                *(f'lpt{i}' for i in range(1, 10))}
    if (not name or not path.parts or len(name) > 240 or '\\' in name or ':' in name
            or path.is_absolute() or path.as_posix() != name
            or any(part in {'.', '..'} or part.rstrip(' .') != part
                   or part.split('.')[0].lower() in reserved
                   or any(ord(char) < 32 for char in part) for part in path.parts)):
        raise ValueError('unsafe or noncanonical archive member')
    return name


def parse_manifest(raw):
    if len(raw) > MAX_MANIFEST_BYTES:
        raise ValueError('manifest exceeds bound')
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError('duplicate JSON key')
            result[key] = value
        return result
    value = json.loads(raw, object_pairs_hook=unique)
    if (not isinstance(value, dict) or value.get('format_version') != 1 or value.get('kind') != 'experimental-controller-trial'
            or value.get('qualification') != 'unqualified'
            or not isinstance(value.get('files'), dict) or not 1 <= len(value['files']) < MAX_FILES):
        raise ValueError('unsupported trial manifest')
    names = set()
    total = 0
    for name, record in value['files'].items():
        safe_name(name)
        if name == MANIFEST or name.casefold() in names:
            raise ValueError('duplicate or reserved manifest member')
        names.add(name.casefold())
        if (not isinstance(record, dict) or set(record) != {'sha256', 'bytes'} or type(record['bytes']) is not int
                or not 0 <= record['bytes'] <= MAX_BYTES
                or not isinstance(record['sha256'], str)
                or re.fullmatch('[a-f0-9]{64}', record['sha256']) is None):
            raise ValueError('invalid file digest/size record')
        total += record['bytes']
    if total > MAX_BYTES:
        raise ValueError('archive payload exceeds bound')
    if set(value['files']) != REQUIRED_FILES:
        raise ValueError('trial manifest does not declare the required artifact set')
    return value


def verify_archive(path, expected_sha256):
    path = Path(path)
    if path.is_symlink() or not path.is_file() or path.stat().st_size > MAX_BYTES:
        raise ValueError('archive must be a bounded regular file')
    if digest_file(path) != expected_sha256:
        raise ValueError('archive digest differs from supplied expectation')
    with zipfile.ZipFile(path) as archive:
        members = archive.infolist()
        if not 1 < len(members) <= MAX_FILES:
            raise ValueError('archive member count outside bound')
        names = set()
        for info in members:
            safe_name(info.filename)
            if (info.is_dir() or info.filename.casefold() in names or info.flag_bits & 1
                    or stat.S_IFMT(info.external_attr >> 16) not in (0, stat.S_IFREG)
                    or not 0 <= info.file_size <= MAX_BYTES):
                raise ValueError('duplicate, special, encrypted or oversized member')
            names.add(info.filename.casefold())
        if MANIFEST not in archive.namelist() or archive.getinfo(MANIFEST).file_size > MAX_MANIFEST_BYTES:
            raise ValueError('bounded manifest is missing')
        manifest = parse_manifest(archive.read(MANIFEST))
        if set(archive.namelist()) != set(manifest['files']) | {MANIFEST}:
            raise ValueError('archive has missing or unexpected files')
        for name, record in manifest['files'].items():
            if archive.getinfo(name).file_size != record['bytes']:
                raise ValueError('member size differs from manifest: ' + name)
            with archive.open(name) as stream:
                if hashlib.file_digest(stream, 'sha256').hexdigest() != record['sha256']:
                    raise ValueError('member digest differs from manifest: ' + name)
        if archive.getinfo('build-receipt.json').file_size > MAX_MANIFEST_BYTES:
            raise ValueError('build receipt exceeds bound')
        receipt = json.loads(archive.read('build-receipt.json'))
        build = receipt['report']['build']
        if (receipt['binary'] != 'rrflow.exe'
                or build['executable_sha256'] != manifest['files']['rrflow.exe']['sha256']
                or build['provenance']['source_commit'] != manifest['binary_source_commit']
                or build['provenance']['target'] != manifest['platform']
                or build['provenance']['dirty']
                or build['provenance']['cargo_profile'] != 'release'):
            raise ValueError('binary receipt differs from the declared trial identity')
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive', type=Path)
    parser.add_argument('--sha256', required=True, help='Expected archive digest supplied separately')
    args = parser.parse_args()
    manifest = verify_archive(args.archive, args.sha256)
    print(json.dumps({'verified': True, 'files': len(manifest['files']),
                      'qualification': manifest['qualification'], 'extractsOrExecutes': False}))


if __name__ == '__main__':
    main()
