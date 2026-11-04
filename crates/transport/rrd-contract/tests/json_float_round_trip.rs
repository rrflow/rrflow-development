//! A byte count or digest over a float can only be checked if the float survives the wire.

#[test]
fn every_float_keeps_its_exact_value_through_json() {
    let mut state = 88_172_645_463_325_252_u64;
    for _ in 0..200_000 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        // The range BM25 scores and rank contributions fall in.
        let value = (state >> 11) as f64 / (1_u64 << 53) as f64 * 40.0;
        let text = serde_json::to_string(&value).unwrap();
        let back: f64 = serde_json::from_str(&text).unwrap();
        assert_eq!(
            back.to_bits(),
            value.to_bits(),
            "{text} came back as {back}"
        );
    }
}

#[test]
fn a_float_inside_a_decoded_value_keeps_its_exact_value_too() {
    let value = 15.269_460_205_859_353_f64;
    let text = serde_json::json!({ "score": value }).to_string();
    let decoded: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        decoded["score"].as_f64().unwrap().to_bits(),
        value.to_bits()
    );
}
