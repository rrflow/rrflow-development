use super::query_error_class;

#[test]
fn graphql_errors_retain_their_canonical_trace_class() {
    let error = rrflow_ql::Error::Graphql("unsupported operation".into());

    assert_eq!(query_error_class(&error), "graphql");
}
