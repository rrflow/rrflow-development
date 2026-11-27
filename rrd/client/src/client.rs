//! Client identity plus capability and schema discovery.

use crate::operation::outcome;
use crate::transport::Transport;
use crate::{ClientConfig, Error, Result};
use hyper::{Method, StatusCode};
use rrd_contract::{
    CanonicalId, EndpointCatalogue, Readiness, ResponseEnvelope, ServiceCapabilities, PROTOCOL,
    PROTOCOL_VERSION,
};
use rustls::ClientConfig as RustlsClientConfig;
use std::sync::Arc;

#[derive(Clone)]
pub struct RrdClient {
    pub(crate) transport: Transport,
    pub(crate) endpoint: String,
    pub(crate) instance: CanonicalId,
    pub(crate) config: ClientConfig,
    pub(crate) websocket_tls: Option<Arc<RustlsClientConfig>>,
}

impl RrdClient {
    pub async fn readiness(&self) -> Result<Readiness> {
        let response: ResponseEnvelope<Readiness> = self
            .send_raw(Method::GET, "/v1/health/ready", Vec::new(), &[], true, None)
            .await?;
        let readiness = outcome(StatusCode::OK, response, None)?;
        readiness.validate().map_err(crate::error::contract)?;
        Ok(readiness)
    }

    pub async fn capabilities(&self) -> Result<ServiceCapabilities> {
        let response: ResponseEnvelope<ServiceCapabilities> = self
            .send_raw(Method::GET, "/v1/capabilities", Vec::new(), &[], true, None)
            .await?;
        let capabilities = outcome(StatusCode::OK, response, None)?;
        capabilities.validate().map_err(crate::error::contract)?;
        if capabilities.protocol != PROTOCOL || capabilities.protocol_version != PROTOCOL_VERSION {
            return Err(Error::UnsupportedProtocol {
                protocol: capabilities.protocol,
                version: capabilities.protocol_version,
            });
        }
        if capabilities.instance.id != self.instance {
            return Err(Error::ResponseIdentityMismatch);
        }
        Ok(capabilities)
    }

    pub async fn endpoint_catalogue(&self) -> Result<EndpointCatalogue> {
        let response: ResponseEnvelope<EndpointCatalogue> = self
            .send_raw(
                Method::GET,
                "/v1/schema/endpoints",
                Vec::new(),
                &[],
                true,
                None,
            )
            .await?;
        let catalogue = outcome(StatusCode::OK, response, None)?;
        catalogue.validate().map_err(crate::error::contract)?;
        Ok(catalogue)
    }

    pub async fn openapi_document(&self) -> Result<serde_json::Value> {
        let response: ResponseEnvelope<serde_json::Value> = self
            .send_raw(
                Method::GET,
                "/v1/schema/openapi",
                Vec::new(),
                &[],
                true,
                None,
            )
            .await?;
        let document = outcome(StatusCode::OK, response, None)?;
        if document["openapi"] != "3.1.0"
            || document["x-rrd-protocol"] != PROTOCOL
            || document["x-rrd-protocol-version"] != PROTOCOL_VERSION
        {
            return Err(Error::UnsupportedProtocol {
                protocol: document["x-rrd-protocol"]
                    .as_str()
                    .unwrap_or("missing")
                    .into(),
                version: document["x-rrd-protocol-version"]
                    .as_u64()
                    .and_then(|version| u16::try_from(version).ok())
                    .unwrap_or_default(),
            });
        }
        Ok(document)
    }
}
