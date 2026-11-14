//! Offline verification of actual captured configuration and model output.
//! This command makes no network call and grants no execution authorization.
use rrd_contract::{
    CorrelationId, EndpointConfiguration, EndpointSelectionAllowance, EndpointSelectionChoice,
    EndpointSelectionRequest, EndpointSelectionTask, TaskRecord, MAX_ENDPOINT_CONFIGURATION_BYTES,
    MAX_ROUTE_REQUEST_BYTES, MAX_ROUTE_RESPONSE_BYTES,
};
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::io::Read;

fn read_bounded(path: &str, maximum: u64) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(maximum + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > maximum {
        return Err("input exceeds contract byte ceiling".into());
    }
    Ok(bytes)
}

fn read_json<T: DeserializeOwned>(
    path: &str,
    maximum: u64,
) -> Result<T, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(&read_bounded(path, maximum)?)?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.as_slice() {
        [command] if command == "schema" => {
            println!("{}", serde_json::to_string(&schemars::schema_for!(EndpointSelectionChoice))?);
        }
        [command, path] if command == "configuration-sha256" => {
            let configuration: EndpointConfiguration = read_json(path, MAX_ENDPOINT_CONFIGURATION_BYTES as u64)?;
            println!("{}", configuration.sha256()?);
        }
        [command, task_path, configuration_path, attempt_id] if command == "proposal-request" => {
            let task: TaskRecord = read_json(task_path, MAX_ROUTE_REQUEST_BYTES)?;
            let configuration: EndpointConfiguration = read_json(configuration_path, MAX_ENDPOINT_CONFIGURATION_BYTES as u64)?;
            let projected = EndpointSelectionTask::from_task(&task)?;
            let request = EndpointSelectionRequest {
                scope: task.scope, task_id: task.task_id,
                attempt_id: CorrelationId::new(attempt_id)?,
                max_storage_keys: rrd_contract::MAX_TASK_STORAGE_KEYS,
                expected_task_sha256: projected.task_sha256,
                expected_configuration_sha256: configuration.sha256()?,
            };
            request.validate()?;
            println!("{}", serde_json::to_string(&request)?);
        }
        [command, configuration_path, allowance_path, response_path] if command == "verify" => {
            let configuration: EndpointConfiguration = read_json(configuration_path, MAX_ENDPOINT_CONFIGURATION_BYTES as u64)?;
            let allowance: EndpointSelectionAllowance = read_json(allowance_path, MAX_ROUTE_REQUEST_BYTES)?;
            let response = read_bounded(response_path, MAX_ROUTE_RESPONSE_BYTES)?;
            let choice = EndpointSelectionChoice::decode(&response, &allowance, &configuration)?;
            println!("{}", serde_json::json!({
                "choice": choice,
                "allowance_sha256": allowance.sha256(&configuration)?,
                "raw_response_sha256": Sha256::digest(&response).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
                "configuration_sha256": configuration.sha256()?,
                "external_effect_performed": false,
                "authorization_granted": false
            }));
        }
        _ => return Err("usage: endpoint_selection schema | configuration-sha256 CONFIGURATION | proposal-request TASK CONFIGURATION ATTEMPT_ID | verify CONFIGURATION ALLOWANCE RESPONSE".into()),
    }
    Ok(())
}
