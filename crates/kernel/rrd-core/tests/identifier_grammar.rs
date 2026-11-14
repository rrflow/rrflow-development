//! An identifier outside the canonical grammar cannot be constructed, so nothing can store a value
//! the public projection would later refuse.
use rrd_core::ident::canonical_id_problem;
use rrd_core::{
    OutboxId, Predicate, ProjectionId, Reader, RetentionPinId, RuntimeId, RuntimeRef, RuntimeType,
    ScopeId, SnapshotId, Subject,
};

fn candidates() -> Vec<String> {
    let mut values: Vec<String> = [
        "",
        "0",
        "a",
        "alpha",
        "a-b_c.d",
        "a1",
        "-a",
        "_a",
        ".a",
        "A",
        "aB",
        "a:b",
        "a/b",
        "a b",
        "a@b",
        "a\0b",
        "é",
        "cursor:7",
        "cursor.7",
        "vector:body",
    ]
    .iter()
    .map(|value| (*value).to_owned())
    .collect();
    values.push("a".repeat(128));
    values.push("a".repeat(129));
    values
}

macro_rules! holds_the_grammar {
    ($name:ident, $type:ty) => {
        #[test]
        fn $name() {
            for value in candidates() {
                let expected = canonical_id_problem(&value).is_none();
                assert_eq!(
                    <$type>::new(value.clone()).is_ok(),
                    expected,
                    "new({value:?})"
                );
                let text = serde_json::to_string(&value).unwrap();
                assert_eq!(
                    serde_json::from_str::<$type>(&text).is_ok(),
                    expected || value == "cursor:7" && stringify!($type) == "RuntimeId",
                    "deserialize {value:?}"
                );
            }
        }
    };
}

holds_the_grammar!(projection_id_holds_the_grammar, ProjectionId);
holds_the_grammar!(runtime_type_holds_the_grammar, RuntimeType);
holds_the_grammar!(runtime_id_holds_the_grammar, RuntimeId);
holds_the_grammar!(snapshot_id_holds_the_grammar, SnapshotId);
holds_the_grammar!(retention_pin_id_holds_the_grammar, RetentionPinId);
holds_the_grammar!(outbox_id_holds_the_grammar, OutboxId);
holds_the_grammar!(subject_holds_the_grammar, Subject);
holds_the_grammar!(predicate_holds_the_grammar, Predicate);

#[test]
fn a_refusal_names_the_kind_and_the_value() {
    let message = ProjectionId::new("vector:body").unwrap_err().to_string();
    assert!(message.contains("projection id"), "{message}");
    assert!(message.contains("vector:body"), "{message}");
}

#[test]
fn only_the_kernel_can_issue_an_event_identity() {
    let event = RuntimeId::event(7);
    assert_eq!(event.as_str(), "cursor:7");
    assert_eq!(event.event_cursor(), Some(7));

    // What a client could supply cannot name an event, in either spelling.
    assert!(RuntimeId::new("cursor:7").is_err());
    assert_eq!(RuntimeId::new("cursor.7").unwrap().event_cursor(), None);
    assert_ne!(RuntimeId::new("cursor.7").unwrap(), event);
    assert_eq!(RuntimeId::new("cursor-7").unwrap().event_cursor(), None);

    assert!(RuntimeRef::new("observed", "cursor:7").is_err());
    let reference = RuntimeRef::event(RuntimeType::new("observed").unwrap(), 7);
    assert_eq!(reference.id, event);
    assert_eq!(reference.kind.as_str(), "observed");
    assert_ne!(reference, RuntimeRef::new("observed", "cursor.7").unwrap());
}

#[test]
fn an_event_identity_reads_back_only_in_its_exact_form() {
    for cursor in [0, 7, u64::MAX] {
        let event = RuntimeId::event(cursor);
        assert_eq!(RuntimeId::parse(event.as_str()).unwrap(), event);
        assert_eq!(event.event_cursor(), Some(cursor));
        let text = serde_json::to_string(&event).unwrap();
        assert_eq!(serde_json::from_str::<RuntimeId>(&text).unwrap(), event);
    }
    assert_eq!(
        RuntimeRef::parse("observed", "cursor:7").unwrap(),
        RuntimeRef::event(RuntimeType::new("observed").unwrap(), 7)
    );
    assert_eq!(
        RuntimeId::parse("alpha").unwrap(),
        RuntimeId::new("alpha").unwrap()
    );
    for refused in [
        "cursor:",
        "cursor:x",
        "cursor:007",
        "cursor:+7",
        "cursor:-7",
        "cursor: 7",
        "cursor:7:8",
        "cursor:18446744073709551616",
        "Cursor:7",
        "other:7",
        "a:b",
        "",
    ] {
        assert!(RuntimeId::parse(refused).is_err(), "parse({refused:?})");
        assert!(
            serde_json::from_str::<RuntimeId>(&serde_json::to_string(refused).unwrap()).is_err(),
            "deserialize {refused:?}"
        );
    }
}

#[test]
fn an_event_identity_keeps_its_stored_bytes() {
    let reference = RuntimeRef::event(RuntimeType::new("observed").unwrap(), 7);
    assert_eq!(
        serde_json::to_string(&reference).unwrap(),
        r#"{"kind":"observed","id":"cursor:7"}"#
    );
}

#[test]
fn a_scope_and_a_reader_keep_their_namespaced_forms() {
    for scope in ["instance:alpha", "maintenance:alpha", "project:alpha"] {
        assert_eq!(ScopeId::new(scope).unwrap().as_str(), scope);
    }
    assert_eq!(
        Reader::new("agent:clyffy/worker").unwrap().as_str(),
        "agent:clyffy/worker"
    );
    assert!(ScopeId::new("").is_err());
    assert!(ScopeId::new("a\0b").is_err());
    assert!(Reader::new("").is_err());
    assert!(Reader::new("a\0b").is_err());
}
