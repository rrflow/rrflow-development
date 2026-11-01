#!/usr/bin/env python3
"""Assemble an unqualified local Windows testing archive from identified inputs."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import zipfile

import build_rrflow
from verify_controller_trial import MANIFEST, MAX_BYTES, digest_file, verify_archive

ROOT = Path(__file__).resolve().parents[1]
TRACKED_INPUTS = {
    'LICENSE': 'LICENSE',
    'README.txt': 'scripts/inference/controller_trial_README.txt',
    'verify_controller_trial.py': 'scripts/verify_controller_trial.py',
    'tools/installed_controller_proof.py': 'scripts/inference/installed_controller_proof.py',
    'tools/installed_controller_fault_relay.py': 'scripts/inference/installed_controller_fault_relay.py',
}


def encoded(value):
    return (json.dumps(value, sort_keys=True, indent=2) + '\n').encode('utf-8')


def command(args, **kwargs):
    return subprocess.run(args, cwd=ROOT, check=True, text=True, encoding='utf-8', **kwargs)


def assemble(args):
    if os.name != 'nt':
        raise ValueError('this experimental assembler qualifies Windows inputs only')
    output = args.output.resolve()
    if output.exists() or output.suffix != '.zip':
        raise ValueError('choose an absent .zip output')
    checksum = output.with_suffix('.sha256')
    if checksum.exists():
        raise ValueError('checksum output already exists')
    source = build_rrflow.source_snapshot(ROOT, False)
    receipt_path = args.build_receipt.resolve(strict=True)
    receipt = json.loads(receipt_path.read_text(encoding='utf-8'))
    build = receipt['report']['build']
    provenance = build['provenance']
    if provenance['dirty'] or provenance['target'] != 'x86_64-pc-windows-msvc':
        raise ValueError('requires a clean-source native Windows x64 binary')
    if provenance['cargo_profile'] != 'release':
        raise ValueError('testing archive requires the explicit release build profile')
    binary = Path(receipt['binary'])
    if not binary.is_absolute():
        binary = receipt_path.parent / binary
    if binary.is_symlink() or not binary.is_file() or binary.stat().st_size > MAX_BYTES:
        raise ValueError('binary must be a bounded regular file')
    if digest_file(binary) != build['executable_sha256']:
        raise ValueError('binary differs from build receipt')
    report = json.loads(command([str(binary), '--json', 'version'], capture_output=True).stdout)
    if report != receipt['report']:
        raise ValueError('actual executable identity differs from receipt')
    # Assembly tooling can advance independently; every runtime/contract input
    # must still match the actual binary source rather than relabeling that binary.
    command(['git', 'diff', '--exit-code', provenance['source_commit'], '--',
             'crates', 'sdks', 'Cargo.toml', 'Cargo.lock', '.cargo',
             'rust-toolchain', 'rust-toolchain.toml'], capture_output=True)
    env = dict(os.environ)
    env.pop('RRFLOW_BUILD_MANIFEST', None)
    if build_rrflow.build_environment(env) != provenance['build_environment_sha256']:
        raise ValueError('helper build environment differs from primary provenance')
    if command(['rustc', '--version'], capture_output=True, env=env).stdout.strip() != provenance['rustc']:
        raise ValueError('helper compiler differs from primary provenance')
    compiled = command(['cargo', 'build', '--locked', '-p', 'rrd-contract', '--example',
                        'endpoint_selection', '--profile', 'release', '--target-dir',
                        str(args.target_dir.resolve()), '--message-format', 'json'],
                       stdout=subprocess.PIPE, env=env)
    helpers = []
    for line in compiled.stdout.splitlines():
        event = json.loads(line)
        if (event.get('reason') == 'compiler-artifact'
                and event.get('target', {}).get('name') == 'endpoint_selection'
                and event.get('executable')):
            helpers.append(Path(event['executable']))
    if len(helpers) != 1 or helpers[0].is_symlink() or not helpers[0].is_file():
        raise ValueError('helper compilation did not identify exactly one artifact')
    schema = command([str(helpers[0]), 'schema'], capture_output=True).stdout.encode('utf-8')
    if len(schema) > 128 * 1024:
        raise ValueError('schema exceeds bound')
    json.loads(schema)
    files = {'rrflow.exe': binary, 'tools/endpoint_selection.exe': helpers[0]}
    for target, relative in TRACKED_INPUTS.items():
        command(['git', 'ls-files', '--error-unmatch', relative], capture_output=True)
        path = ROOT / relative
        if path.is_symlink() or not path.is_file():
            raise ValueError('assembly input must be a tracked regular file')
        files[target] = path
    # Portable locator only; embedded report/provenance is preserved exactly.
    portable_receipt = {**receipt, 'binary': 'rrflow.exe'}
    payloads = {'build-receipt.json': encoded(portable_receipt),
                'contracts/endpoint-selection.schema.json': schema}
    manifest = {
        'format_version': 1, 'kind': 'experimental-controller-trial',
        'qualification': 'unqualified', 'platform': provenance['target'],
        'product_version': report['product_version'],
        'binary_source_commit': provenance['source_commit'],
        'assembly_source_commit': source['source_commit'],
        'assembly_source_content_sha256': source['source_content_sha256'],
        'original_build_receipt_sha256': digest_file(receipt_path),
        'runtime_source_compatible': True, 'model_weights_included': False,
        'credentials_included': False, 'files': {},
    }
    for name, path in files.items():
        manifest['files'][name] = {'sha256': digest_file(path), 'bytes': path.stat().st_size}
    for name, data in payloads.items():
        manifest['files'][name] = {'sha256': hashlib.sha256(data).hexdigest(), 'bytes': len(data)}
    if sum(record['bytes'] for record in manifest['files'].values()) > MAX_BYTES:
        raise ValueError('testing payload exceeds bound')
    build_rrflow.require_same(source, build_rrflow.source_snapshot(ROOT, False), 'assembly source')
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='controller-trial-', dir=output.parent) as staging:
        staged = Path(staging) / output.name
        with zipfile.ZipFile(staged, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=6) as archive:
            for name in sorted(set(files) | set(payloads) | {MANIFEST}):
                info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                info.external_attr = 0o100644 << 16
                data = encoded(manifest) if name == MANIFEST else payloads.get(name)
                with archive.open(info, 'w', force_zip64=True) as target:
                    if data is not None:
                        target.write(data)
                    else:
                        with files[name].open('rb') as source_file:
                            while chunk := source_file.read(1024 * 1024):
                                target.write(chunk)
        expected = digest_file(staged)
        verify_archive(staged, expected)
        build_rrflow.require_same(source, build_rrflow.source_snapshot(ROOT, False), 'assembly source')
        # Exclusive publication; never replace a previous archive or checksum.
        with output.open('xb') as target, staged.open('rb') as source_file:
            while chunk := source_file.read(1024 * 1024):
                target.write(chunk)
        if digest_file(output) != expected:
            raise ValueError('published archive differs from verified bytes')
        with checksum.open('x', encoding='utf-8', newline='\n') as stream:
            stream.write(expected + '  ' + output.name + '\n')
    return {'archive': str(output), 'sha256': expected, 'files': len(manifest['files']),
            'qualification': 'unqualified', 'binarySource': provenance['source_commit']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--build-receipt', type=Path, required=True)
    parser.add_argument('--target-dir', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(assemble(args)))


if __name__ == '__main__':
    main()
