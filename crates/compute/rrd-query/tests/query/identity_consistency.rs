use super::*;

#[test]
fn series_identity_uses_sample_table_and_id_through_reopen() {
    let root = tempfile::tempdir().unwrap();
    let store = RrflowKvStore::open(root.path()).unwrap();
    let mut fixture = fixture_commit();
    let RuntimeMutation::Schema { registry } = &mut fixture.mutations[0] else {
        panic!("fixture schema")
    };
    registry.tables.insert(
        RuntimeType::new("other-sample").unwrap(),
        RuntimeTableSchema::schemaless(RuntimeLogicalModel::TimeSeries),
    );
    let original = fixture
        .mutations
        .iter()
        .find_map(|mutation| match mutation {
            RuntimeMutation::SeriesSample { sample } => Some(sample.clone()),
            _ => None,
        })
        .unwrap();
    let mut second = original;
    second.reference = RuntimeRef::new("other-sample", "latency-100").unwrap();
    fixture
        .mutations
        .push(RuntimeMutation::SeriesSample { sample: second });
    store.runtime().commit(&fixture).unwrap();
    let checked_query = parse("FROM series:metric AT VALID 100 KNOWN HEAD PROJECT id").unwrap();
    let mut checked_plan =
        plan(&bind(&checked_query, &Parameters::new(), &fixture_catalog(&store)).unwrap()).unwrap();
    checked_plan.logical.result_identity_version = 1;
    assert!(matches!(checked_plan.verify(), Err(Error::Integrity(_))));
    let run = |store: &RrflowKvStore| {
        let query = parse("FROM series:metric AT VALID 100 KNOWN HEAD WHERE series_id = \"latency\" PROJECT id, kind").unwrap();
        let bound = bind(&query, &Parameters::new(), &fixture_catalog(store)).unwrap();
        let result = execute(store, &plan(&bound).unwrap(), &ExecutionBudget::default()).unwrap();
        result
            .batches
            .into_iter()
            .flat_map(|batch| batch.rows)
            .map(|row| row.identity)
            .collect::<BTreeSet<_>>()
    };
    let expected = BTreeSet::from([
        "series:other-sample:latency-100".to_string(),
        "series:sample:latency-100".to_string(),
    ]);
    assert_eq!(run(&store), expected);
    drop(store);
    let store = RrflowKvStore::open(root.path()).unwrap();
    assert_eq!(run(&store), expected);
}
