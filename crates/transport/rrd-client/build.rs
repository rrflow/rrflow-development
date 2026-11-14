mod build_support;

fn main() {
    let catalogue = rrd_contract::endpoint_catalogue();
    let projection =
        build_support::render(&catalogue).expect("valid generated Rust operation bindings");
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR"));
    std::fs::write(output.join("operation_bindings.rs"), projection)
        .expect("write generated operation bindings");
    println!("cargo:rerun-if-changed=build_support.rs");
}
