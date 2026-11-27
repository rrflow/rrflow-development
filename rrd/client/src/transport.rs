//! Bounded HTTP carriage and current transport-attempt execution.

use crate::error::decode;
use crate::retry::request_timeout;
use crate::{Error, Result, RrdClient};
use bytes::{BufMut, Bytes, BytesMut};
use http_body_util::{BodyExt, Full};
use hyper::header::CONTENT_TYPE;
use hyper::{Method, Request, Uri};
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use rrd_contract::{RequestContext, ResponseEnvelope, ResponseOutcome, PROTOCOL, PROTOCOL_VERSION};
use serde::de::DeserializeOwned;
use serde::Deserialize;

pub const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone)]
pub(crate) enum Transport {
    Local(Client<HttpConnector, Full<Bytes>>),
    MutualTls(Client<HttpsConnector<HttpConnector>, Full<Bytes>>),
}

#[derive(Deserialize)]
struct ResponseProtocol {
    protocol: String,
    protocol_version: u16,
}

impl RrdClient {
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn send_raw<O: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Vec<u8>,
        headers: &[(&str, &str)],
        retry_safe: bool,
        deadline_unix_ms: Option<u64>,
    ) -> Result<ResponseEnvelope<O>> {
        let attempts = if retry_safe {
            self.config.max_attempts
        } else {
            1
        };
        let mut last_error = None;
        for _ in 0..attempts {
            let remaining = request_timeout(self.config.request_timeout, deadline_unix_ms)?;
            match tokio::time::timeout(
                remaining,
                self.send_once(method.clone(), path, body.clone(), headers, None),
            )
            .await
            {
                Ok(Ok(response)) => return Ok(response),
                Ok(Err(error @ Error::Transport(_))) => last_error = Some(error),
                Ok(Err(error)) => return Err(error),
                Err(_) => last_error = Some(Error::Timeout),
            }
        }
        Err(last_error.unwrap_or(Error::Timeout))
    }

    /// Catalogue-bound effects get exactly one send. A lost response is uncertain,
    /// irrespective of the legacy transport's configured retry count.
    pub(crate) async fn send_operation<O: DeserializeOwned>(
        &self,
        path: &str,
        body: Vec<u8>,
        headers: &[(&str, &str)],
        expected: &RequestContext,
    ) -> Result<ResponseEnvelope<O>> {
        let remaining = request_timeout(self.config.request_timeout, expected.deadline_unix_ms)?;
        tokio::time::timeout(
            remaining,
            self.send_once(Method::POST, path, body, headers, Some(expected)),
        )
        .await
        .map_err(|_| Error::Timeout)?
    }

    async fn send_once<O: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Vec<u8>,
        headers: &[(&str, &str)],
        expected: Option<&RequestContext>,
    ) -> Result<ResponseEnvelope<O>> {
        let uri: Uri = format!("{}{path}", self.endpoint)
            .parse()
            .map_err(|error| Error::Contract(format!("invalid request URI: {error}")))?;
        let mut builder = Request::builder().method(method.clone()).uri(uri);
        if method != Method::GET {
            builder = builder.header(CONTENT_TYPE, "application/json");
        }
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        let request = builder
            .body(Full::new(Bytes::from(body)))
            .map_err(|error| Error::Contract(error.to_string()))?;
        let response = match &self.transport {
            Transport::Local(transport) => transport.request(request).await,
            Transport::MutualTls(transport) => transport.request(request).await,
        }
        .map_err(|error| Error::Transport(error.to_string()))?;
        let status = response.status();
        if expected.is_some() {
            let mut media = response.headers().get_all(CONTENT_TYPE).iter();
            let json = media
                .next()
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| {
                    value
                        .split(';')
                        .next()
                        .unwrap_or_default()
                        .trim()
                        .eq_ignore_ascii_case("application/json")
                });
            if !json || media.next().is_some() {
                return Err(Error::Decode(
                    "operation response requires one application/json content type".into(),
                ));
            }
            if status.is_success() && status != hyper::StatusCode::OK {
                return Err(Error::Decode("operation success requires HTTP 200".into()));
            }
        }
        let mut incoming = response.into_body();
        let mut bytes = BytesMut::new();
        while let Some(frame) = incoming.frame().await {
            let frame = frame.map_err(|error| Error::Transport(error.to_string()))?;
            if let Some(data) = frame.data_ref() {
                if bytes.len().saturating_add(data.len()) > MAX_RESPONSE_BYTES {
                    return Err(Error::ResponseTooLarge);
                }
                bytes.put_slice(data);
            }
        }
        let response_protocol: ResponseProtocol = serde_json::from_slice(&bytes).map_err(decode)?;
        if response_protocol.protocol != PROTOCOL
            || response_protocol.protocol_version != PROTOCOL_VERSION
        {
            return Err(Error::UnsupportedProtocol {
                protocol: response_protocol.protocol,
                version: response_protocol.protocol_version,
            });
        }
        let decoded: ResponseEnvelope<O> = serde_json::from_slice(&bytes).map_err(decode)?;
        if expected.is_some_and(|context| {
            decoded.request_id != context.request_id || decoded.operation_id != context.operation_id
        }) {
            return Err(Error::ResponseIdentityMismatch);
        }
        if status.is_success() != matches!(decoded.outcome, ResponseOutcome::Ok { .. }) {
            return Err(Error::Decode(
                "HTTP status and response outcome disagree".into(),
            ));
        }
        if let ResponseOutcome::Error { ref error } = decoded.outcome {
            return Err(Error::Api {
                status,
                error: error.clone(),
            });
        }
        Ok(decoded)
    }
}
