use rrd_contract::{
    knowledge_package_sha256, knowledge_record_sha256, CanonicalId, KnowledgeClassification,
    KnowledgeExclusionV1, KnowledgeManifestEntryV1, KnowledgePackageV1, KnowledgeProvenance,
    KnowledgeRecordV1, KnowledgeSourceInventoryEntryV1, KNOWLEDGE_PACKAGE_CONTRACT_VERSION,
};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct GoldenKnowledgeContract {
    source_inventory: Vec<KnowledgeSourceInventoryEntryV1>,
    package: KnowledgePackageV1,
}

fn canonical_id(value: &str) -> CanonicalId {
    CanonicalId::new(value).unwrap()
}

fn digest(bytes: impl AsRef<[u8]>) -> String {
    Sha256::digest(bytes.as_ref())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn provenance() -> KnowledgeProvenance {
    KnowledgeProvenance {
        repository: canonical_id("rrflow"),
        revision: "70c528ef22670c528ef22670c528ef22670c528e".into(),
        exporter: canonical_id("rrflow-knowledge-exporter"),
        exporter_version: 1,
    }
}

fn record(
    coordinate: &str,
    source_path: &str,
    classification: KnowledgeClassification,
    owner_coordinate: &str,
    body: &str,
) -> KnowledgeRecordV1 {
    let mut record = KnowledgeRecordV1 {
        contract_version: KNOWLEDGE_PACKAGE_CONTRACT_VERSION,
        coordinate: coordinate.into(),
        source_path: source_path.into(),
        classification,
        owner_coordinate: owner_coordinate.into(),
        body: body.into(),
        body_sha256: digest(body),
        provenance: provenance(),
        record_sha256: digest([]),
    };
    record.record_sha256 = knowledge_record_sha256(&record).unwrap();
    record
}

fn golden_contract() -> GoldenKnowledgeContract {
    let records = vec![
        record(
            "rrflow://rrflow-instance/data/architecture/engine-data-flow",
            "docs/architecture/engine-data-flow.md",
            KnowledgeClassification::Architecture,
            "rrflow://rrflow-instance/data/architecture-index/rrflow-architecture",
            "# Engine data flow\n\nAuthoritative stamped flow.\n",
        ),
        record(
            "rrflow://rrflow-instance/data/roadmap/rrflow-1.0",
            "docs/roadmap/rrflow-1.0.md",
            KnowledgeClassification::Roadmap,
            "rrflow://rrflow-instance/data/roadmap-index/rrflow-roadmaps",
            "# RRFlow 1.0 roadmap\n\nOrdered evidence gates.\n",
        ),
    ];
    let exclusions = vec![KnowledgeExclusionV1 {
        source_path: "docs/unclassified-draft.md".into(),
        source_sha256: digest("# Draft\n"),
        reason_code: canonical_id("unclassified-record"),
        reason: "The source has no accepted owner or stable coordinate.".into(),
    }];
    let mut manifest = records
        .iter()
        .map(KnowledgeManifestEntryV1::included)
        .chain(exclusions.iter().map(KnowledgeManifestEntryV1::excluded))
        .collect::<Vec<_>>();
    manifest.sort_by(|left, right| left.source_path.cmp(&right.source_path));
    let source_inventory = manifest
        .iter()
        .map(|entry| KnowledgeSourceInventoryEntryV1 {
            source_path: entry.source_path.clone(),
            source_sha256: entry.source_sha256.clone(),
        })
        .collect();
    let mut package = KnowledgePackageV1 {
        contract_version: KNOWLEDGE_PACKAGE_CONTRACT_VERSION,
        id: canonical_id("rrflow-bootstrap-knowledge"),
        provenance: provenance(),
        manifest,
        records,
        exclusions,
        package_sha256: digest([]),
    };
    package.package_sha256 = knowledge_package_sha256(&package).unwrap();
    GoldenKnowledgeContract {
        source_inventory,
        package,
    }
}

fn reseal_record(record: &mut KnowledgeRecordV1) {
    record.record_sha256 = knowledge_record_sha256(record).unwrap();
}

fn reseal_package(package: &mut KnowledgePackageV1) {
    package.package_sha256 = knowledge_package_sha256(package).unwrap();
}

#[test]
fn knowledge_package_matches_golden_and_reopens() {
    let fixture = golden_contract();
    fixture
        .package
        .validate_against_inventory(&fixture.source_inventory)
        .unwrap();

    let expected: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/knowledge-package-v1.json")).unwrap();
    let actual = serde_json::to_value(&fixture).unwrap();
    assert_eq!(
        actual,
        expected,
        "{}",
        serde_json::to_string_pretty(&actual).unwrap()
    );
    let reopened: GoldenKnowledgeContract = serde_json::from_value(expected).unwrap();
    reopened
        .package
        .validate_against_inventory(&reopened.source_inventory)
        .unwrap();
    assert_eq!(reopened, fixture);
}

#[test]
fn generated_schema_is_closed_for_every_object_and_disposition() {
    let schema = serde_json::to_value(schema_for!(GoldenKnowledgeContract)).unwrap();
    assert_eq!(schema["additionalProperties"], serde_json::json!(false));
    let definitions = schema["$defs"].as_object().unwrap();
    for name in [
        "KnowledgeExclusionV1",
        "KnowledgeManifestEntryV1",
        "KnowledgePackageV1",
        "KnowledgeProvenance",
        "KnowledgeRecordV1",
        "KnowledgeSourceInventoryEntryV1",
    ] {
        assert!(definitions.contains_key(name), "missing schema {name}");
        assert_eq!(
            definitions[name]["additionalProperties"],
            serde_json::json!(false),
            "schema {name} must reject unknown fields"
        );
    }
    let dispositions = definitions["KnowledgeManifestDispositionV1"]["oneOf"]
        .as_array()
        .unwrap();
    assert_eq!(dispositions.len(), 2);
    assert!(dispositions
        .iter()
        .all(|option| option["additionalProperties"] == serde_json::json!(false)));

    let mut record = serde_json::to_value(&golden_contract().package.records[0]).unwrap();
    record["provider"] = serde_json::json!("openai");
    assert!(serde_json::from_value::<KnowledgeRecordV1>(record).is_err());

    let mut package = serde_json::to_value(&golden_contract().package).unwrap();
    package["import_path"] = serde_json::json!("/tmp/rrflow");
    assert!(serde_json::from_value::<KnowledgePackageV1>(package).is_err());
}

#[test]
fn records_reject_unsafe_coordinates_paths_and_non_normalized_content() {
    let original = golden_contract().package.records[0].clone();

    let mut nested_coordinate = original.clone();
    nested_coordinate.coordinate =
        "rrflow://rrflow-instance/data/reference/storage/rrflowkv-current-format".into();
    reseal_record(&mut nested_coordinate);
    nested_coordinate.validate().unwrap();

    let mut unsafe_coordinate = original.clone();
    unsafe_coordinate.coordinate = "file:///tmp/knowledge.md".into();
    reseal_record(&mut unsafe_coordinate);
    assert!(unsafe_coordinate.validate().is_err());

    let mut empty_coordinate_segment = original.clone();
    empty_coordinate_segment.coordinate =
        "rrflow://rrflow-instance/data/reference//rrflowkv-current-format".into();
    reseal_record(&mut empty_coordinate_segment);
    assert!(empty_coordinate_segment.validate().is_err());

    let mut unsafe_path = original.clone();
    unsafe_path.source_path = "../outside.md".into();
    reseal_record(&mut unsafe_path);
    assert!(unsafe_path.validate().is_err());

    let mut windows_path = original.clone();
    windows_path.source_path = r"docs\architecture\flow.md".into();
    reseal_record(&mut windows_path);
    assert!(windows_path.validate().is_err());

    let mut crlf = original.clone();
    crlf.body = "# Engine data flow\r\n".into();
    crlf.body_sha256 = digest(&crlf.body);
    reseal_record(&mut crlf);
    assert!(crlf.validate().is_err());

    let mut missing_final_newline = original.clone();
    missing_final_newline.body = "# Engine data flow".into();
    missing_final_newline.body_sha256 = digest(&missing_final_newline.body);
    reseal_record(&mut missing_final_newline);
    assert!(missing_final_newline.validate().is_err());

    let mut body_mismatch = original;
    body_mismatch.body.push_str("changed\n");
    reseal_record(&mut body_mismatch);
    assert!(body_mismatch.validate().is_err());
}

#[test]
fn package_rejects_unstable_order_duplicates_and_manifest_drift() {
    let fixture = golden_contract();

    let mut records_out_of_order = fixture.package.clone();
    records_out_of_order.records.reverse();
    reseal_package(&mut records_out_of_order);
    assert!(records_out_of_order.validate().is_err());

    let mut manifest_out_of_order = fixture.package.clone();
    manifest_out_of_order.manifest.reverse();
    reseal_package(&mut manifest_out_of_order);
    assert!(manifest_out_of_order.validate().is_err());

    let mut duplicate_coordinate = fixture.package.clone();
    duplicate_coordinate.records[1].coordinate = duplicate_coordinate.records[0].coordinate.clone();
    reseal_record(&mut duplicate_coordinate.records[1]);
    duplicate_coordinate.manifest = duplicate_coordinate
        .records
        .iter()
        .map(KnowledgeManifestEntryV1::included)
        .chain(
            duplicate_coordinate
                .exclusions
                .iter()
                .map(KnowledgeManifestEntryV1::excluded),
        )
        .collect();
    duplicate_coordinate
        .manifest
        .sort_by(|left, right| left.source_path.cmp(&right.source_path));
    reseal_package(&mut duplicate_coordinate);
    assert!(duplicate_coordinate.validate().is_err());

    let mut manifest_drift = fixture.package.clone();
    manifest_drift.manifest[0].source_sha256 = digest("different source");
    reseal_package(&mut manifest_drift);
    assert!(manifest_drift.validate().is_err());
}

#[test]
fn independent_inventory_detects_every_eligible_omission_and_changed_source() {
    let fixture = golden_contract();

    let mut omitted = fixture.package.clone();
    omitted.records.pop();
    omitted
        .manifest
        .retain(|entry| entry.source_path != "docs/roadmap/rrflow-1.0.md");
    reseal_package(&mut omitted);
    omitted.validate().unwrap();
    assert!(omitted
        .validate_against_inventory(&fixture.source_inventory)
        .is_err());

    let mut changed_inventory = fixture.source_inventory.clone();
    changed_inventory[0].source_sha256 = digest("changed after discovery");
    assert!(fixture
        .package
        .validate_against_inventory(&changed_inventory)
        .is_err());

    let mut unsorted_inventory = fixture.source_inventory.clone();
    unsorted_inventory.reverse();
    assert!(fixture
        .package
        .validate_against_inventory(&unsorted_inventory)
        .is_err());
}

#[test]
fn exclusions_require_normalized_reason_and_exact_manifest_binding() {
    let fixture = golden_contract();

    let mut blank_reason = fixture.package.clone();
    blank_reason.exclusions[0].reason = "  ".into();
    reseal_package(&mut blank_reason);
    assert!(blank_reason.validate().is_err());

    let mut changed_reason_code = fixture.package.clone();
    changed_reason_code.exclusions[0].reason_code = canonical_id("different-reason");
    reseal_package(&mut changed_reason_code);
    assert!(changed_reason_code.validate().is_err());

    let mut package_digest_drift = fixture.package;
    package_digest_drift.package_sha256 = digest("tampered package digest");
    assert!(package_digest_drift.validate().is_err());
}

#[test]
fn package_and_record_digests_bind_field_boundaries_and_provenance() {
    let fixture = golden_contract();
    let original_record_digest = fixture.package.records[0].record_sha256.clone();
    let original_package_digest = fixture.package.package_sha256.clone();

    let mut owner_changed = fixture.package.records[0].clone();
    owner_changed.owner_coordinate =
        "rrflow://rrflow-instance/data/architecture-index/other-owner".into();
    reseal_record(&mut owner_changed);
    assert_ne!(owner_changed.record_sha256, original_record_digest);

    let mut revision_changed = fixture.package;
    revision_changed.provenance.revision = "different-revision".into();
    for record in &mut revision_changed.records {
        record.provenance = revision_changed.provenance.clone();
        reseal_record(record);
    }
    revision_changed.manifest = revision_changed
        .records
        .iter()
        .map(KnowledgeManifestEntryV1::included)
        .chain(
            revision_changed
                .exclusions
                .iter()
                .map(KnowledgeManifestEntryV1::excluded),
        )
        .collect();
    revision_changed
        .manifest
        .sort_by(|left, right| left.source_path.cmp(&right.source_path));
    reseal_package(&mut revision_changed);
    assert_ne!(revision_changed.package_sha256, original_package_digest);
    revision_changed.validate().unwrap();
}
