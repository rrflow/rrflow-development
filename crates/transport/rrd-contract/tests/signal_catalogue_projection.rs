use rrd_core::{signal_catalogue, signal_catalogue_sha256};
use serde_json::Value;
use std::collections::BTreeSet;

const SIGNAL_CATALOGUE_SHA256: &str =
    "239df2ced5974d4351ca01565369646964cb65c63d0fbae01dddccb4c3ac5a07";

#[test]
fn openapi_projects_the_exact_kernel_signal_catalogue() {
    let document = rrd_contract::openapi_document().expect("OpenAPI document must build");
    let catalogue = signal_catalogue();
    catalogue
        .validate()
        .expect("kernel signal catalogue must be valid");
    let expected = serde_json::to_value(&catalogue).expect("catalogue must serialize");

    assert_eq!(
        document.get("x-rrd-signal-catalogue"),
        Some(&expected),
        "OpenAPI must project the kernel catalogue without interpretation"
    );
    assert_eq!(
        document
            .get("x-rrd-signal-catalogue-sha256")
            .and_then(Value::as_str),
        Some(SIGNAL_CATALOGUE_SHA256)
    );
    assert_eq!(
        signal_catalogue_sha256().expect("kernel catalogue must fingerprint"),
        SIGNAL_CATALOGUE_SHA256
    );

    assert_eq!(catalogue.diagnostic_levels.len(), 4);
    assert_eq!(catalogue.metric_attributes.len(), 9);
    assert_eq!(catalogue.metric_instruments.len(), 22);
}

#[test]
fn signal_projection_is_discovery_metadata_not_runtime_state() {
    let document = rrd_contract::openapi_document().expect("OpenAPI document must build");
    let projection = document
        .get("x-rrd-signal-catalogue")
        .and_then(Value::as_object)
        .expect("OpenAPI must contain an object-valued signal catalogue");

    let expected_top_level = BTreeSet::from([
        "contract_version",
        "default_diagnostic_level",
        "default_metric_cardinality_limit",
        "diagnostic_levels",
        "metric_attributes",
        "metric_instruments",
        "runtime_trace_contract_version",
        "trace_attributes",
        "trace_operations",
    ]);
    let actual_top_level: BTreeSet<&str> = projection.keys().map(String::as_str).collect();
    assert_eq!(actual_top_level, expected_top_level);

    let encoded = serde_json::to_string(projection).expect("projection must serialize");
    for forbidden_key in [
        "activation",
        "active_level",
        "current_level",
        "diagnostic_snapshot",
        "enabled",
        "events",
        "exporter",
        "lifecycle",
        "samples",
        "subscriber",
        "timestamp",
    ] {
        assert!(
            !encoded.contains(&format!("\"{forbidden_key}\":")),
            "signal discovery projection must not contain runtime key {forbidden_key}"
        );
    }
}
