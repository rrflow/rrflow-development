"""Actual installed-process proposal proof, never a routing-quality evaluation.

All model/provider/task inputs are supplied files. No generated labels or model
responses. Starts and terminates only owned child processes; retains no secrets.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
import tomllib
from urllib.error import HTTPError
from urllib.request import Request, urlopen
from uuid import uuid4


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    inputs = ['build-receipt', 'contract-example', 'profile', 'task-source',
              'secret-file', 'secret-name', 'environment-name']
    for name in inputs:
        parser.add_argument('--' + name)
    parser.add_argument('--output', required=True)
    parser.add_argument('--recover-proof', help='Read an existing attempt and budget; never dispatch or replay.')
    args = parser.parse_args()
    recovery = Path(args.recover_proof).resolve() if args.recover_proof else None
    if recovery is None and any(getattr(args, name.replace('-', '_')) is None for name in inputs):
        parser.error('A new proof requires all build, profile, task and credential inputs.')
    if recovery is not None and any(getattr(args, name.replace('-', '_')) is not None for name in inputs):
        parser.error('Recovery accepts only --recover-proof and --output.')
    out = Path(args.output).resolve()
    out.mkdir(parents=True, exist_ok=False)

    def save(name, value):
        with (out / name).open('x', encoding='utf-8') as stream:
            json.dump(value, stream, indent=2, ensure_ascii=False)
            stream.write('\n')

    def cli(*parts):
        r = subprocess.run([str(binary), '--json', *map(str, parts)],
                           capture_output=True, text=True, encoding='utf-8', timeout=120)
        if r.returncode:
            raise RuntimeError('CLI failed: ' + str(parts[0]) + ': ' + r.stderr[:1000])
        return json.loads(r.stdout)

    receipt_path = recovery / 'build-receipt.json' if recovery else Path(args.build_receipt)
    receipt = json.loads(receipt_path.read_text(encoding='utf-8'))
    build = receipt['report']['build']
    assert build['provenance']['dirty'] is False
    if recovery:
        binary = recovery / Path(receipt['binary']).name
        assert hashlib.sha256(binary.read_bytes()).hexdigest() == build['executable_sha256']
        assert cli('version')['build'] == build
        project = recovery / 'project'
        save('recovery-source.json', {'proof': str(recovery), 'binarySha256': build['executable_sha256'],
             'sourceCommit': build['provenance']['source_commit'], 'providerDispatchAllowed': False})
    else:
        source_binary = Path(receipt['binary'])
        assert hashlib.sha256(source_binary.read_bytes()).hexdigest() == build['executable_sha256']
        binary = out / source_binary.name
        shutil.copyfile(source_binary, binary)
        assert binary.stat().st_nlink == 1
        save('build-receipt.json', receipt)
        assert cli('version')['build'] == build
        profile = json.loads(Path(args.profile).read_text(encoding='utf-8'))
        task_source = json.loads(Path(args.task_source).read_text(encoding='utf-8'))
        save('profile-input.json', profile)
        save('task-source.json', task_source)
        policy = profile['selection_policy']
        binding = next(b for b in profile['bindings'] if b['id'] == policy['controller_binding_id'])
        mapping = {'format_version': 1, 'entries': [{
            'reference': binding['credential'], 'environment_variable': args.environment_name}]}
        save('credential-mapping.json', mapping)
        secret = next(line.split('=', 1)[1] for line in Path(args.secret_file).read_text(encoding='utf-8').splitlines()
                      if line.startswith(args.secret_name + '='))
        project = out / 'project'
        project.mkdir()
        identity = cli('install', 'identity')['installation_identity']
        plan = cli('install', 'plan', '--project', project, '--mode', 'existing', '--identity', identity)
        save('installation-plan.json', plan)
        save('installation-result.json', cli('install', 'apply', '--project', project, '--mode', 'existing',
             '--plan', out / 'installation-plan.json', '--expect', plan['installation']['plan_sha256']))
    locator = tomllib.loads((project / '.rrflow/config.toml').read_text(encoding='utf-8'))
    credential = json.loads((project / locator['operator_credential']).read_text(encoding='utf-8'))
    instance = locator['identity']['instance_id']
    request_payload = json.loads((recovery / 'proposal-request.json').read_text(encoding='utf-8')) if recovery else None
    accepted = None
    attempt_before_restart = None
    budget_before_restart = None

    for phase in (['recovered'] if recovery else ['missing-credential', 'authorized', 'reopened']):
        log_path = out / (phase + '-server.log')
        env = dict(os.environ)
        command = [str(binary), '--json', 'serve', '--project', str(project), '--bind', '127.0.0.1:0']
        if phase in ['authorized', 'reopened']:
            env[args.environment_name] = secret
            command += ['--endpoint-credentials', str(out / 'credential-mapping.json')]
        with log_path.open('xb') as log:
            process = subprocess.Popen(command, stdout=log, stderr=log, env=env,
                                       creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
            save(phase + '-owned-process.json', {'pid': process.pid, 'binary': str(binary),
                 'project': str(project), 'binarySha256': build['executable_sha256']})
            try:
                deadline = time.monotonic() + 60
                base = None
                catalogue_path = None
                while base is None:
                    assert process.poll() is None, 'Owned server exited; inspect its log.'
                    for line in log_path.read_text(encoding='utf-8', errors='replace').splitlines():
                        try:
                            announcement = json.loads(line)
                            if announcement.get('status') == 'listening':
                                base = announcement['url']
                                catalogue_path = announcement['endpoint_catalogue_path']
                        except json.JSONDecodeError:
                            pass
                    assert time.monotonic() < deadline, 'Server startup timed out'
                    if base is None:
                        time.sleep(0.2)

                with urlopen(base + catalogue_path, timeout=15) as response:
                    catalogue = json.load(response)
                save(phase + '-catalogue.json', catalogue)
                endpoints = {entry['operation']: entry for entry in catalogue['outcome']['payload']['endpoints']}

                def envelope(payload, mutation):
                    operation = 'controller-proof-' + uuid4().hex
                    return {'protocol': 'rrd', 'protocol_version': 1,
                            'context': {'request_id': operation, 'operation_id': operation,
                                        'idempotency_key': operation if mutation else None, 'deadline_unix_ms': None},
                            'resource': {'segments': [{'kind': 'instance', 'id': instance}]}, 'payload': payload}

                def http(operation, payload, headers=None):
                    descriptor = endpoints[operation]
                    body = envelope(payload, descriptor['mutation'])
                    req = Request(base + descriptor['path'], method=descriptor['method'].upper(),
                                  data=json.dumps(body).encode('utf-8'),
                                  headers={'Content-Type': 'application/json', **(headers or {})})
                    try:
                        with urlopen(req, timeout=90) as response:
                            return response.status, json.load(response), body
                    except HTTPError as error:
                        return error.code, json.load(error), body

                status, response, _ = http('session-create', {'limits': {
                    'idle_timeout_ms': 120000, 'absolute_timeout_ms': 300000, 'max_open_transactions': 1}},
                    {'Authorization': 'ApiKey ' + credential['credential'], 'X-RRD-Principal': credential['principal_id']})
                assert status == 200, 'Session admission failed'
                session = response['outcome']['payload']
                headers = {'Authorization': 'Bearer ' + session['token'], 'X-RRD-Session': session['session_id']}

                def call(name, operation, payload, expected=200, authenticate=True):
                    descriptor = endpoints[operation]
                    save(phase + '-' + name + '-intent.json', {'operation': operation, 'path': descriptor['path'], 'payload': payload,
                         'mutation': descriptor['mutation'], 'responseObserved': False})
                    status, response, request = http(operation, payload, headers if authenticate else {})
                    save(phase + '-' + name + '.json', {'status': status, 'request': request, 'response': response})
                    assert status == expected, f'{name} returned {status}, expected {expected}; inspect retained response'
                    return response['outcome'].get('payload')

                if phase == 'recovered':
                    selection = {key: request_payload[key] for key in ['scope', 'task_id', 'attempt_id']}
                    # No proposal/replay operation and no provider credentials in this phase.
                    assert endpoints['task-attempt-read']['mutation'] is False
                    assert endpoints['task-budget-read']['mutation'] is False
                    attempt = call('attempt', 'task-attempt-read', selection)
                    budget = call('budget', 'task-budget-read', {key: selection[key] for key in ['scope', 'task_id']})
                elif phase == 'missing-credential':
                    preview = call('preview', 'endpoint-configuration-preview', {'expected_revision': 0, 'profile': profile})
                    configuration = call('apply', 'endpoint-configuration-apply',
                                         {'plan': preview, 'approved_plan_sha256': preview['plan_sha256']})
                    # Exact bounded prefix of the actual PR body, with truncation
                    # explicitly recorded; no invented task or correctness label.
                    goal = task_source['body'].encode('utf-8')[:4096].decode('utf-8', errors='ignore')
                    task = call('open-task', 'task-open', {'scope': 'instance:' + instance,
                                'title': task_source['title'], 'goal': goal})['task']
                    save('task-context-projection.json', {'sourceUrl': task_source['url'],
                         'originalBodySha256': hashlib.sha256(task_source['body'].encode('utf-8')).hexdigest(),
                         'projectedBodyBytes': len(goal.encode('utf-8')), 'truncated': goal != task_source['body'],
                         'use': 'transport/admission proof only; no training or quality label'})
                    save('task-record.json', task)
                    save('configuration.json', configuration)
                    result = subprocess.run([args.contract_example, 'proposal-request', str(out / 'task-record.json'),
                         str(out / 'configuration.json'), 'actual-proposal-' + uuid4().hex], capture_output=True,
                         text=True, encoding='utf-8', timeout=30, check=True)
                    request_payload = json.loads(result.stdout)
                    save('proposal-request.json', request_payload)
                    call('unauthenticated', 'task-endpoint-propose', request_payload, 401, False)
                    call('missing-credential', 'task-endpoint-propose', request_payload, 412)
                    budget = call('budget', 'task-budget-read', {'scope': task['scope'], 'task_id': task['task_id']})
                    assert budget['budget'] is None
                else:
                    assert request_payload is not None
                    selection = {key: request_payload[key] for key in ['scope', 'task_id', 'attempt_id']}
                    if phase == 'authorized':
                        accepted = call('proposal', 'task-endpoint-propose', request_payload)
                    else:
                        call('replay', 'task-endpoint-propose', request_payload, 412)
                    attempt = call('attempt', 'task-attempt-read', selection)
                    budget = call('budget', 'task-budget-read', {key: selection[key] for key in ['scope', 'task_id']})
                    assert attempt['attempt']['status'] == 'accepted'
                    assert attempt['attempt']['endpoint_proposal'] == accepted
                    assert budget['budget']['reserved_steps'] == 1
                    assert budget['budget']['active_attempt'] is None
                    assert budget['budget']['execution_may_continue'] is False
                    if phase == 'authorized':
                        attempt_before_restart, budget_before_restart = attempt, budget
                    else:
                        assert attempt == attempt_before_restart
                        assert budget == budget_before_restart
            finally:
                if process.poll() is None:
                    process.terminate()
                    process.wait(timeout=30)
                save(phase + '-owned-process-stopped.json', {'pid': process.pid, 'exitCode': process.returncode})
    if recovery:
        save('result.json', {'sourceCommit': build['provenance']['source_commit'],
             'binarySha256': build['executable_sha256'], 'proof': str(recovery),
             'runnerSha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
             'attempt': attempt, 'budget': budget, 'providerCallRequested': False,
             'ownedServersStopped': True, 'modelQualityQualified': False, 'trainingDataCreated': False})
        print(json.dumps({'output': str(out), 'status': 'existing attempt and budget read; no provider call'}))
        return
    save('result.json', {'sourceCommit': build['provenance']['source_commit'],
         'binarySha256': build['executable_sha256'], 'acceptedProposal': accepted,
         'runnerSha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
         'restartReadbackEqual': True, 'sameAttemptReplayRejected': True, 'ownedServersStopped': True,
         'engineDispatchObserved': True, 'modelQualityQualified': False, 'trainingDataCreated': False,
         'scope': 'actual experimental task-text controller transport/admission; no worker execution or source-edit plan proof'})
    print(json.dumps({'output': str(out), 'status': 'actual installed proposal/reopen checks passed'}))


if __name__ == '__main__':
    main()
