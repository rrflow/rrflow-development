"""Operator proof of a verified local archive; no provider or model invocation."""
import argparse, hashlib, io, json, os, subprocess, sys, time, zipfile
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--archive', type=Path, required=True)
parser.add_argument('--sha256', required=True)
parser.add_argument('--repository', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
sys.path.insert(0, str(args.repository / 'scripts'))
from verify_controller_trial import verify_archive
manifest = verify_archive(args.archive, args.sha256)
raw = args.archive.read_bytes()
assert hashlib.sha256(raw).hexdigest() == args.sha256
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
bundle = out / 'bundle'
bundle.mkdir()
with zipfile.ZipFile(io.BytesIO(raw)) as archive:
    archive.extractall(bundle)
binary = bundle / 'rrflow.exe'
assert binary.stat().st_nlink == 1
assert hashlib.sha256(binary.read_bytes()).hexdigest() == manifest['files']['rrflow.exe']['sha256']
project = out / 'project'
project.mkdir()
env = {name: os.environ[name] for name in ('SystemRoot', 'WINDIR', 'TEMP', 'TMP', 'USERPROFILE', 'LOCALAPPDATA', 'APPDATA', 'SYSTEMDRIVE', 'COMSPEC') if name in os.environ}
env['PATH'] = str(Path(os.environ['SystemRoot']) / 'System32')

def save(name, value):
    (out / name).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8', newline='\n')

def command(name, *parts):
    argv = [str(binary), '--json', *map(str, parts)]
    run = subprocess.run(argv, cwd=bundle, env=env, capture_output=True, timeout=60)
    record = {'command': argv, 'exit_code': run.returncode, 'stdout': run.stdout.decode('utf-8', errors='replace'), 'stderr': run.stderr.decode('utf-8', errors='replace')}
    save(name + '-command.json', record)
    assert run.returncode == 0, (name, run.returncode)
    value = json.loads(run.stdout)
    save(name + '.json', value)
    return value

version = command('version', 'version')
receipt = json.loads((bundle / 'build-receipt.json').read_text(encoding='utf-8'))
assert version == receipt['report']
identity = command('identity', 'install', 'identity')['installation_identity']
plan = command('installation-plan', 'install', 'plan', '--project', project, '--mode', 'existing', '--identity', identity)
command('installation-result', 'install', 'apply', '--project', project, '--mode', 'existing', '--plan', out / 'installation-plan.json', '--expect', plan['installation']['plan_sha256'])
ready_results = []
verified_results = []
stopped = []
for phase in ('first', 'reopened'):
    argv = [str(binary), '--json', 'serve', '--project', str(project), '--bind', '127.0.0.1:0']
    polls = []
    with (out / (phase + '-server.log')).open('xb') as log:
        process = subprocess.Popen(argv, cwd=bundle, env=env, stdout=log, stderr=log, creationflags=subprocess.CREATE_NO_WINDOW)
        save(phase + '-owned-process.json', {'pid': process.pid, 'command': argv, 'binary_sha256': manifest['files']['rrflow.exe']['sha256']})
        try:
            deadline = time.monotonic() + 60
            while time.monotonic() < deadline:
                assert process.poll() is None, 'owned server exited before ready'
                run = subprocess.run([str(binary), '--json', 'ready', '--project', str(project)], cwd=bundle, env=env, capture_output=True, timeout=10)
                polls.append({'exit_code': run.returncode, 'stdout': run.stdout.decode('utf-8', errors='replace'), 'stderr': run.stderr.decode('utf-8', errors='replace')})
                save(phase + '-readiness-polls.json', polls)
                if run.returncode == 0:
                    ready = json.loads(run.stdout)
                    save(phase + '-ready.json', ready)
                    ready_results.append(ready)
                    break
                time.sleep(0.25)
            else:
                raise RuntimeError('ready deadline exceeded')
        finally:
            if process.poll() is None:
                process.terminate()
            code = process.wait(timeout=30)
            stop = {'pid': process.pid, 'exit_code': code, 'method': 'terminate owned child; not graceful-shutdown proof'}
            save(phase + '-owned-process-stopped.json', stop)
            stopped.append(stop)
    verified = command(phase + '-verify', 'verify', '--project', project, '--level', 'quick')
    assert verified['status'] == 'passed', verified['status']
    verified_results.append(verified)
for field in ('instance_id', 'plan_sha256', 'project_inventory_sha256', 'runtime_manifest_sha256'):
    assert verified_results[0][field] == verified_results[1][field], field
result = {'archive_sha256': args.sha256, 'binary_sha256': manifest['files']['rrflow.exe']['sha256'], 'binary_source_commit': manifest['binary_source_commit'], 'version_matches_receipt': True, 'path_contains_only_system32': True, 'provider_credentials_supplied': False, 'ready_phases': len(ready_results), 'owned_servers_stopped': len(stopped), 'quick_verify_phases': len(verified_results), 'reopen_identity_matches': True, 'clean_machine_qualified': False, 'model_or_worker_execution': False}
save('result.json', result)
print(json.dumps(result))
