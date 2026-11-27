fn main() {
    let document =
        serde_json::to_string_pretty(&rrflow_cx_operator::crd_document()).expect("CRD serializes");
    println!("{document}");
}
