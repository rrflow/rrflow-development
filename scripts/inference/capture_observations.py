"""Bounded, optional local observation capture. No engine or training authority."""
from __future__ import annotations

import argparse
from contextlib import contextmanager
from datetime import datetime, timezone
import math
import os
from pathlib import Path
import re
import subprocess
import sys
import threading
import time

from review_outcomes import canonical, digest, fields, identity, read_bounded, strict_json

MAX_CONTEXT = 64 * 1024
MAX_DIFF = 16 * 1024 * 1024
MAX_RECORD = 128 * 1024
MAX_LOG = 64 * 1024 * 1024
ROUTE_FIELDS = {'provider', 'modelId', 'effort', 'accessMode', 'bindingId'}
SOURCES = {name: digest(Path(__file__).with_name(name).read_bytes())
           for name in ('capture_observations.py', 'review_outcomes.py')}
SECRET_PATTERNS = (
    r'\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}\b',
    r'\bAKIA[0-9A-Z]{16}\b', r'(?i)\bBearer\s+[A-Za-z0-9._\-]{16,}\b',
    r'(?i)\b(api[_\-]?key|token|secret|password)\s*[:=]\s*[\"\x27]?([A-Za-z0-9._\-]{8,})',
    r'\bgh[pousr]_[A-Za-z0-9]{20,}\b', r'\bsk-[A-Za-z0-9_-]{20,}\b',
    r'\b(?:\d{1,3}\.){3}\d{1,3}\b', r'\+?\d[\d\-\s()]{7,}\d',
)


def redact(value: str) -> str:
    for pattern in SECRET_PATTERNS:
        value = re.sub(pattern, '<REDACTED>', value)
    return value


def bounded_text(value: object, limit: int, empty: bool = True) -> str:
    if not isinstance(value, str) or len(value) > limit or (not empty and not value.strip()):
        raise ValueError('require bounded text')
    return value


def task_input(path: Path) -> tuple[dict, str]:
    raw = read_bounded(path, MAX_CONTEXT)
    value = fields(strict_json(raw), {'schemaVersion', 'issueTitle'},
                   {'issueId', 'intentText', 'userOverride', 'route'})
    if type(value['schemaVersion']) is not int or value['schemaVersion'] != 1:
        raise ValueError('unsupported task context version')
    bounded_text(value['issueTitle'], 2000, False)
    bounded_text(value.get('issueId', ''), 200)
    bounded_text(value.get('intentText', ''), 8000)
    if value.get('userOverride') is not None and type(value['userOverride']) is not bool:
        raise ValueError('override is an explicit boolean or unknown')
    route = fields(value.get('route', dict.fromkeys(ROUTE_FIELDS)), ROUTE_FIELDS)
    for item in route.values():
        if item is not None and redact(bounded_text(item, 200, False)) != item:
            raise ValueError('sensitive-pattern route identity cannot be silently changed')
    if route['accessMode'] not in (None, 'local', 'api', 'subscription'):
        raise ValueError('unsupported access mode')
    return value, digest(raw)


def git(repo: Path, *args: str, limit: int = MAX_DIFF) -> bytes:
    """Cap command output and elapsed wait; discard potentially sensitive stderr."""
    process = subprocess.Popen(
        ['git', '--no-optional-locks', '-c', 'core.fsmonitor=false', '-c', 'core.hooksPath=',
         '-C', str(repo), '--no-pager', *args], stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
        creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
    data: list[bytes] = []
    assert process.stdout is not None
    reader = threading.Thread(target=lambda: data.append(process.stdout.read(limit + 1)), daemon=True)
    try:
        reader.start()
        reader.join(20)
        if reader.is_alive() or not data:
            raise ValueError('Git observation timed out')
        if len(data[0]) > limit:
            raise ValueError('Git observation exceeded byte budget')
        if process.wait(timeout=2):
            raise ValueError('Git observation failed; a committed repository is required')
        return data[0]
    finally:
        if process.poll() is None:
            process.kill()
            process.wait(timeout=5)
        reader.join(5)
        process.stdout.close()


def observation(repo: Path, context: Path) -> tuple[dict, dict, list[str]]:
    task, context_sha = task_input(context)
    head = git(repo, 'rev-parse', '--verify', 'HEAD', limit=256).decode().strip()
    if not re.fullmatch(r'[0-9a-f]{40}|[0-9a-f]{64}', head):
        raise ValueError('invalid Git object identity')
    branch = git(repo, 'rev-parse', '--abbrev-ref', 'HEAD', limit=4096).decode().strip()
    status = git(repo, 'status', '--porcelain=v1', '-z', '--untracked-files=no', limit=1024 * 1024)
    staged = git(repo, 'diff', '--cached', '--binary', '--no-ext-diff', '--no-textconv', 'HEAD', '--')
    unstaged = git(repo, 'diff', '--binary', '--no-ext-diff', '--no-textconv', '--')
    paths: list[str] = []
    entries = iter(status.decode('utf-8').split('\0'))
    for entry in entries:
        if entry:
            paths.append(redact(entry[3:]))
            if 'R' in entry[:2] or 'C' in entry[:2]:
                paths.append(redact(next(entries)))
    if len(paths) > 1000:
        raise ValueError('changed-path budget exceeded')
    source = {'scope': 'tracked_files_only', 'head': head, 'branchSha256': digest(branch.encode()),
              'statusSha256': digest(status), 'stagedDiffSha256': digest(staged),
              'unstagedDiffSha256': digest(unstaged), 'contextSha256': context_sha,
              'collectorSources': SOURCES, 'pythonVersion': sys.version.split()[0]}
    return task, source, paths


def validate_record(record: object) -> dict:
    value = fields(record, {'schemaVersion', 'purpose', 'recordId', 'capturedAt', 'streamSha256',
        'snapshotSha256', 'source', 'changedPaths', 'task', 'previousRecordId',
        'labelStatus', 'trainingAdmitted', 'engineReceipt'})
    if (type(value['schemaVersion']) is not int or value['schemaVersion'] != 1
            or value['purpose'] != 'unreviewed_local_observation' or value['labelStatus'] != 'unreviewed'
            or value['trainingAdmitted'] is not False or value['engineReceipt'] is not None):
        raise ValueError('unsupported observation or invented authority')
    identity(value['streamSha256'])
    identity(value['snapshotSha256'])
    if not isinstance(value['source'], dict) or digest(canonical(value['source'])) != value['snapshotSha256']:
        raise ValueError('observation source identity changed')
    if value['previousRecordId'] is not None:
        identity(value['previousRecordId'])
    if digest(canonical({k: v for k, v in value.items() if k != 'recordId'})) != identity(value['recordId']):
        raise ValueError('retained observation identity changed')
    return value


@contextmanager
def writer_lock(path: Path):
    handle = path.open('xb')
    try:
        with handle:
            yield
    finally:
        path.unlink()


def append(path: Path, record: dict) -> bool:
    """Serial append and predecessor binding; no silent tail or stale-lock repair."""
    path.parent.mkdir(parents=True, exist_ok=True)
    with writer_lock(path.with_name(path.name + '.lock')):
        previous = None
        length = path.stat().st_size if path.exists() else 0
        if length > MAX_LOG:
            raise ValueError('capture log exceeds budget; retain it and select a new output')
        if length:
            with path.open('rb') as existing:
                while line := existing.readline(MAX_RECORD + 1):
                    if len(line) > MAX_RECORD or not line.endswith(b'\n'):
                        raise ValueError('oversized or incomplete retained observation; preserve it')
                    prior = validate_record(strict_json(line))
                    if prior['streamSha256'] == record['streamSha256']:
                        if prior['previousRecordId'] != (previous['recordId'] if previous else None):
                            raise ValueError('retained observation chain changed')
                        previous = prior
        if previous and previous['snapshotSha256'] == record['snapshotSha256']:
            return False
        record['previousRecordId'] = previous['recordId'] if previous else None
        record['recordId'] = digest(canonical(record))
        validate_record(record)
        encoded = canonical(record)
        if len(encoded) > MAX_RECORD or length + len(encoded) > MAX_LOG:
            raise ValueError('observation or log exceeds byte budget')
        with path.open('ab') as output:
            output.write(encoded)
            output.flush()
            os.fsync(output.fileno())
        return True


def capture_once(repo: Path, context: Path, output: Path) -> bool:
    repo, context, output = repo.resolve(), context.resolve(), output.resolve()
    if output.is_relative_to(repo):
        raise ValueError('output must be outside the observed repository')
    top = Path(git(repo, 'rev-parse', '--show-toplevel', limit=16384).decode().strip()).resolve()
    if top != repo:
        raise ValueError('select the Git root explicitly')
    first = observation(repo, context)
    second = observation(repo, context)
    if first != second:
        raise ValueError('inputs moved during observation; retry next poll')
    task, source, paths = second
    record = {'schemaVersion': 1, 'purpose': 'unreviewed_local_observation',
              'capturedAt': datetime.now(timezone.utc).isoformat(),
              'streamSha256': digest(canonical([str(repo), str(context), source['scope']])),
              'snapshotSha256': digest(canonical(source)), 'source': source, 'changedPaths': paths,
              'task': {'issueTitle': redact(task['issueTitle']), 'issueId': redact(task.get('issueId', '')),
                       'intentText': redact(task.get('intentText', '')),
                       'route': task.get('route', dict.fromkeys(ROUTE_FIELDS)),
                       'userOverride': task.get('userOverride'), 'outcome': None},
              'previousRecordId': None, 'labelStatus': 'unreviewed',
              'trainingAdmitted': False, 'engineReceipt': None}
    return append(output, record)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--context', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--once', action='store_true')
    parser.add_argument('--interval-seconds', type=float, default=5)
    parser.add_argument('--duration-seconds', type=float, default=300)
    parser.add_argument('--max-records', type=int, default=100)
    args = parser.parse_args()
    if (not math.isfinite(args.interval_seconds) or not .25 <= args.interval_seconds <= 3600
            or not math.isfinite(args.duration_seconds) or not 1 <= args.duration_seconds <= 86400
            or not 1 <= args.max_records <= 10000):
        parser.error('require interval .25..3600 seconds, duration 1..86400 seconds, records 1..10000')
    deadline, written, failed = time.monotonic() + args.duration_seconds, 0, False
    while time.monotonic() < deadline and written < args.max_records:
        try:
            changed = capture_once(args.repo, args.context, args.output)
            written += int(changed)
            failed = False
            if changed or args.once:
                print(canonical({'state': 'captured' if changed else 'unchanged', 'recordsWritten': written}).decode(), end='', flush=True)
        except (OSError, ValueError, subprocess.TimeoutExpired) as error:
            if not failed or args.once:
                print(canonical({'state': 'observation_deferred', 'errorClass': type(error).__name__}).decode(), end='', flush=True)
            failed = True
        if args.once:
            return int(failed)
        time.sleep(max(0, min(args.interval_seconds, deadline - time.monotonic())))
    print(canonical({'state': 'stopped', 'recordsWritten': written, 'trainingAdmitted': False}).decode(), end='', flush=True)
    return int(failed)


if __name__ == '__main__':
    raise SystemExit(main())
