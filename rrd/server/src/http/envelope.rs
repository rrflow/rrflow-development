use super::*;

impl AppState {
    pub(super) fn with_envelope<T, O, F>(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
        operation_kind: RrdOperation,
        operation: F,
    ) -> HttpResponse
    where
        T: DeserializeOwned,
        O: Serialize,
        F: FnOnce(&RequestEnvelope<T>, Option<SessionIdentity>) -> std::result::Result<O, ApiError>,
    {
        let attempt = HTTP_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let envelope = match parse_envelope::<T>(
            headers,
            body,
            now,
            operation_kind.mutates(),
            self.service.instance_id(),
        ) {
            Ok(envelope) => envelope,
            Err(error) => {
                let (context, error) = *error;
                return self.rejected_response(
                    context,
                    instance_resource(self.service.instance_id()),
                    body,
                    now,
                    attempt,
                    operation_kind,
                    None,
                    error,
                );
            }
        };
        let invocation = invocation(&envelope, body, now, attempt);
        let identity = if has_session_creation_headers(headers) {
            match session_creation_identity(headers) {
                Ok(identity) => Some(identity),
                Err(error) => {
                    return self.rejected_response(
                        envelope.context,
                        envelope.resource,
                        body,
                        now,
                        attempt,
                        operation_kind,
                        None,
                        error,
                    );
                }
            }
        } else {
            None
        };
        let authorized = match identity.as_ref() {
            Some(SessionIdentity::ApiKey {
                principal_id,
                credential,
            }) => self.service.begin_invocation(
                invocation,
                operation_kind,
                InvocationCredential::ApiKey {
                    principal_id,
                    credential: credential.as_bytes(),
                },
            ),
            Some(SessionIdentity::Jwt(token)) => self.service.begin_invocation(
                invocation,
                operation_kind,
                InvocationCredential::Jwt {
                    token,
                    signing_key: self
                        .jwt_verification_key
                        .as_ref()
                        .map_or(&[], |key| key.as_bytes()),
                },
            ),
            None => self.service.begin_invocation(
                invocation,
                operation_kind,
                InvocationCredential::Anonymous,
            ),
        };
        let authorized = match authorized {
            Ok(authorized) => authorized,
            Err(error) => return failure(&envelope.context, api_error(error)),
        };

        let response = match operation(&envelope, identity) {
            Ok(payload) => success(StatusCode::OK, &envelope.context, payload),
            Err(error) => failure(&envelope.context, error),
        };
        self.complete_response(&envelope.context, &authorized, response)
    }

    pub(super) fn with_authenticated_envelope<T, O, F>(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
        operation_kind: RrdOperation,
        expected_session: Option<&str>,
        operation: F,
    ) -> HttpResponse
    where
        T: DeserializeOwned,
        O: Serialize,
        F: FnOnce(
            &RequestEnvelope<T>,
            &CorrelationId,
            &CorrelationId,
        ) -> std::result::Result<O, ApiError>,
    {
        let attempt = HTTP_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let envelope = match parse_envelope::<T>(
            headers,
            body,
            now,
            operation_kind.mutates(),
            self.service.instance_id(),
        ) {
            Ok(envelope) => envelope,
            Err(error) => {
                let (context, error) = *error;
                return self.rejected_response(
                    context,
                    instance_resource(self.service.instance_id()),
                    body,
                    now,
                    attempt,
                    operation_kind,
                    None,
                    error,
                );
            }
        };
        let session = match authenticated_session(headers, expected_session) {
            Ok(session) => session,
            Err(error) => {
                return self.rejected_response(
                    envelope.context,
                    envelope.resource,
                    body,
                    now,
                    attempt,
                    operation_kind,
                    None,
                    error,
                );
            }
        };
        let authorized = match self.service.begin_invocation(
            invocation(&envelope, body, now, attempt),
            operation_kind,
            InvocationCredential::Session {
                session_id: &session.0,
                token: &session.1,
            },
        ) {
            Ok(authorized) => authorized,
            Err(error) => return failure(&envelope.context, api_error(error)),
        };

        let response = match operation(&envelope, &session.0, &session.1) {
            Ok(payload) => success(StatusCode::OK, &envelope.context, payload),
            Err(error) => failure(&envelope.context, error),
        };
        self.complete_response(&envelope.context, &authorized, response)
    }

    fn complete_response(
        &self,
        context: &RequestContext,
        authorized: &AuthorizedInvocation,
        response: HttpResponse,
    ) -> HttpResponse {
        let completion = invocation_completion(&response);
        match self.service.complete_invocation(authorized, completion) {
            Ok(()) => response,
            Err(error) => failure(context, api_error(error)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::http) fn rejected_response(
        &self,
        context: RequestContext,
        resource: rrd_contract::ResourcePath,
        body: &[u8],
        now: u64,
        attempt: u64,
        operation: RrdOperation,
        principal_id: Option<CanonicalId>,
        error: ApiError,
    ) -> HttpResponse {
        let context = if context.validate(false).is_ok() {
            context
        } else {
            generated_context(now, "rejected-invocation")
        };
        let response = failure(&context, error);
        let invocation = Invocation {
            context: context.clone(),
            resource,
            observed_at_unix_ms: now,
            attempt,
            request_sha256: sha256_hex(body),
        };
        match self.service.record_invocation_failure(
            invocation,
            operation,
            principal_id,
            invocation_completion(&response),
        ) {
            Ok(()) => response,
            Err(error) => failure(&context, api_error(error)),
        }
    }
}

pub(in crate::http) fn invocation<T>(
    envelope: &RequestEnvelope<T>,
    body: &[u8],
    now: u64,
    attempt: u64,
) -> Invocation {
    Invocation {
        context: envelope.context.clone(),
        resource: envelope.resource.clone(),
        observed_at_unix_ms: now,
        attempt,
        request_sha256: sha256_hex(body),
    }
}

pub(super) fn invocation_completion(response: &HttpResponse) -> InvocationCompletion {
    let status_code = response.status().as_u16();
    let decision = match status_code {
        200..=299 => AuditDecision::Allowed,
        401 | 403 => AuditDecision::Denied,
        _ => AuditDecision::Failed,
    };
    let response_sha256 = response
        .extensions()
        .get::<ResponseDigest>()
        .map(|digest| digest.0.clone())
        .unwrap_or_else(|| sha256_hex(status_code.to_string().as_bytes()));
    InvocationCompletion {
        decision,
        status_code,
        response_sha256,
    }
}

fn has_session_creation_headers(headers: &HeaderMap) -> bool {
    headers.contains_key("X-RRD-Principal") || headers.contains_key("Authorization")
}

pub(in crate::http) fn parse_envelope<T: DeserializeOwned>(
    headers: &HeaderMap,
    bytes: &[u8],
    now: u64,
    mutation: bool,
    instance: &CanonicalId,
) -> std::result::Result<RequestEnvelope<T>, Box<(RequestContext, ApiError)>> {
    let fallback = generated_context(now, "invalid-envelope");
    if !header_values(headers, "Content-Type")
        .is_ok_and(|values| values.len() == 1 && values[0].starts_with("application/json"))
    {
        return Err(Box::new((
            fallback,
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Content-Type must be application/json",
                false,
            ),
        )));
    }
    if bytes.len() > RRD_MAX_BODY_BYTES {
        return Err(Box::new((
            fallback,
            ApiError::new(
                ErrorCode::ResourceExhausted,
                "request body exceeds one MiB",
                false,
            ),
        )));
    }
    let envelope: RequestEnvelope<T> = serde_json::from_slice(bytes).map_err(|error| {
        // A payload that does not decode is still answered under the caller's
        // own request identity when the envelope carries a valid one, so the
        // client sees this typed error instead of an identity mismatch.
        #[derive(serde::Deserialize)]
        struct CallerContext {
            context: RequestContext,
        }
        let context = serde_json::from_slice::<CallerContext>(bytes)
            .ok()
            .map(|caller| caller.context)
            .filter(|context| context.validate(false).is_ok())
            .unwrap_or_else(|| fallback.clone());
        Box::new((
            context,
            ApiError::new(ErrorCode::InvalidArgument, error.to_string(), false),
        ))
    })?;
    envelope.validate(mutation).map_err(|error| {
        Box::new((
            envelope.context.clone(),
            ApiError::new(ErrorCode::InvalidArgument, error.to_string(), false),
        ))
    })?;
    let target = envelope
        .resource
        .segments
        .iter()
        .find(|segment| segment.kind == ResourceKind::Instance);
    if target.is_none_or(|target| target.id != *instance) {
        return Err(Box::new((
            envelope.context.clone(),
            ApiError::new(
                ErrorCode::FailedPrecondition,
                "request resource does not target this instance",
                false,
            ),
        )));
    }
    if envelope
        .context
        .deadline_unix_ms
        .is_some_and(|deadline| deadline <= now)
    {
        return Err(Box::new((
            envelope.context.clone(),
            ApiError::new(
                ErrorCode::DeadlineExceeded,
                "request deadline elapsed",
                false,
            ),
        )));
    }
    Ok(envelope)
}

pub(super) fn required_idempotency(
    context: &RequestContext,
) -> std::result::Result<&CorrelationId, ApiError> {
    context.idempotency_key.as_ref().ok_or_else(|| {
        ApiError::new(
            ErrorCode::InvalidArgument,
            "mutation requires idempotency key",
            false,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[allow(dead_code)]
    struct Payload {
        required: u64,
    }

    /// A payload that does not decode is answered under the caller's request
    /// identity, so the client sees the typed error. The 2026-11-12 real run
    /// saw "identity differs" in the SDK where the server had answered a
    /// missing `max_storage_keys` under an identity it generated itself.
    #[test]
    fn an_undecodable_payload_keeps_the_callers_identity() {
        let instance = CanonicalId::new("instance-envelope").unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        let context = RequestContext {
            request_id: CorrelationId::new("request-caller").unwrap(),
            operation_id: CorrelationId::new("operation-caller").unwrap(),
            idempotency_key: None,
            deadline_unix_ms: None,
        };
        let body = serde_json::to_vec(&serde_json::json!({
            "protocol": PROTOCOL,
            "protocol_version": PROTOCOL_VERSION,
            "context": context,
            "resource": instance_resource(&instance),
            "payload": {"other": 1},
        }))
        .unwrap();
        let (answered, error) =
            *parse_envelope::<Payload>(&headers, &body, 1_000, false, &instance)
                .expect_err("the payload lacks its required field");
        assert_eq!(answered.request_id, context.request_id);
        assert_eq!(answered.operation_id, context.operation_id);
        assert!(format!("{error:?}").contains("required"), "{error:?}");

        // Without a readable context the server still answers, under an
        // identity it generates.
        let (generated, _) =
            *parse_envelope::<Payload>(&headers, b"{\"payload\":1}", 1_000, false, &instance)
                .expect_err("not an envelope");
        assert_ne!(generated.request_id, context.request_id);
    }
}
