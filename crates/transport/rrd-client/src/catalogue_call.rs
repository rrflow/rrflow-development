//! Generated carriage metadata with explicitly admitted semantic validators.
//!
//! Metadata alone does not make an operation callable. Each callable operation
//! supplies contract-owned input/output validation; it grants no engine authority.

use crate::error::{contract, decode};
use crate::operation::outcome;
use crate::{RequestOptions, Result, RrdClient, Session};
use hyper::StatusCode;
use serde::{de::DeserializeOwned, Serialize};

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// Replaceable catalogue projection for a fixed-path authenticated POST.
pub trait OperationMetadata: sealed::Sealed {
    type Request: Serialize;
    type Response: DeserializeOwned;
    const OPERATION: &'static str;
    const PATH: &'static str;
    const MUTATION: bool;
}

/// A semantic validator must exist before generated metadata can be invoked.
///
/// Metadata for other operations is not callable through this API yet:
/// ```compile_fail
/// use rrd_client::{operations::TaskOpen, RequestOptions, RrdClient, Session};
/// async fn unavailable(client: &RrdClient, session: &Session,
///     request: rrd_contract::OpenTask, options: RequestOptions) {
///     client.call_operation::<TaskOpen>(session, request, options).await.unwrap();
/// }
/// ```
pub trait ValidatedOperation: OperationMetadata {
    fn validate_request(request: &Self::Request) -> rrd_contract::Result<()>;
    fn validate_response(
        request: &Self::Request,
        response: &Self::Response,
    ) -> rrd_contract::Result<()>;
}

impl ValidatedOperation for crate::operations::TaskEndpointPropose {
    fn validate_request(request: &Self::Request) -> rrd_contract::Result<()> {
        request.validate()
    }

    fn validate_response(
        request: &Self::Request,
        response: &Self::Response,
    ) -> rrd_contract::Result<()> {
        response.validate_request_identity(request)
    }
}

impl ValidatedOperation for crate::operations::TaskWorkerExecute {
    fn validate_request(request: &Self::Request) -> rrd_contract::Result<()> {
        request.validate()
    }

    fn validate_response(
        request: &Self::Request,
        response: &Self::Response,
    ) -> rrd_contract::Result<()> {
        response.validate_request_identity(request)
    }
}

impl ValidatedOperation for crate::operations::TaskBudgetResume {
    fn validate_request(request: &Self::Request) -> rrd_contract::Result<()> {
        request.validate()
    }

    fn validate_response(
        request: &Self::Request,
        response: &Self::Response,
    ) -> rrd_contract::Result<()> {
        request.validate_response(response)
    }
}

impl RrdClient {
    /// Carry one validated operation through the current native session.
    ///
    /// Never retries after timeout or lost ACK. Read the engine-owned attempt
    /// before deciding any further effect; a proposal is not worker permission.
    pub async fn call_operation<O: ValidatedOperation>(
        &self,
        session: &Session,
        request: O::Request,
        options: RequestOptions,
    ) -> Result<O::Response> {
        O::validate_request(&request).map_err(contract)?;
        let expected = options.context();
        let envelope = self.envelope(&request, &options, O::MUTATION)?;
        let body = serde_json::to_vec(&envelope).map_err(decode)?;
        let authorization = format!("Bearer {}", session.token().as_str());
        let headers = [
            ("X-RRD-Session", session.session_id().as_str()),
            ("Authorization", authorization.as_str()),
        ];
        let response = self
            .send_operation(O::PATH, body, &headers, &expected)
            .await?;
        let response = outcome(StatusCode::OK, response, Some(&expected))?;
        O::validate_response(&request, &response).map_err(contract)?;
        Ok(response)
    }
}
