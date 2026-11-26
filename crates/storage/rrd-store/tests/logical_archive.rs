use rrd_store::{
    export_logical_archive, inspect_logical_archive, restore_logical_archive_to_new_root,
    ControlTransition, Repositories, RrflowKvStore,
};
use rrflow_core::{
    Claim, DataTransaction, Predicate, Producer, RuntimeCommit, RuntimeMutation, ScopeId, Subject,
};
use rrflow_core::{
    RuntimeEventSchema, RuntimeLogicalModel, RuntimeRecord, RuntimeRef, RuntimeSchemaRegistry,
    RuntimeTableSchema, RuntimeType,
};
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};

fn claim(name: &str, at: u64) -> Claim {
    Claim::new(
        Subject::new(format!("subject-{name}")).unwrap(),
        Predicate::new("status").unwrap(),
        "verified",
        at,
        at,
        Producer {
            actor: "agent:archive-test".into(),
            on_behalf_of: None,
            session: Some("archive-test".into()),
        },
    )
}

fn runtime_claim(expected_cursor: u64, value: Claim) -> RuntimeCommit {
    RuntimeCommit {
        scope: ScopeId::new("instance:archive-test").unwrap(),
        at: value.tx_time,
        actor: "agent:archive-test".into(),
        expected_cursor,
        mutations: vec![RuntimeMutation::Claim { claim: value }],
    }
}

fn source(root: &std::path::Path) -> RrflowKvStore {
    let engine = RrflowKvStore::open(root).unwrap();
    engine
        .claims()
        .append_batch(&[claim("standalone-before", 10)])
        .unwrap();
    let mut registry = RuntimeSchemaRegistry::empty(1, "archive test bootstrap");
    registry
        .define_event_table(
            RuntimeType::new("archive_event").unwrap(),
            RuntimeLogicalModel::Event,
            RuntimeEventSchema::default(),
        )
        .unwrap();
    engine
        .runtime()
        .commit(&RuntimeCommit {
            scope: ScopeId::new("instance:archive-test").unwrap(),
            at: 15,
            actor: "agent:archive-test".into(),
            expected_cursor: 0,
            mutations: vec![RuntimeMutation::Schema { registry }],
        })
        .unwrap();
    engine
        .runtime()
        .commit(&runtime_claim(1, claim("runtime-one", 20)))
        .unwrap();
    engine
        .claims()
        .append_batch(&[claim("standalone-middle", 30)])
        .unwrap();
    engine
        .runtime()
        .commit(&runtime_claim(2, claim("runtime-two", 40)))
        .unwrap();
    engine
        .claims()
        .append_batch(&[claim("standalone-after", 50)])
        .unwrap();
    engine
}

#[test]
fn exports_inspects_and_restores_exact_log_coordinates() {
    let root = tempfile::tempdir().unwrap();
    let source_root = root.path().join("source");
    let archive = root.path().join("backup.rrd-archive");
    let target = root.path().join("restored");
    let source = source(&source_root);

    let exported = export_logical_archive(&source, &archive).unwrap();
    assert_eq!(exported.claim_sequence, 5);
    assert_eq!(exported.runtime_cursor, 3);
    assert_eq!(exported.standalone_claims, 3);
    assert_eq!(exported.runtime_commits, 3);
    assert_eq!(inspect_logical_archive(&archive).unwrap(), exported);

    let report = restore_logical_archive_to_new_root(&archive, &target, 100).unwrap();
    assert!(report.reopened);
    assert_eq!(report.inventory, exported);

    let restored = RrflowKvStore::open(&target).unwrap();
    assert_eq!(
        restored.claims().sequence().unwrap(),
        source.claims().sequence().unwrap()
    );
    assert_eq!(
        restored.runtime().cursor().unwrap(),
        source.runtime().cursor().unwrap()
    );
    assert_eq!(
        restored.claims().claims_in_range(0, 5).unwrap(),
        source.claims().claims_in_range(0, 5).unwrap()
    );
    assert_eq!(
        restored
            .runtime()
            .changes_since(0, 10, None)
            .unwrap()
            .changes,
        source.runtime().changes_since(0, 10, None).unwrap().changes
    );
}

#[test]
fn corruption_is_denied_before_target_publication_and_retry_succeeds() {
    let root = tempfile::tempdir().unwrap();
    let source = source(&root.path().join("source"));
    let archive = root.path().join("backup.rrd-archive");
    let target = root.path().join("restored");
    export_logical_archive(&source, &archive).unwrap();
    let original = std::fs::read(&archive).unwrap();

    let mut file = OpenOptions::new().write(true).open(&archive).unwrap();
    file.seek(SeekFrom::End(-1)).unwrap();
    file.write_all(&[0x7f]).unwrap();
    file.sync_all().unwrap();
    assert!(inspect_logical_archive(&archive).is_err());
    assert!(restore_logical_archive_to_new_root(&archive, &target, 100).is_err());
    assert!(!target.exists());
    assert!(!std::fs::read_dir(root.path()).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains("rrd-restore")
    }));

    std::fs::write(&archive, original).unwrap();
    restore_logical_archive_to_new_root(&archive, &target, 101).unwrap();
    assert_eq!(
        RrflowKvStore::open(&target)
            .unwrap()
            .claims()
            .sequence()
            .unwrap(),
        5
    );
}

#[test]
fn truncated_archive_and_existing_restore_root_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    let source = source(&root.path().join("source"));
    let archive = root.path().join("backup.rrd-archive");
    let target = root.path().join("existing");
    export_logical_archive(&source, &archive).unwrap();
    std::fs::create_dir(&target).unwrap();
    assert!(restore_logical_archive_to_new_root(&archive, &target, 100).is_err());

    let truncated = root.path().join("truncated.rrd-archive");
    let bytes = std::fs::read(&archive).unwrap();
    std::fs::write(&truncated, &bytes[..bytes.len() - 17]).unwrap();
    assert!(inspect_logical_archive(&truncated).is_err());
}

#[test]
fn restore_preserves_the_original_transaction_audit_envelope() {
    let root = tempfile::tempdir().unwrap();
    let source_root = root.path().join("source");
    let archive = root.path().join("audit.rrd-archive");
    let target = root.path().join("restored");
    let source = source(&source_root);
    let scope = ScopeId::new("instance:archive-test").unwrap();
    let read = source.runtime().read_stamp(&scope).unwrap();
    let transaction = DataTransaction::new(
        read,
        RuntimeCommit {
            scope,
            at: 60,
            actor: "agent:archive-transaction".into(),
            expected_cursor: 3,
            mutations: vec![RuntimeMutation::Claim {
                claim: claim("transactional", 60),
            }],
        },
    )
    .unwrap();
    let outcome = source
        .runtime()
        .commit_data_transaction(&transaction)
        .unwrap();
    let expected_audit = source.runtime().audit(&outcome.commit_id).unwrap().unwrap();
    assert_eq!(expected_audit.read.as_ref(), Some(&transaction.read));

    export_logical_archive(&source, &archive).unwrap();
    restore_logical_archive_to_new_root(&archive, &target, 100).unwrap();

    let restored = RrflowKvStore::open(&target).unwrap();
    assert_eq!(
        restored.runtime().audit(&outcome.commit_id).unwrap(),
        Some(expected_audit),
        "logical recovery must retain the transaction's exact read and audit chain"
    );
}

#[test]
fn disjoint_document_transactions_restore_original_read_and_publication_order() {
    let root = tempfile::tempdir().unwrap();
    let source = RrflowKvStore::open(&root.path().join("source")).unwrap();
    let scope = ScopeId::new("instance:archive-documents").unwrap();
    let mut registry = RuntimeSchemaRegistry::empty(1, "archive documents");
    registry.tables.insert(
        RuntimeType::new("document").unwrap(),
        RuntimeTableSchema::schemaless(RuntimeLogicalModel::Document),
    );
    source
        .runtime()
        .commit(&RuntimeCommit {
            scope: scope.clone(),
            at: 1,
            actor: "archive-test".into(),
            expected_cursor: 0,
            mutations: vec![RuntimeMutation::Schema { registry }],
        })
        .unwrap();
    // Logical archives replay runtime history. The separate control archive
    // owns this catalogue transition, while the runtime audit retains its stamp.
    source
        .control()
        .commit_catalog(
            &scope,
            &ControlTransition {
                key: "server/state/catalogue/documents".into(),
                expected: None,
                replacement: Some(b"ready".to_vec()),
                at: 1,
                actor: "archive-test".into(),
                action: "catalogue.ensure".into(),
                request_id: "catalogue-documents".into(),
                operation_id: "catalogue-documents".into(),
            },
        )
        .unwrap();
    let read = source.runtime().read_stamp(&scope).unwrap();
    assert_eq!(read.catalog_revision, 1);
    let mut outcomes = Vec::new();
    for id in ["left", "right"] {
        let commit = RuntimeCommit {
            scope: scope.clone(),
            at: 2,
            actor: id.into(),
            expected_cursor: read.commit_cursor,
            mutations: vec![RuntimeMutation::Record {
                record: RuntimeRecord {
                    reference: RuntimeRef::new("document", id).unwrap(),
                    valid_from: 1,
                    valid_to: None,
                    properties: Default::default(),
                },
            }],
        };
        let txn = DataTransaction::new(read.clone(), commit).unwrap();
        outcomes.push(source.runtime().commit_data_transaction(&txn).unwrap());
    }
    assert_eq!(outcomes[0].last_cursor, 2);
    assert_eq!(outcomes[1].first_cursor, 3);
    let archive = root.path().join("documents.rrd-archive");
    let inventory = export_logical_archive(&source, &archive).unwrap();
    assert_eq!(inspect_logical_archive(&archive).unwrap(), inventory);
    let target = root.path().join("restored");
    restore_logical_archive_to_new_root(&archive, &target, 100).unwrap();
    let restored = RrflowKvStore::open(&target).unwrap();
    for outcome in outcomes {
        let audit = restored
            .runtime()
            .audit(&outcome.commit_id)
            .unwrap()
            .unwrap();
        assert_eq!(audit.read.as_ref(), Some(&read));
        assert_eq!(audit.outcome_cursor, Some(outcome.last_cursor));
        assert_eq!(
            restored
                .runtime()
                .commit_outcome(&outcome.commit_id)
                .unwrap(),
            Some(outcome)
        );
    }
    let second_export = root.path().join("restored.rrd-archive");
    assert_eq!(
        export_logical_archive(&restored, &second_export).unwrap(),
        inventory
    );
    assert_eq!(
        std::fs::read(second_export).unwrap(),
        std::fs::read(archive).unwrap()
    );
}
