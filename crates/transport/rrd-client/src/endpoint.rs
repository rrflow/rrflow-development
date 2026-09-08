//! Explicit loopback or mutual-TLS endpoint construction.

use crate::retry::validate_client_config;
use crate::transport::Transport;
use crate::{ClientConfig, Error, Result, RrdClient};
use bytes::Bytes;
use http_body_util::Full;
use hyper::Uri;
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use rrd_contract::CanonicalId;
use rustls::ClientConfig as RustlsClientConfig;
use std::net::SocketAddr;
use std::sync::Arc;

impl RrdClient {
    pub fn instance_id(&self) -> &CanonicalId {
        &self.instance
    }

    pub fn connect_local(
        address: SocketAddr,
        instance: CanonicalId,
        config: ClientConfig,
    ) -> Result<Self> {
        if !address.ip().is_loopback() {
            return Err(Error::Contract(
                "RRD Rust client permits only loopback HTTP before TLS qualification".into(),
            ));
        }
        validate_client_config(&config)?;
        let connector = HttpConnector::new();
        let transport: Client<_, Full<Bytes>> =
            Client::builder(TokioExecutor::new()).build(connector);
        Ok(Self {
            transport: Transport::Local(transport),
            endpoint: format!("http://{address}"),
            instance,
            config,
            websocket_tls: None,
        })
    }

    pub fn connect_mtls(
        endpoint: impl Into<String>,
        instance: CanonicalId,
        tls: RustlsClientConfig,
        config: ClientConfig,
    ) -> Result<Self> {
        validate_client_config(&config)?;
        let endpoint = endpoint.into();
        let parsed: Uri = endpoint
            .parse()
            .map_err(|error| Error::Contract(format!("invalid RRD TLS endpoint: {error}")))?;
        if parsed.scheme_str() != Some("https")
            || parsed.authority().is_none()
            || parsed
                .path_and_query()
                .is_some_and(|value| value.as_str() != "/")
        {
            return Err(Error::Contract(
                "RRD TLS endpoint must be an https origin without a path or query".into(),
            ));
        }
        let websocket_tls = Some(Arc::new(tls.clone()));
        let connector = HttpsConnectorBuilder::new()
            .with_tls_config(tls)
            .https_only()
            .enable_http1()
            .build();
        let transport = Client::builder(TokioExecutor::new()).build(connector);
        Ok(Self {
            transport: Transport::MutualTls(transport),
            endpoint: endpoint.trim_end_matches('/').into(),
            instance,
            config,
            websocket_tls,
        })
    }
}
