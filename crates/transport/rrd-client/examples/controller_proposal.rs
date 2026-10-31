//! One installed-estate proposal through the generated Rust client binding.
//! Locator/credential bootstrap follows rrflowdb; all operations use public HTTP.
use rrd_client::{
    operations::TaskEndpointPropose, ClientConfig, OperationMetadata, RequestOptions, RrdClient,
};
use rrd_contract::{CloseSession, CreateSession, EndpointSelectionRequest, SessionLimits};
use rrd_engine::RrdEngine;
use std::{
    io::Read,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err(
            "usage: controller_proposal PROJECT LOOPBACK_ADDRESS REQUEST_JSON TIMEOUT_MS".into(),
        );
    }
    let timeout_ms: u64 = args[3].parse()?;
    let limits = SessionLimits {
        idle_timeout_ms: timeout_ms,
        absolute_timeout_ms: timeout_ms.checked_mul(2).ok_or("timeout overflow")?,
        max_open_transactions: 1,
    };
    limits.validate()?;
    let mut bytes = Vec::new();
    std::fs::File::open(&args[2])?
        .take(rrd_contract::MAX_ROUTE_REQUEST_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if u64::try_from(bytes.len())? > rrd_contract::MAX_ROUTE_REQUEST_BYTES {
        return Err("proposal request file exceeds the contract byte limit".into());
    }
    let request: EndpointSelectionRequest = serde_json::from_slice(&bytes)?;
    request.validate()?;
    let project = Path::new(&args[0]);
    let locator = RrdEngine::read_project_locator(project)?;
    let credential = RrdEngine::read_installed_api_key(project)?;
    let client = RrdClient::connect_local(
        args[1].parse()?,
        locator.identity.instance_id,
        ClientConfig {
            request_timeout: Duration::from_millis(timeout_ms),
            max_attempts: 1,
            ..ClientConfig::default()
        },
    )?;
    client.capabilities().await?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let key = format!(
        "controller-native-{}-{}",
        std::process::id(),
        now.as_nanos()
    );
    let session = client
        .create_session(
            credential.principal_id.clone(),
            credential.credential(),
            CreateSession { limits },
            RequestOptions::mutation(&key, &key, &key)?,
        )
        .await?;
    let at = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
    let options = RequestOptions::new(
        &key,
        &key,
        Some(&key),
        Some(at.checked_add(timeout_ms).ok_or("deadline overflow")?),
    )?;
    let result = client
        .call_operation::<TaskEndpointPropose>(&session, request.clone(), options)
        .await;
    let close_key = format!("{key}-close");
    let closed = client
        .close_session(
            &session,
            CloseSession {},
            RequestOptions::mutation(&close_key, &close_key, &close_key)?,
        )
        .await;
    let close_error = closed.err().map(|error| error.to_string());
    let failed = result.is_err() || close_error.is_some();
    let outcome = match result {
        Ok(proposal) => serde_json::json!({"status": "ok", "proposal": proposal}),
        Err(error) => serde_json::json!({"status": "error", "error": error.to_string()}),
    };
    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({
            "operation": TaskEndpointPropose::OPERATION, "request": request,
            "outcome": outcome, "sessionCloseError": close_error,
        }))?
    );
    if failed {
        std::process::exit(1);
    }
    Ok(())
}
