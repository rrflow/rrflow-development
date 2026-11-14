use super::*;

#[test]
fn select_joins_relationships_at_one_stamp_and_survives_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("database");
    let scope = ScopeId::new("instance:test").unwrap();
    let store = RrflowKvStore::open(&path).unwrap();
    let committed = store.runtime().commit(&fixture_commit()).unwrap();
    let text = "SELECT left.id, right.to_id FROM record:document JOIN relation:depends_on ON id = from_id AT VALID 100 KNOWN HEAD WHERE left.status = $status";
    let parameters = Parameters::from([("status".into(), value("open"))]);
    let run = |store: &RrflowKvStore| {
        let catalog = fixture_catalog(store);
        let bound = bind(&parse(text).unwrap(), &parameters, &catalog).unwrap();
        let result = execute(store, &plan(&bound).unwrap(), &ExecutionBudget::default()).unwrap();
        assert_eq!(result.known_at_cursor, committed.last_cursor);
        let rows = flattened_rows(&result);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].values["left.id"], value("a"));
        assert_eq!(rows[0].values["right.to_id"], value("b"));
        rows
    };
    let initial = run(&store);
    drop(store);
    let reopened = RrflowKvStore::open(&path).unwrap();
    assert_eq!(run(&reopened), initial);
    assert_eq!(
        reopened.runtime().read_stamp(&scope).unwrap().commit_cursor,
        committed.last_cursor
    );
}
