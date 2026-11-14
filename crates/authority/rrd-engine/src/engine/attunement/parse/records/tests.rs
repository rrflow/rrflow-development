use super::*;

const SOURCE: &str = "fn alpha() {}\n\nfn beta() {\n    alpha();\n}\n";

fn parsed(source: &str) -> (FileState, rrd_attunement::facts::SourceFacts) {
    let digest = digest::sha256_hex(source.as_bytes());
    let limits = rrd_attunement::parse::ParseLimits {
        input_bytes: 1 << 20,
        nodes: 1 << 20,
        depth: 256,
        output_bytes: 1 << 20,
        elapsed_micros: 5_000_000,
    };
    let facts = rrd_attunement::facts::source_facts(
        "src/lib.rs",
        &digest,
        "rust",
        source.as_bytes(),
        &limits,
        1_500,
        || false,
    )
    .unwrap();
    let state = FileState {
        path: "src/lib.rs".into(),
        content_sha256: digest,
        language: "rust".into(),
        parser_revision: facts.parser_revision.clone(),
        chunk_bytes: 1_500,
        job_id: "job".into(),
        status: Status::Lost,
        reason: String::new(),
        facts_sha256: String::new(),
        symbols: 0,
        spans: 0,
        chunks: 0,
        nodes: 0,
        error_nodes: 0,
    };
    (state, facts)
}

fn written<'a>(mutations: &'a [RuntimeMutation], kind: &str) -> Vec<&'a RuntimeRecord> {
    mutations
        .iter()
        .filter_map(|mutation| match mutation {
            RuntimeMutation::Record { record } if record.reference.kind.as_str() == kind => {
                Some(record)
            }
            _ => None,
        })
        .collect()
}

fn retired(mutations: &[RuntimeMutation]) -> usize {
    mutations
        .iter()
        .filter(|mutation| matches!(mutation, RuntimeMutation::Retire { .. }))
        .count()
}

fn text(record: &RuntimeRecord, key: &str) -> String {
    match record.properties.get(key) {
        Some(RuntimeValue::String(value)) => value.clone(),
        other => panic!("{key} is {other:?}"),
    }
}

#[test]
fn a_first_parse_writes_facts_sliced_from_the_verified_source() {
    let (state, facts) = parsed(SOURCE);
    let mutations =
        file_mutations(None, state, Outcome::Parsed(&facts), SOURCE.as_bytes(), 9).unwrap();
    let names: BTreeSet<_> = written(&mutations, SYMBOL)
        .into_iter()
        .map(|record| (text(record, "name"), text(record, "role")))
        .collect();
    assert!(names.contains(&("alpha".into(), "definition".into())));
    assert!(names.contains(&("beta".into(), "definition".into())));
    assert!(names.contains(&("alpha".into(), "reference".into())));
    let chunks = written(&mutations, CHUNK);
    assert_eq!(text(chunks[0], "text").trim_end(), SOURCE.trim_end());
    let states = written(&mutations, STATE);
    let state = state_from(states[0]).unwrap();
    assert_eq!(state.status, Status::Parsed);
    assert_eq!(state.symbols as usize, facts.symbols.len());
    assert_eq!(state.chunks as usize, chunks.len());
    assert_eq!(retired(&mutations), 0);
}

#[test]
fn an_edit_retires_the_facts_of_the_old_content() {
    let (old, old_facts) = parsed(SOURCE);
    let first =
        file_mutations(None, old, Outcome::Parsed(&old_facts), SOURCE.as_bytes(), 9).unwrap();
    let previous = state_from(written(&first, STATE)[0]).unwrap();
    let edited = "fn gamma() {}\n";
    let (next, facts) = parsed(edited);
    let mutations = file_mutations(
        Some(&previous),
        next,
        Outcome::Parsed(&facts),
        edited.as_bytes(),
        10,
    )
    .unwrap();
    let before = (previous.symbols + previous.spans + previous.chunks) as usize;
    assert_eq!(retired(&mutations), before, "every old fact is retired");
    assert_eq!(text(written(&mutations, SYMBOL)[0], "name"), "gamma");
}

#[test]
fn a_refusal_retires_old_facts_and_records_why() {
    let (old, facts) = parsed(SOURCE);
    let first = file_mutations(None, old, Outcome::Parsed(&facts), SOURCE.as_bytes(), 9).unwrap();
    let previous = state_from(written(&first, STATE)[0]).unwrap();
    let mutations = file_mutations(
        Some(&previous),
        previous.clone(),
        Outcome::Rejected("limit_exceeded".into()),
        SOURCE.as_bytes(),
        10,
    )
    .unwrap();
    let state = state_from(written(&mutations, STATE)[0]).unwrap();
    assert_eq!(
        (state.status, state.reason.as_str()),
        (Status::Rejected, "limit_exceeded")
    );
    assert_eq!((state.symbols, state.spans, state.chunks), (0, 0, 0));
    assert_eq!(
        retired(&mutations),
        (previous.symbols + previous.spans + previous.chunks) as usize
    );
}

#[test]
fn facts_that_do_not_match_the_verified_bytes_are_refused() {
    let (state, facts) = parsed(SOURCE);
    let other = SOURCE.replace("alpha", "omega");
    assert!(file_mutations(
        None,
        state.clone(),
        Outcome::Parsed(&facts),
        other.as_bytes(),
        9
    )
    .is_err());
    let mut outside = facts.clone();
    outside.symbols[0].name_end = SOURCE.len() as u64 + 1;
    assert!(file_mutations(None, state, Outcome::Parsed(&outside), SOURCE.as_bytes(), 9).is_err());
}

#[test]
fn a_removed_file_retires_its_facts_and_state() {
    let (state, facts) = parsed(SOURCE);
    let first = file_mutations(None, state, Outcome::Parsed(&facts), SOURCE.as_bytes(), 9).unwrap();
    let written = state_from(written(&first, STATE)[0]).unwrap();
    let removed = retire_file(&written, 10).unwrap();
    let facts = (written.symbols + written.spans + written.chunks) as usize;
    assert_eq!(removed.len(), facts + 1, "every fact and the state");
    assert_eq!(retired(&removed), removed.len());
}

#[test]
fn states_are_found_and_summarized_by_their_answer() {
    let (state, facts) = parsed(SOURCE);
    let mutations =
        file_mutations(None, state, Outcome::Parsed(&facts), SOURCE.as_bytes(), 9).unwrap();
    let found: Vec<_> = mutations
        .iter()
        .filter_map(|mutation| state_of(mutation).unwrap())
        .collect();
    assert_eq!(found.len(), 1, "only the state record yields a state");
    let mut states = BTreeMap::from([(found[0].path.clone(), found[0].clone())]);
    let parsed_summary = summary_sha256(&states).unwrap();
    states.get_mut("src/lib.rs").unwrap().status = Status::Lost;
    assert_ne!(parsed_summary, summary_sha256(&states).unwrap());
}

#[test]
fn the_parsed_source_tables_are_registered_once() {
    let mut registry = RuntimeSchemaRegistry::empty(1, "parsed source");
    assert!(register(&mut registry).unwrap());
    assert!(
        !register(&mut registry).unwrap(),
        "a second registration changes nothing"
    );
    for kind in [STATE, SYMBOL, SPAN, CHUNK] {
        assert!(registry
            .records
            .contains_key(&RuntimeType::new(kind).unwrap()));
    }
}

#[test]
fn a_fact_identity_changes_with_path_content_parser_chunk_size_and_index() {
    let base = fact_id("a.rs", "c1", "r1", 1_500, 0);
    assert_eq!(base, fact_id("a.rs", "c1", "r1", 1_500, 0));
    for other in [
        fact_id("b.rs", "c1", "r1", 1_500, 0),
        fact_id("a.rs", "c2", "r1", 1_500, 0),
        fact_id("a.rs", "c1", "r2", 1_500, 0),
        fact_id("a.rs", "c1", "r1", 1_000, 0),
        fact_id("a.rs", "c1", "r1", 1_500, 1),
    ] {
        assert_ne!(base, other);
    }
}

#[test]
fn a_file_state_reads_back_from_its_record() {
    let state = FileState {
        path: "src/lib.rs".into(),
        content_sha256: "a".repeat(64),
        language: "rust".into(),
        parser_revision: "tree-sitter-rust".into(),
        chunk_bytes: 1_500,
        job_id: "job".into(),
        status: Status::Rejected,
        reason: "file_too_large".into(),
        facts_sha256: "0".repeat(64),
        symbols: 3,
        spans: 2,
        chunks: 1,
        nodes: 40,
        error_nodes: 0,
    };
    let RuntimeMutation::Record { record } = state_record(&state, 7).unwrap() else {
        panic!("a state is written as a record");
    };
    assert_eq!(record.reference.kind.as_str(), STATE);
    assert_eq!(state_from(&record).unwrap(), state);
}
