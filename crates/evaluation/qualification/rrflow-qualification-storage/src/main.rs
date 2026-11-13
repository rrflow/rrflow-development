//! The storage acceptance command: schemas, validation, verification,
//! campaigns and CI receipts for RRFlow's storage release gate.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = rrflow_qualification_storage::cli::run(&args) {
        eprintln!("rrflow-qualification-storage: {error}");
        std::process::exit(1);
    }
}
