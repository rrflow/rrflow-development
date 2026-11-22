//! Public session leases on an installed project, exercised across a forced restart.
use super::Workload;
use crate::options;
use rrd_client::Session;
use rrd_contract::{
    transaction_operation_sha256, BeginTransaction, CanonicalId, CommitReceipt, CommitTransaction,
    DataReference, ErrorCode, PreviewTransaction, QueryValue, TransactionLease,
    TransactionMutation,
};
use std::collections::BTreeMap;

fn document(id: &str, revision: u64, valid_at: u64) -> CommitTransaction {
    let mutations = vec![TransactionMutation::PutRecord {
        reference: DataReference {
            kind: CanonicalId::new(super::workload::WORKLOAD_RECORD_KIND).unwrap(),
            id: CanonicalId::new(id).unwrap(),
        },
        valid_from: valid_at,
        valid_to: None,
        properties: BTreeMap::from([("revision".into(), QueryValue::Unsigned(revision))]),
    }];
    CommitTransaction {
        operation_sha256: transaction_operation_sha256(&mutations),
        mutations,
    }
}

async fn begin(work: &Workload, session: &Session, name: &str) -> TransactionLease {
    work.client
        .begin_transaction(
            session,
            BeginTransaction {
                scope: CanonicalId::new("data").unwrap(),
                timeout_ms: 60_000,
            },
            options(name),
        )
        .await
        .unwrap()
}

pub(super) struct DocumentWrites {
    right: CommitTransaction,
    receipt: CommitReceipt,
}

pub(super) async fn exercise(work: &Workload, peer: &Session) -> DocumentWrites {
    let left = begin(work, &work.session, "document-left-begin").await;
    let right = begin(work, peer, "document-right-begin").await;
    let overlap = begin(work, peer, "document-overlap-begin").await;
    assert_ne!(left.session_id, right.session_id);
    assert_eq!(left.read_cursor, right.read_cursor);
    assert_eq!(left.read_cursor, overlap.read_cursor);

    let first = document("writer-left", 1, work.valid_at);
    let second = document("writer-right", 2, work.valid_at);
    let accepted = work
        .client
        .commit_transaction(
            &work.session,
            &left.transaction_id,
            first.clone(),
            options("document-left-commit"),
        )
        .await
        .unwrap();
    assert_eq!(accepted.last_runtime_cursor, Some(left.read_cursor + 1));

    let preview = work
        .client
        .preview_transaction(
            peer,
            &right.transaction_id,
            PreviewTransaction {
                mutations: second.mutations.clone(),
                valid_at: Some(work.valid_at),
                max_storage_keys: 4096,
            },
            options("document-right-preview"),
        )
        .await
        .unwrap();
    assert_eq!(preview.read_cursor, right.read_cursor);
    let entries = &preview.prospective.entries;
    assert!(entries.iter().any(|e| e.value == second.mutations[0]));
    assert!(!entries.iter().any(|e| e.value == first.mutations[0]));

    let receipt = work
        .client
        .commit_transaction(
            peer,
            &right.transaction_id,
            second.clone(),
            options("document-right-commit"),
        )
        .await
        .unwrap();
    assert!(!receipt.idempotent_replay);
    assert_eq!(receipt.last_runtime_cursor, Some(left.read_cursor + 2));

    let conflict = work
        .client
        .commit_transaction(
            peer,
            &overlap.transaction_id,
            document("writer-left", 3, work.valid_at),
            options("document-overlap-commit"),
        )
        .await
        .unwrap_err();
    assert!(matches!(conflict, rrd_client::Error::Api { status, error }
        if status.as_u16() == 409 && error.code == ErrorCode::Conflict && !error.retryable));
    println!(
        "installed document writers: read_cursor={} left_commit={} right_commit={} overlap_status=409",
        right.read_cursor,
        accepted.last_runtime_cursor.unwrap(),
        receipt.last_runtime_cursor.unwrap()
    );
    DocumentWrites {
        right: second,
        receipt,
    }
}

impl DocumentWrites {
    pub(super) async fn verify_reopened(&self, work: &Workload, peer: &Session) {
        // Read current values before a replay can change what the read observes.
        let result = work
            .query(
                "record:capacity_document",
                r#"id >= "writer-left" AND id <= "writer-right""#,
                "id, revision",
                None,
            )
            .await
            .unwrap();
        assert_eq!(
            Some(result.known_at_cursor),
            self.receipt.last_runtime_cursor
        );
        assert_eq!(result.rows.len(), 2);
        for (id, revision) in [("writer-left", 1), ("writer-right", 2)] {
            assert!(result.rows.iter().any(|row| row.values.get("id")
                == Some(&QueryValue::String(id.into()))
                && row.values.get("revision") == Some(&QueryValue::Unsigned(revision))));
        }
        let replay = work
            .client
            .commit_transaction(
                peer,
                &self.receipt.transaction_id,
                self.right.clone(),
                options("document-right-commit"),
            )
            .await
            .unwrap();
        assert_eq!(
            replay,
            CommitReceipt {
                idempotent_replay: true,
                ..self.receipt.clone()
            }
        );
        println!(
            "reopened document writers: rows={} head={} replay_commit_sha256={}",
            result.rows.len(),
            result.known_at_cursor,
            replay.runtime_commit_sha256.as_deref().unwrap()
        );
    }
}
