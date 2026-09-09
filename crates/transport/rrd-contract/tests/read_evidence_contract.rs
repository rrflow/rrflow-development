use rrd_contract::{
    ReadAccessPath, ReadEvidence, ReadPathEvidence, ReadStampValidationEvidence,
    ReadStampValidationMethod, READ_EVIDENCE_CONTRACT_VERSION,
};

fn fixture() -> ReadEvidence {
    ReadEvidence {
        contract_version: READ_EVIDENCE_CONTRACT_VERSION,
        key_budget: 64,
        point_reads: 7,
        range_scans: 1,
        keys_examined: 9,
        values_decoded: 8,
        decoded_bytes: 512,
        stamp_validation: ReadStampValidationEvidence {
            method: ReadStampValidationMethod::Rfc9162DirectVersions,
            change_reads: 2,
            proof_nodes: 4,
        },
        paths: vec![
            ReadPathEvidence {
                path: ReadAccessPath::ReadStamp,
                point_reads: 5,
                range_scans: 0,
                keys_examined: 5,
                values_decoded: 4,
                decoded_bytes: 256,
            },
            ReadPathEvidence {
                path: ReadAccessPath::RecordVersions,
                point_reads: 0,
                range_scans: 1,
                keys_examined: 2,
                values_decoded: 2,
                decoded_bytes: 128,
            },
            ReadPathEvidence {
                path: ReadAccessPath::AccumulatorProof,
                point_reads: 2,
                range_scans: 0,
                keys_examined: 2,
                values_decoded: 2,
                decoded_bytes: 128,
            },
        ],
    }
}

#[test]
fn read_evidence_matches_closed_provider_neutral_golden() {
    let evidence = fixture();
    evidence.validate().unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/read-evidence-v1.json")).unwrap();
    assert_eq!(serde_json::to_value(&evidence).unwrap(), expected);
    assert_eq!(
        serde_json::from_value::<ReadEvidence>(expected).unwrap(),
        evidence
    );
}

#[test]
fn read_evidence_rejects_unknown_paths_false_totals_and_invalid_head_proofs() {
    let mut unknown = serde_json::to_value(fixture()).unwrap();
    unknown["paths"][0]["backend"] = serde_json::json!("rocksdb");
    assert!(serde_json::from_value::<ReadEvidence>(unknown).is_err());

    let mut wrong_total = fixture();
    wrong_total.keys_examined += 1;
    assert!(wrong_total.validate().is_err());

    let mut duplicate_path = fixture();
    duplicate_path.paths[1].path = ReadAccessPath::ReadStamp;
    assert!(duplicate_path.validate().is_err());

    let mut invalid_head = fixture();
    invalid_head.stamp_validation.method = ReadStampValidationMethod::AuthenticatedCurrentHead;
    assert!(invalid_head.validate().is_err());
}
