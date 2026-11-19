//! Command-line arguments, mutual TLS and JWT key loading.

use super::*;

/// An argument the server cannot use, with the usage text where that helps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ArgumentError(String);

impl std::fmt::Display for ArgumentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ArgumentError {}

impl From<String> for ArgumentError {
    fn from(message: String) -> Self {
        Self(message)
    }
}

impl From<&str> for ArgumentError {
    fn from(message: &str) -> Self {
        Self(message.to_owned())
    }
}

pub(super) fn parse_args(arguments: impl Iterator<Item = String>) -> Result<Args, ArgumentError> {
    let mut project = None;
    let mut distribution_executable = None;
    let mut bind = "127.0.0.1:9477".parse().expect("static bind address");
    let mut jwt_key_file = None;
    let mut ready_file = None;
    let mut stop_poll_interval_ms = rrd_engine::DEFAULT_STOP_POLL_INTERVAL_MS;
    let mut instance_registry = None;
    let mut instance_registry_lock_wait_ms = None;
    let mut tls_certificate_file = None;
    let mut tls_private_key_file = None;
    let mut tls_client_ca_file = None;
    let mut arguments = arguments;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--project" => {
                project = Some(PathBuf::from(required_value(&mut arguments, "--project")?));
            }
            "--distribution-executable" => {
                distribution_executable = Some(PathBuf::from(required_value(
                    &mut arguments,
                    "--distribution-executable",
                )?));
            }
            "--bind" => {
                let value = required_value(&mut arguments, "--bind")?;
                bind = value
                    .parse()
                    .map_err(|error| format!("invalid --bind {value:?}: {error}"))?;
            }
            "--jwt-key-file" => {
                jwt_key_file = Some(PathBuf::from(required_value(
                    &mut arguments,
                    "--jwt-key-file",
                )?));
            }
            "--ready-file" => {
                ready_file = Some(PathBuf::from(required_value(
                    &mut arguments,
                    "--ready-file",
                )?));
            }
            "--stop-poll-interval-ms" => {
                stop_poll_interval_ms = milliseconds(&mut arguments, "--stop-poll-interval-ms")?;
                if stop_poll_interval_ms == 0 {
                    return Err("--stop-poll-interval-ms must be at least 1".into());
                }
            }
            "--instance-registry" => {
                instance_registry = Some(PathBuf::from(required_value(
                    &mut arguments,
                    "--instance-registry",
                )?));
            }
            "--instance-registry-lock-wait-ms" => {
                instance_registry_lock_wait_ms = Some(milliseconds(
                    &mut arguments,
                    "--instance-registry-lock-wait-ms",
                )?);
            }
            "--tls-cert" => {
                tls_certificate_file =
                    Some(PathBuf::from(required_value(&mut arguments, "--tls-cert")?));
            }
            "--tls-key" => {
                tls_private_key_file =
                    Some(PathBuf::from(required_value(&mut arguments, "--tls-key")?));
            }
            "--tls-client-ca" => {
                tls_client_ca_file = Some(PathBuf::from(required_value(
                    &mut arguments,
                    "--tls-client-ca",
                )?));
            }
            "--help" | "-h" => return Err(usage().into()),
            value => return Err(format!("unknown argument {value:?}\n{}", usage()).into()),
        }
    }
    let tls_file_count = [
        tls_certificate_file.is_some(),
        tls_private_key_file.is_some(),
        tls_client_ca_file.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    if tls_file_count != 0 && tls_file_count != 3 {
        return Err(format!(
            "--tls-cert, --tls-key, and --tls-client-ca must be provided together\n{}",
            usage()
        )
        .into());
    }
    if ready_file
        .as_ref()
        .is_some_and(|ready| !ready.is_absolute())
    {
        return Err("readiness file must be absolute".into());
    }
    Ok(Args {
        project: project.ok_or_else(|| format!("--project is required\n{}", usage()))?,
        distribution_executable: distribution_executable
            .ok_or_else(|| format!("--distribution-executable is required\n{}", usage()))?,
        bind,
        jwt_key_file,
        ready_file,
        stop_poll_interval: Duration::from_millis(stop_poll_interval_ms),
        instance_registry,
        instance_registry_lock_wait_ms,
        tls_certificate_file,
        tls_private_key_file,
        tls_client_ca_file,
    })
}

pub(super) fn load_mtls(
    certificate_file: Option<PathBuf>,
    private_key_file: Option<PathBuf>,
    client_ca_file: Option<PathBuf>,
) -> Result<Option<RrdMutualTlsServerConfig>, Box<dyn std::error::Error + Send + Sync>> {
    let (certificate_file, private_key_file, client_ca_file) =
        match (certificate_file, private_key_file, client_ca_file) {
            (None, None, None) => return Ok(None),
            (Some(certificate), Some(key), Some(ca)) => (certificate, key, ca),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "--tls-cert, --tls-key, and --tls-client-ca are required together",
                )
                .into());
            }
        };
    let certificate_chain =
        rustls_pemfile::certs(&mut BufReader::new(File::open(certificate_file)?))
            .collect::<std::result::Result<Vec<_>, _>>()?;
    let private_key =
        rustls_pemfile::private_key(&mut BufReader::new(File::open(private_key_file)?))?
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "TLS private key is absent")
            })?;
    let mut client_roots = RootCertStore::empty();
    for certificate in rustls_pemfile::certs(&mut BufReader::new(File::open(client_ca_file)?)) {
        client_roots
            .add(certificate?)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    }
    Ok(Some(RrdMutualTlsServerConfig::new(
        certificate_chain,
        private_key,
        client_roots,
    )?))
}

pub(super) fn load_jwt_key(
    path: PathBuf,
) -> Result<RrdJwtVerificationKey, Box<dyn std::error::Error + Send + Sync>> {
    let metadata = std::fs::metadata(&path)?;
    if !metadata.is_file() || metadata.len() > 64 * 1024 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "JWT key must be a bounded regular file",
        )
        .into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "JWT key file must be owner-only",
            )
            .into());
        }
    }
    RrdJwtVerificationKey::new(std::fs::read(path)?).map_err(Into::into)
}

pub(super) fn required_value(
    arguments: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<String, ArgumentError> {
    arguments
        .next()
        .ok_or_else(|| format!("{option} requires a value").into())
}

fn milliseconds(
    arguments: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<u64, ArgumentError> {
    let value = required_value(arguments, option)?;
    value
        .parse()
        .map_err(|error| format!("invalid {option} {value:?}: {error}").into())
}

pub(super) fn usage() -> &'static str {
    "usage: rrd-server --project PROJECT --distribution-executable RRFLOW [--bind 127.0.0.1:9477] [--jwt-key-file PATH] [--ready-file PATH] [--tls-cert PATH --tls-key PATH --tls-client-ca PATH] [--stop-poll-interval-ms MS] [--instance-registry DIR] [--instance-registry-lock-wait-ms MS]"
}

#[cfg(test)]
mod tests;
