"""Installed recovery assertions over public engine calls and actual provider bytes.

This helper owns no lifecycle state and never generates a provider response.
"""
from copy import deepcopy
import hashlib
import json
import subprocess
from uuid import uuid4


def validate_resume_profiles(original, replacement):
    binding_id = original['selection_policy']['controller_binding_id']
    before = next(w for w in original['worker_policy']['bindings'] if w['binding_id'] == binding_id)
    after = next(w for w in replacement['worker_policy']['bindings'] if w['binding_id'] == binding_id)
    prior = before['generation']['maximum_output_tokens']
    restored = after['generation']['maximum_output_tokens']
    assert isinstance(prior, int) and not isinstance(prior, bool) and prior > 0
    assert isinstance(restored, int) and not isinstance(restored, bool) and restored > prior
    expected = deepcopy(original)
    worker = next(w for w in expected['worker_policy']['bindings'] if w['binding_id'] == binding_id)
    worker['generation']['maximum_output_tokens'] = restored
    assert replacement == expected, 'Resume profiles may differ only in the selected worker output allowance'


def assert_resumed_budget(before, after):
    old = before['budget']
    current = after['budget']
    assert old['stopped'] == 'attempt_failed'
    assert old['active_attempt'] is None and old['execution_may_continue'] is False
    expected = deepcopy(old)
    expected['stopped'] = None
    expected['stop_expected_sha256'] = None
    expected['last_observed_at_unix_ms'] = current['last_observed_at_unix_ms']
    assert current['last_observed_at_unix_ms'] >= old['last_observed_at_unix_ms']
    assert current == expected, 'Resume reset or changed retained task effort/history'
    assert after['budget_sha256'] != before['budget_sha256']


def read_resume_probe(call, expected):
    selector = call('resumed-selector-attempt', 'task-attempt-read',
                    {k: expected['proposalRequest'][k] for k in ['scope', 'task_id', 'attempt_id']})
    worker = call('resumed-worker-attempt', 'task-attempt-read',
                  {k: expected['workerRequest'][k] for k in ['scope', 'task_id', 'attempt_id']})
    budget = call('resumed-budget', 'task-budget-read',
                  {k: expected['workerRequest'][k] for k in ['scope', 'task_id']})
    assert selector == expected['selectorAttempt']
    assert worker == expected['workerAttempt']
    assert budget == expected['budget']
    return {**expected, 'selectorAttempt': selector, 'workerAttempt': worker, 'budget': budget}


def execute_resume_probe(call, save, out, contract_example, relay, profile, configuration,
                         task, original_request, original_selector, original_worker_request,
                         original_worker, stopped_budget, binding_id):
    selection = {k: original_request[k] for k in ['scope', 'task_id']}
    resume = {**selection, 'expected_budget_sha256': stopped_budget['budget_sha256']}
    save('resume-request.json', resume)
    call('resume-unauthenticated', 'task-budget-resume', resume, 401, False)
    assert call('before-resume-budget', 'task-budget-read', selection) == stopped_budget
    resumed = call('resume', 'task-budget-resume', resume)
    assert_resumed_budget(stopped_budget, resumed)
    call('stale-resume', 'task-budget-resume', resume, 409)
    assert call('after-stale-resume-budget', 'task-budget-read', selection) == resumed
    call('original-worker-replay', 'task-worker-execute', original_worker_request, 412,
         error_contains='ReasoningAttemptRecorded')
    assert relay.calls == 2 and relay.error is None, 'Resume/replay must not dispatch'

    plan = call('restore-profile-preview', 'endpoint-configuration-preview',
                {'expected_revision': configuration['revision'], 'profile': profile})
    configuration = call('restore-profile-apply', 'endpoint-configuration-apply',
                         {'plan': plan, 'approved_plan_sha256': plan['plan_sha256']})
    save('resume-configuration.json', configuration)
    assert call('after-profile-budget', 'task-budget-read', selection) == resumed
    generated = subprocess.run([contract_example, 'proposal-request', str(out / 'task-record.json'),
        str(out / 'resume-configuration.json'), 'actual-resumed-proposal-' + uuid4().hex],
        capture_output=True, text=True, encoding='utf-8', timeout=30, check=True)
    request = json.loads(generated.stdout)
    assert {k: request[k] for k in selection} == selection
    assert request['expected_task_sha256'] == original_request['expected_task_sha256']
    assert request['expected_configuration_sha256'] != original_request['expected_configuration_sha256']
    save('resume-proposal-request.json', request)
    proposal = call('new-proposal', 'task-endpoint-propose', request)
    assert proposal['controller_binding_id'] == binding_id
    assert proposal['choice']['kind'] == 'select', 'Actual resumed selector abstained'
    assert proposal['choice']['binding_id'] == binding_id, 'Resume proof requires the relayed binding'
    assert relay.calls == 3 and relay.error is None
    selector = call('new-selector-attempt', 'task-attempt-read',
                    {k: request[k] for k in ['scope', 'task_id', 'attempt_id']})
    assert selector['attempt']['status'] == 'accepted'
    assert selector['attempt']['endpoint_proposal'] == proposal
    selected_budget = call('new-selector-budget', 'task-budget-read', selection)
    assert selected_budget['budget']['reserved_steps'] == stopped_budget['budget']['reserved_steps'] + 1
    assert selected_budget['budget']['deadline_unix_ms'] == stopped_budget['budget']['deadline_unix_ms']
    assert selected_budget['budget']['charged_elapsed_ms'] >= stopped_budget['budget']['charged_elapsed_ms']
    worker_request = {
        **selection, 'attempt_id': 'actual-resumed-worker-' + uuid4().hex,
        'selection_attempt_id': request['attempt_id'],
        'expected_selection_sha256': selector['attempt']['outcome_sha256'],
        'expected_task_sha256': request['expected_task_sha256'],
        'expected_configuration_sha256': request['expected_configuration_sha256'],
        'max_storage_keys': request['max_storage_keys'], 'context': None,
    }
    save('resume-worker-request.json', worker_request)
    output = call('new-worker', 'task-worker-execute', worker_request)
    for field, value in {
        **selection, 'attempt_id': worker_request['attempt_id'],
        'selection_attempt_id': worker_request['selection_attempt_id'],
        'selection_sha256': worker_request['expected_selection_sha256'],
        'task_sha256': worker_request['expected_task_sha256'],
        'configuration_sha256': worker_request['expected_configuration_sha256'],
        'binding_id': binding_id,
    }.items():
        assert output[field] == value, 'Resumed worker identity mismatch: ' + field
    assert hashlib.sha256(output['content'].encode('utf-8')).hexdigest() == output['content_sha256']
    worker = call('new-worker-attempt', 'task-attempt-read',
                  {k: worker_request[k] for k in ['scope', 'task_id', 'attempt_id']})
    assert worker['attempt']['status'] == 'accepted' and worker['attempt']['worker_output'] == output
    budget = call('new-worker-budget', 'task-budget-read', selection)
    current, before = budget['budget'], stopped_budget['budget']
    assert current['reserved_steps'] == before['reserved_steps'] + 2
    assert current['charged_elapsed_ms'] >= selected_budget['budget']['charged_elapsed_ms']
    assert current['active_attempt'] is None and current['execution_may_continue'] is False
    assert current['stopped'] is None
    for key in ['format_version', 'instance_id', 'scope', 'task_id', 'configuration_sha256',
                'limits', 'started_at_unix_ms', 'deadline_unix_ms', 'repetition_policy']:
        assert current.get(key) == before.get(key), 'Changed original budget field: ' + key
    for key, count in before.get('equivalent_attempts', {}).items():
        assert current['equivalent_attempts'][key] >= count
    assert relay.calls == 4 and relay.error is None
    call('new-worker-replay', 'task-worker-execute', worker_request, 412,
         error_contains='ReasoningAttemptRecorded')
    call('new-selector-replay', 'task-endpoint-propose', request, 412,
         error_contains='ReasoningAttemptRecorded')
    assert relay.calls == 4
    assert call('after-replays-budget', 'task-budget-read', selection) == budget
    assert call('original-selector-retained', 'task-attempt-read',
                {k: original_request[k] for k in ['scope', 'task_id', 'attempt_id']}) == original_selector
    assert call('original-worker-retained', 'task-attempt-read',
                {k: original_worker_request[k] for k in ['scope', 'task_id', 'attempt_id']}) == original_worker
    assert call('task-retained', 'task-read', {**selection,
                'max_storage_keys': request['max_storage_keys']})['task'] == task
    return {'resumeRequest': resume, 'stoppedBudget': stopped_budget, 'resumedBudget': resumed,
            'proposalRequest': request, 'proposal': proposal, 'selectorAttempt': selector,
            'workerRequest': worker_request, 'workerOutput': output, 'workerAttempt': worker,
            'budget': budget}
