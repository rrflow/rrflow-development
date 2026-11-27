use std::io::{self, Write};

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !args.is_empty() && args != ["--catalogue"] {
        eprintln!("usage: rrd-contract-export [--catalogue]");
        std::process::exit(2);
    }
    let exported = if args.is_empty() {
        rrd_contract::openapi_document()
    } else {
        let catalogue = rrd_contract::endpoint_catalogue();
        catalogue.validate().and_then(|()| {
            serde_json::to_value(catalogue)
                .map_err(|error| rrd_contract::ContractError(error.to_string()))
        })
    };
    let document = match exported {
        Ok(document) => document,
        Err(error) => {
            eprintln!("RRD contract export failed: {error}");
            std::process::exit(1);
        }
    };
    let mut encoded = match serde_json::to_vec_pretty(&document) {
        Ok(encoded) => encoded,
        Err(error) => {
            eprintln!("RRD contract encoding failed: {error}");
            std::process::exit(1);
        }
    };
    encoded.push(b'\n');
    if let Err(error) = io::stdout().lock().write_all(&encoded) {
        eprintln!("RRD contract write failed: {error}");
        std::process::exit(1);
    }
}
