//! Operator-only real transport probe, not an engine-authorized task runner.
#[cfg(feature = "endpoint-chat")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rrd_contract::{
        CanonicalId, EndpointConfiguration, EndpointSelectionAllowance,
        MAX_ENDPOINT_CONFIGURATION_BYTES, MAX_ROUTE_REQUEST_BYTES,
    };
    use rrd_inference::{
        endpoint_chat::{EndpointChatRequest, PreparedEndpointChat},
        EmbeddingCancellation, EmbeddingExecutionControl, NetworkPolicy,
    };
    use std::{
        io::Read,
        path::Path,
        time::{Duration, Instant},
    };
    fn read<T: serde::de::DeserializeOwned>(
        path: &str,
        limit: u64,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(limit + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > limit {
            return Err("input exceeds byte ceiling".into());
        }
        Ok(serde_json::from_slice(&bytes)?)
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 7 && args.len() != 8 {
        return Err("usage: endpoint_chat CONFIG ALLOWANCE CONTROLLER_ID REQUEST KEY_ENV OUTPUT_DIR TIMEOUT_MS [CANCEL_AFTER_MS]".into());
    }
    let configuration: EndpointConfiguration =
        read(&args[0], MAX_ENDPOINT_CONFIGURATION_BYTES as u64)?;
    let allowance: EndpointSelectionAllowance = read(&args[1], MAX_ROUTE_REQUEST_BYTES)?;
    let binding = CanonicalId::new(&args[2])?;
    let request: EndpointChatRequest = read(&args[3], MAX_ROUTE_REQUEST_BYTES)?;
    let prepared = PreparedEndpointChat::new(&configuration, &binding, &allowance, &request)?;
    let duration = args[6].parse::<u64>()?;
    if duration == 0 || duration > 3_600_000 {
        return Err("invalid probe timeout".into());
    }
    let cancellation = EmbeddingCancellation::default();
    let cancel_after = args.get(7).map(|s| s.parse::<u64>()).transpose()?;
    if cancel_after.is_some_and(|ms| ms > duration) {
        return Err("cancellation must fall within timeout".into());
    }
    let directory = Path::new(&args[5]);
    std::fs::create_dir(directory)?; // Fresh output only; never overwrite evidence.
    std::fs::write(directory.join("request.json"), prepared.request_bytes())?;
    let control = EmbeddingExecutionControl::new(
        Instant::now() + Duration::from_millis(duration),
        cancellation.clone(),
    );
    if let Some(ms) = cancel_after {
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(ms));
            cancellation.cancel();
        });
    }
    let result = prepared.execute(NetworkPolicy::Allow, &control, |_, _| {
        std::env::var(&args[4]).map_err(|_| "configured environment credential unavailable".into())
    });
    let envelope = match result {
        Ok(observation) => {
            std::fs::write(directory.join("response.json"), &observation.response_bytes)?;
            serde_json::json!({"scope":"operator_transport_observation", "engine_authorized":false,
                "worker_dispatched":false,"training_admitted":false,"observation":observation})
        }
        Err(failure) => {
            serde_json::json!({"scope":"operator_transport_observation", "engine_authorized":false,
            "worker_dispatched":false,"training_admitted":false,"failure":failure})
        }
    };
    std::fs::write(
        directory.join("receipt.json"),
        serde_json::to_vec_pretty(&envelope)?,
    )?;
    println!("{}", serde_json::to_string(&envelope)?);
    if envelope.get("failure").is_some() {
        return Err("transport probe did not produce an accepted choice".into());
    }
    Ok(())
}

#[cfg(not(feature = "endpoint-chat"))]
fn main() {
    eprintln!("enable the endpoint-chat feature");
    std::process::exit(2);
}
