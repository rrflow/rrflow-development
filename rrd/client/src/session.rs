//! Credential-bearing session state with redacted public diagnostics.

use crate::error::decode;
use crate::operation::{outcome, RequestOptions};
use crate::{Result, RrdClient};
use hyper::{Method, StatusCode};
use rrd_contract::{
    CanonicalId, CloseSession, CorrelationId, CreateSession, RenewSession, SessionLease,
    SessionLimits, SessionTermination,
};
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct Session {
    principal_id: CanonicalId,
    lease: SessionLease,
}

impl Session {
    fn new(principal_id: CanonicalId, lease: SessionLease) -> Self {
        Self {
            principal_id,
            lease,
        }
    }

    pub fn principal_id(&self) -> &CanonicalId {
        &self.principal_id
    }

    pub fn session_id(&self) -> &CorrelationId {
        &self.lease.session_id
    }

    pub fn issued_at_unix_ms(&self) -> u64 {
        self.lease.issued_at_unix_ms
    }

    pub fn idle_expires_at_unix_ms(&self) -> u64 {
        self.lease.idle_expires_at_unix_ms
    }

    pub fn absolute_expires_at_unix_ms(&self) -> u64 {
        self.lease.absolute_expires_at_unix_ms
    }

    pub fn limits(&self) -> &SessionLimits {
        &self.lease.limits
    }

    pub(crate) fn token(&self) -> &CorrelationId {
        &self.lease.token
    }
}

impl fmt::Debug for Session {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Session")
            .field("principal_id", &self.principal_id)
            .field("session_id", &self.lease.session_id)
            .field("issued_at_unix_ms", &self.lease.issued_at_unix_ms)
            .field(
                "idle_expires_at_unix_ms",
                &self.lease.idle_expires_at_unix_ms,
            )
            .field(
                "absolute_expires_at_unix_ms",
                &self.lease.absolute_expires_at_unix_ms,
            )
            .field("limits", &self.lease.limits)
            .field("credential", &"[REDACTED]")
            .finish()
    }
}

impl RrdClient {
    pub async fn create_session(
        &self,
        principal_id: CanonicalId,
        api_key: &str,
        request: CreateSession,
        options: RequestOptions,
    ) -> Result<Session> {
        let expected = options.context();
        let envelope = self.envelope(request, &options, true)?;
        let bytes = serde_json::to_vec(&envelope).map_err(decode)?;
        let authorization = format!("ApiKey {api_key}");
        let headers = [
            ("X-RRD-Principal", principal_id.as_str()),
            ("Authorization", authorization.as_str()),
        ];
        let response = self
            .send_raw(
                Method::POST,
                "/v1/sessions",
                bytes,
                &headers,
                true,
                expected.deadline_unix_ms,
            )
            .await?;
        let lease = outcome(StatusCode::OK, response, Some(&expected))?;
        Ok(Session::new(principal_id, lease))
    }

    pub async fn renew_session(
        &self,
        session: &Session,
        request: RenewSession,
        options: RequestOptions,
    ) -> Result<Session> {
        let lease = self
            .session_call(
                Method::POST,
                &format!("/v1/sessions/{}/renew", session.session_id().as_str()),
                session,
                request,
                options,
                true,
            )
            .await?;
        Ok(Session::new(session.principal_id.clone(), lease))
    }

    pub async fn close_session(
        &self,
        session: &Session,
        request: CloseSession,
        options: RequestOptions,
    ) -> Result<SessionTermination> {
        self.session_call(
            Method::DELETE,
            &format!("/v1/sessions/{}", session.session_id().as_str()),
            session,
            request,
            options,
            true,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_debug_redacts_the_bearer_credential() {
        let session = Session::new(
            CanonicalId::new("principal").unwrap(),
            SessionLease {
                session_id: CorrelationId::new("session").unwrap(),
                token: CorrelationId::new("secret-bearer-token").unwrap(),
                issued_at_unix_ms: 1,
                idle_expires_at_unix_ms: 2,
                absolute_expires_at_unix_ms: 3,
                limits: SessionLimits {
                    idle_timeout_ms: 1,
                    absolute_timeout_ms: 2,
                    max_open_transactions: 1,
                },
            },
        );

        let rendered = format!("{session:?}");
        assert!(rendered.contains("[REDACTED]"));
        assert!(!rendered.contains("secret-bearer-token"));
    }
}
