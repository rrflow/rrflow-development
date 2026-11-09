use rrd_query::parse;

#[test]
fn select_relationship_join_uses_the_existing_relational_query_contract() {
    let select = parse("SELECT left.id, right.to_id FROM record:document JOIN relation:depends_on ON id = from_id AT VALID $when KNOWN HEAD WHERE left.status = $status LIMIT 10").unwrap();
    let established = parse("FROM record:document JOIN relation:depends_on ON id = from_id AT VALID $when KNOWN HEAD WHERE left.status = $status PROJECT left.id, right.to_id LIMIT 10").unwrap();
    assert_eq!(select, established);
    assert_eq!(select.canonical(), established.canonical());
}

#[test]
fn select_rejects_ambiguous_projection_and_unsupported_sql() {
    for input in [
        "SELECT FROM record:document AT VALID 1 KNOWN HEAD",
        "SELECT id, FROM record:document AT VALID 1 KNOWN HEAD",
        "SELECT * FROM record:document AT VALID 1 KNOWN HEAD PROJECT id",
        "SELECT id FROM record:document AT VALID 1 KNOWN HEAD GROUP BY id",
        "SELECT id FROM record:document AT VALID 1 KNOWN HEAD WHERE status =",
        "SELECT * FROM traverse:edge START node:a DIRECTION OUTGOING DEPTH 2 AT VALID 1 KNOWN HEAD",
    ] {
        assert!(parse(input).is_err(), "unexpectedly accepted: {input}");
    }
}
