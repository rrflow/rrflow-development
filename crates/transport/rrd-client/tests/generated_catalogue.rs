#[path = "../build_support.rs"]
mod build_support;

#[test]
fn generated_catalogue_is_deterministic_complete_and_fails_closed() {
    let catalogue = rrd_contract::endpoint_catalogue();
    let first = build_support::render(&catalogue).unwrap();
    assert_eq!(first, build_support::render(&catalogue).unwrap());
    let mut reordered = catalogue.clone();
    reordered.endpoints.reverse();
    assert!(build_support::render(&reordered).is_err());
    let generated_ids = first
        .lines()
        .filter(|line| line.trim_start().starts_with("const ID:"))
        .count();
    assert_eq!(generated_ids, catalogue.endpoints.len());
    for endpoint in &catalogue.endpoints {
        let expected = format!(
            "const ID: &'static str = {:?};",
            endpoint.operation.as_str()
        );
        assert_eq!(
            first.matches(&expected).count(),
            1,
            "{}",
            endpoint.operation
        );
    }
    let mut invalid = catalogue.clone();
    invalid.endpoints[0].request_type = "Bad<Type>".into();
    assert!(build_support::render(&invalid).is_err());
    let mut duplicate = catalogue.clone();
    duplicate.endpoints.push(duplicate.endpoints[0].clone());
    assert!(build_support::render(&duplicate).is_err());
}
