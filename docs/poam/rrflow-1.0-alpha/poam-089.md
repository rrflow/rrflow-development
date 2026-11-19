# POAM-089 — installed process suites sometimes fail on a busy Windows host

**Status:** active parent-owned POA&M detail
**Coordinate:** `rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha/poam-089`
**Owner:** [RRFlow 1.0 alpha POA&M](../rrflow-1.0-alpha.md)

This detail is governed by the parent ledger; it cannot independently change
priority or closure status.

## Observed deficiency

Observed on 2026-11-19 on one Windows 11 host that other work was using at the
same time, while running the gates of
[the instance registry and stop package](../../evidence/change-journals/gate-d/instance-registry-and-stop-20261118.md).
The causes are not known.

- **`installed_crash` loses the first request to a server it just started.**
  `cargo test -p rrflow-cli --features crash-test --test installed_crash`
  failed in 3 of 65 runs, each time in a different test
  (`fresh_installed_publication_and_reclamation`,
  `capacity::cases::installed_capacity_wal_payload_is_observed_before_domain_commit`,
  `existing_installed_domain_boundaries`) and each time with
  `Transport("client error (SendRequest)")` on a request made right after
  `serve` announced or resumed. Each failing run took 36 to 59 seconds; a
  passing run takes about 23. Forty-eight of those runs compared the suite
  with and without that package's additions to a starting server (the
  registry write and the stop-request look): 24 of 24 passed either way, so
  the comparison does not say whether the additions matter.
- **`rrflow ready` allows five seconds a request.** `ClientConfig::default`
  sets `request_timeout` to 5 s with two attempts, and `rrflow ready` uses it
  for a discovery that returns a 9.6 MB OpenAPI document. In
  `installed_lifecycle` runs that took 43 to 61 seconds instead of 28, `ready`
  failed with `RRD request timed out` in tests that had passed minutes before.
- **One engine test failed once.**
  `engine::tests::reasoning_budget::worker::source_bound::assessment::dispatch_guards::selected_source_change_during_assessment_dispatch_withholds_output_without_retry`
  failed in 1 of 4 full `cargo test -p rrd-engine --lib` runs (it expected
  `TaskAttemptInvalidated(InputsChanged)`) and passed 10 of 10 times alone.

## Impact

A gate run on a shared host can fail for a change that is correct, and a
passing run does not show that a just-started server always answers its first
request. If the first item is a real window between the announcement and the
first accepted request, a client that connects as soon as `serve` announces
can lose a request.

## Owning gates

D-01 owns the served lifecycle and its readiness proof.

Roadmap owner: [Gate D](../../roadmap/rrflow-1.0/gate-d.md).

## Closure evidence

Closed when the cause of each of the three is recorded with a test that fails
on it, and either fixed or shown to be a bound the test should not assume on a
busy host; and `rrflow ready` takes its request timeout from a named default
with a documented override.
