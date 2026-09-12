#[path = "support/projected_read_model.rs"]
mod projected_read_model;

use projected_read_model::{
    directed_failure_program, directed_lifetime_program, run_encoded_scenario,
    run_seeded_scenarios, ScenarioConfig,
};

#[test]
fn fixed_seed_state_machine_matches_independent_model() {
    let report = run_seeded_scenarios(&ScenarioConfig {
        seed: 0xca81_5e11_7e57_0001,
        cases: 4,
        operations: 64,
        retained_root: None,
    })
    .expect("fixed adversarial scenarios must be valid");

    assert_eq!(report.cases, 4);
    assert_eq!(report.operations, 4 * 64);
    assert!(report.writes > 0);
    assert!(report.projected_reads > 0);
    assert!(report.verification_rounds > 0);
}

#[test]
fn all_failure_boundaries_reopen_inside_mixed_histories() {
    let report = run_encoded_scenario(0xca81_5e11_fa17_0002, &directed_failure_program());

    assert_eq!(report.injected_failures, 10);
    assert_eq!(report.reopens, 10);
    assert_eq!(report.writes, 9);
    assert!(report.flushes > 0);
    assert!(report.verification_rounds > 0);
}

#[test]
fn projected_stream_limits_cancel_and_release_views() {
    let report = run_encoded_scenario(0xca81_5e11_11fe_0003, &directed_lifetime_program());

    assert_eq!(report.cancellations, 1);
    assert_eq!(report.resource_denials, 2);
    assert!(report.projected_reads > 0);
    assert!(report.garbage_collections > 0);
    assert!(report.reopens > 0);
    assert!(report.verification_rounds > 0);
}
