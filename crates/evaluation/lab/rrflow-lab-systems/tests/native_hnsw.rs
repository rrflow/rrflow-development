//! An RRFlow graph is built with what its binding states. The adapter holds no value of its own,
//! and the engine's contract, not the lab, says which values are allowed.

use rrflow_evaluation_core::{Availability, Id};
use rrflow_lab_systems::binding::{
    CredentialReferences, IndexSpec, RrflowHnswParameters, SearchSetting, SystemBinding,
    SystemKind, SystemLocation,
};

fn native() -> RrflowHnswParameters {
    RrflowHnswParameters {
        max_level: 6,
        seed: 7,
        build_policy: Default::default(),
    }
}

fn graph(kind: SystemKind) -> SystemBinding {
    SystemBinding {
        id: Id::try_from("subject".to_string()).unwrap(),
        label: "Subject".into(),
        kind,
        location: SystemLocation::Loopback,
        endpoint: "http://127.0.0.1:9".into(),
        artifact: "pinned".into(),
        storage_profile: "disk".into(),
        availability: Availability::Available,
        index: IndexSpec::Hnsw {
            m: 16,
            ef_construction: 100,
        },
        rrflow_hnsw: None,
        search_ladder: vec![SearchSetting {
            id: Id::try_from("ef-32".to_string()).unwrap(),
            ef: Some(32),
            exact_rerank: None,
            nprobes: None,
        }],
        credentials: CredentialReferences::default(),
        memory_probe: None,
        disk_probe: None,
        engine_settings: Default::default(),
        limits: Vec::new(),
    }
}

#[test]
fn an_rrflow_graph_states_its_native_build_parameters() {
    let mut rrflow = graph(SystemKind::Rrflow);
    // Without them there is nothing to build the graph with, and no default stands in.
    assert_eq!(
        rrflow.validate(10).unwrap_err(),
        "subject: native HNSW parameters are required"
    );
    rrflow.rrflow_hnsw = Some(native());
    rrflow.validate(10).unwrap();
}

#[test]
fn the_engine_contract_bounds_the_native_parameters() {
    let mut rrflow = graph(SystemKind::Rrflow);
    for max_level in [0, 33] {
        rrflow.rrflow_hnsw = Some(RrflowHnswParameters {
            max_level,
            ..native()
        });
        let refusal = rrflow.validate(10).unwrap_err();
        assert!(
            refusal.starts_with("subject: native HNSW parameters are outside engine bounds"),
            "{refusal}"
        );
    }
}

#[test]
fn native_build_parameters_belong_only_to_an_rrflow_graph() {
    let mut other = graph(SystemKind::Qdrant);
    other.validate(10).unwrap();
    other.rrflow_hnsw = Some(native());
    assert_eq!(
        other.validate(10).unwrap_err(),
        "subject: native HNSW parameters require RRFlow HNSW"
    );

    let mut exact = graph(SystemKind::Rrflow);
    exact.index = IndexSpec::Exact;
    exact.search_ladder[0].ef = None;
    exact.rrflow_hnsw = Some(native());
    assert_eq!(
        exact.validate(10).unwrap_err(),
        "subject: native HNSW parameters require RRFlow HNSW"
    );
    exact.rrflow_hnsw = None;
    exact.validate(10).unwrap();
}

#[test]
fn a_system_that_does_not_run_need_not_state_them() {
    let mut held = graph(SystemKind::Rrflow);
    held.availability = Availability::Unavailable {
        reason: "not started on this machine".into(),
    };
    held.validate(10).unwrap();
}

#[test]
fn the_published_rrflow_graph_states_its_parameters() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../eval/lab/systems/rrflow-hnsw.json"
    ))
    .unwrap();
    let definition: serde_json::Value = serde_json::from_str(&text).unwrap();
    let binding: SystemBinding = serde_json::from_value(definition["binding"].clone()).unwrap();
    assert_eq!(binding.rrflow_hnsw, Some(native()));
    binding.validate(10).unwrap();
}
