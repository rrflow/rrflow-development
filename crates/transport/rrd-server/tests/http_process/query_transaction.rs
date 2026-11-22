//! The public QL transaction program must retain both temporal coordinates.
use super::*;

const PATH: &str = "/v1/query/transactions";

fn program(program: &str, bindings: Value, key: &str) -> Value {
    envelope(
        json!({"scope": "instance:socket-test", "program": program,
            "mutation_bindings": bindings, "timeout_ms": 60_000}),
        Some(key),
        None,
    )
}

fn document(id: &str, title: &str) -> Value {
    json!({"mutation": "put_record", "reference": {"kind": "document", "id": id},
        "valid_from": 100, "properties": {"title": {"type": "string", "value": title}}})
}

fn read(
    server: &RunningServer,
    session: (&str, &str),
    cursor: impl std::fmt::Display,
    valid: u64,
) -> Value {
    let request = envelope(
        json!({"scope": "instance:socket-test",
            "query": format!("FROM record:document AT VALID {valid} KNOWN {cursor} WHERE id = \"alpha\" PROJECT title EXPLAIN CONTRACT"),
            "parameters": {}, "budget": {"max_storage_keys": 128, "max_rows": 10,
                "max_output_bytes": 16384, "max_batch_rows": 10}}),
        None,
        None,
    );
    let (status, result) = post(server, "/v1/query", &request, Some(session));
    assert_eq!(status, 200, "{result}");
    payload(&result).clone()
}

#[test]
fn public_query_transaction_preserves_corrections_cancel_and_replay_across_restart() {
    let (_temporary, root, server) = start_root();
    let create = envelope(
        json!({"limits": {"idle_timeout_ms": 60_000,
        "absolute_timeout_ms": 300_000, "max_open_transactions": 8}}),
        Some("ql-session"),
        None,
    );
    let (status, created) = post(&server, "/v1/sessions", &create, None);
    assert_eq!(status, 200, "{created}");
    let session_id = payload(&created)["session_id"].as_str().unwrap().to_owned();
    let token = payload(&created)["token"].as_str().unwrap().to_owned();
    let session = (session_id.as_str(), token.as_str());
    let original = program(
        "BEGIN; MUTATE $schema; MUTATE $first; MUTATE $second; MUTATE $edge; COMMIT;",
        json!({
            "schema": {"mutation": "put_schema", "registry": {"revision": 1,
                "migration": "public temporal graph proof", "tables": {
                    "document": {"model": "document", "mode": "schemaless"},
                    "linked": {"model": "graph_relation", "mode": "strict"}},
                "records": {}, "events": {}, "relations": {"linked": {
                    "from": ["document"], "to": ["document"], "properties": {},
                    "allow_additional_properties": true, "unique_pair": true}}}},
            "first": document("alpha", "Original"), "second": document("beta", "Evidence"),
            "edge": {"mutation": "put_relation", "reference": {"kind": "linked", "id": "evidence"},
                "from": {"kind": "document", "id": "alpha"}, "to": {"kind": "document", "id": "beta"},
                "valid_from": 100, "properties": {}}
        }),
        "ql-original",
    );
    let (status, denied) = post(&server, PATH, &original, None);
    assert_eq!(status, 401, "{denied}");
    let (status, committed) = post(&server, PATH, &original, Some(session));
    assert_eq!(status, 200, "{committed}");
    let old_cursor = payload(&committed)["commit"]["last_runtime_cursor"]
        .as_u64()
        .unwrap();
    assert_eq!(payload(&committed)["mutation_count"], 4);
    let correction = program(
        "BEGIN; MUTATE $correction; COMMIT;",
        json!({"correction": document("alpha", "Corrected")}),
        "ql-correction",
    );
    let (status, corrected) = post(&server, PATH, &correction, Some(session));
    assert_eq!(status, 200, "{corrected}");
    let new_cursor = payload(&corrected)["commit"]["last_runtime_cursor"]
        .as_u64()
        .unwrap();
    assert!(new_cursor > old_cursor);
    let canceled = program(
        "BEGIN; MUTATE $correction; CANCEL;",
        json!({"correction": document("alpha", "Must not publish")}),
        "ql-cancel",
    );
    let (status, result) = post(&server, PATH, &canceled, Some(session));
    assert_eq!(status, 200, "{result}");
    assert_eq!(payload(&result)["state"], "aborted");
    assert!(payload(&result).get("commit").is_none());
    let missing = program(
        "BEGIN; MUTATE $missing; COMMIT;",
        json!({"unused": document("alpha", "Must not publish")}),
        "ql-missing",
    );
    assert_eq!(post(&server, PATH, &missing, Some(session)).0, 400);
    let mut different = correction.clone();
    different["payload"]["mutation_bindings"]["correction"] =
        document("alpha", "Conflicting retry");
    assert_eq!(post(&server, PATH, &different, Some(session)).0, 409);

    let old = read(&server, session, old_cursor, 100);
    let new = read(&server, session, new_cursor, 100);
    assert_eq!(old["rows"][0]["values"]["title"]["value"], "Original");
    assert_eq!(new["rows"][0]["values"]["title"]["value"], "Corrected");
    let head = read(&server, session, "HEAD", 100);
    assert_eq!(head["known_at_cursor"], new_cursor);
    assert_eq!(head["rows"], new["rows"]);
    assert_eq!(read(&server, session, new_cursor, 99)["rows"], json!([]));
    server.stop();
    let reopened = start(&root);
    assert_eq!(
        read(&reopened, session, old_cursor, 100)["rows"],
        old["rows"]
    );
    assert_eq!(
        read(&reopened, session, new_cursor, 100)["rows"],
        new["rows"]
    );
    let (status, replay) = post(&reopened, PATH, &correction, Some(session));
    assert_eq!(status, 200, "{replay}");
    assert_eq!(payload(&replay)["commit"]["idempotent_replay"], true);
    assert_eq!(
        payload(&replay)["commit"]["last_runtime_cursor"],
        new_cursor
    );
    let head = read(&reopened, session, "HEAD", 100);
    assert_eq!(head["known_at_cursor"], new_cursor);
    assert_eq!(head["rows"], new["rows"]);
}
