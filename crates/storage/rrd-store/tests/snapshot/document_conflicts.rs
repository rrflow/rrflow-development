use super::*;
use rrd_store::{ControlTransition, Repositories};
use rrflow_core::{RuntimePropertySchema, RuntimeRetirement, RuntimeValue, RuntimeValueType};

fn put(read: &ReadStamp, id: &str, valid_from: u64, code: Option<&str>) -> DataTransaction {
    let mut commit = item(&read.scope, read.commit_cursor, id);
    let RuntimeMutation::Record { record } = &mut commit.mutations[0] else {
        unreachable!("item is a record fixture")
    };
    record.valid_from = valid_from;
    if let Some(code) = code {
        record
            .properties
            .insert("code".into(), RuntimeValue::String(code.into()));
    }
    DataTransaction::new(read.clone(), commit).unwrap()
}

#[test]
fn later_knowledge_conflicts_on_the_same_identity_across_valid_times_and_retirement() {
    let store = RrflowMxStore::new();
    let runtime = store.runtime();
    let scope = ScopeId::new("instance:historical-writers").unwrap();
    runtime.commit(&bootstrap(&scope, 0)).unwrap();
    runtime.commit(&item(&scope, 1, "left")).unwrap();
    runtime.commit(&item(&scope, 2, "right")).unwrap();
    let before = runtime.read_stamp(&scope).unwrap();
    runtime
        .commit_data_transaction(&put(&before, "left", 50, None))
        .unwrap();
    assert!(matches!(
        runtime.commit_data_transaction(&put(&before, "left", 102, None)),
        Err(Error::RuntimeConflict {
            expected: 3,
            actual: 4
        })
    ));
    let accepted = runtime
        .commit_data_transaction(&put(&before, "right", 102, None))
        .unwrap();
    assert_eq!(accepted.last_cursor, 5);
    let unstamped = item(&scope, before.commit_cursor, "unrelated");
    assert!(matches!(
        runtime.commit(&unstamped),
        Err(Error::RuntimeConflict {
            expected: 3,
            actual: 5
        })
    ));
    let event = DataTransaction::new(before.clone(), pulse(&scope, before.commit_cursor)).unwrap();
    assert!(matches!(
        runtime.commit_data_transaction(&event),
        Err(Error::RuntimeConflict {
            expected: 3,
            actual: 5
        })
    ));

    let before_retirement = runtime.read_stamp(&scope).unwrap();
    let mut retire = item(&scope, 5, "left");
    retire.mutations = vec![RuntimeMutation::Retire {
        retirement: RuntimeRetirement {
            model: RuntimeLogicalModel::Relational,
            reference: RuntimeRef::new("item", "left").unwrap(),
            effective_at: 120,
        },
    }];
    runtime.commit(&retire).unwrap();
    assert!(matches!(
        runtime.commit_data_transaction(&put(&before_retirement, "left", 130, None)),
        Err(Error::RuntimeConflict {
            expected: 5,
            actual: 6
        })
    ));
}

#[test]
fn disjoint_writers_cannot_publish_the_same_unique_value() {
    let store = RrflowMxStore::new();
    let runtime = store.runtime();
    let scope = ScopeId::new("instance:unique-writers").unwrap();
    let mut setup = bootstrap(&scope, 0);
    let RuntimeMutation::Schema { registry } = &mut setup.mutations[0] else {
        unreachable!("bootstrap is a schema fixture")
    };
    let rule = registry
        .records
        .get_mut(&RuntimeType::new("item").unwrap())
        .unwrap();
    rule.properties.insert(
        "code".into(),
        RuntimePropertySchema::required(RuntimeValueType::String),
    );
    rule.unique_properties.insert("code".into());
    runtime.commit(&setup).unwrap();
    let read = runtime.read_stamp(&scope).unwrap();
    runtime
        .commit_data_transaction(&put(&read, "one", 1, Some("shared")))
        .unwrap();
    let error = runtime
        .commit_data_transaction(&put(&read, "two", 1, Some("shared")))
        .unwrap_err();
    assert!(
        matches!(error, Error::Kernel(rrflow_core::Error::InvalidRuntime { reason }) if reason.contains("not unique"))
    );
    assert_eq!(runtime.cursor().unwrap(), 2);
}

#[test]
fn disjoint_documents_require_the_original_schema_and_catalogue() {
    let store = RrflowMxStore::new();
    let runtime = store.runtime();
    let scope = ScopeId::new("instance:catalogue-writers").unwrap();
    runtime.commit(&bootstrap(&scope, 0)).unwrap();
    let before_schema = runtime.read_stamp(&scope).unwrap();
    let mut update = bootstrap(&scope, 1);
    let RuntimeMutation::Schema { registry } = &mut update.mutations[0] else {
        unreachable!("bootstrap is a schema fixture")
    };
    registry.revision = 2;
    runtime.commit(&update).unwrap();
    assert!(matches!(
        runtime.commit_data_transaction(&put(&before_schema, "first", 20, None)),
        Err(Error::RuntimeConflict {
            expected: 1,
            actual: 2
        })
    ));
    let before_catalogue = runtime.read_stamp(&scope).unwrap();
    runtime.commit(&item(&scope, 2, "other")).unwrap();
    store
        .control()
        .commit_catalog(
            &scope,
            &ControlTransition {
                key: "server/state/catalogue/document-writers".into(),
                expected: None,
                replacement: Some(b"ready".to_vec()),
                at: 103,
                actor: "test".into(),
                action: "catalogue.ensure".into(),
                request_id: "catalogue-writers".into(),
                operation_id: "catalogue-writers".into(),
            },
        )
        .unwrap();
    assert!(matches!(
        runtime.commit_data_transaction(&put(&before_catalogue, "second", 20, None)),
        Err(Error::RuntimeConflict {
            expected: 2,
            actual: 3
        })
    ));
    assert_eq!(runtime.cursor().unwrap(), 3);
}
