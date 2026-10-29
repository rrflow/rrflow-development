use super::super::*;

impl AppState {
    pub(in crate::http) fn search_vectors(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<SearchVectors, _, _>(
            headers,
            body,
            now,
            RrdOperation::VectorSearch,
            None,
            |envelope, session, token| {
                self.service
                    .search_vectors(
                        session,
                        token,
                        &envelope.payload,
                        now,
                        envelope.context.request_id.as_str(),
                        envelope.context.operation_id.as_str(),
                    )
                    .map_err(api_error)
            },
        )
    }

    pub(in crate::http) fn ensure_vector_collection(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<EnsureVectorCollection, _, _>(
            headers,
            body,
            now,
            RrdOperation::VectorCollectionEnsure,
            None,
            |envelope, session, token| {
                self.service
                    .ensure_vector_collection(
                        session,
                        token,
                        &envelope.payload,
                        &envelope.context,
                        now,
                    )
                    .map_err(api_error)
            },
        )
    }

    pub(in crate::http) fn list_vector_collections(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<ListVectorCollections, _, _>(
            headers,
            body,
            now,
            RrdOperation::VectorCollectionList,
            None,
            |envelope, session, token| {
                self.service
                    .list_vector_collections(
                        session,
                        token,
                        &envelope.payload,
                        now,
                        envelope.context.request_id.as_str(),
                        envelope.context.operation_id.as_str(),
                    )
                    .map_err(api_error)
            },
        )
    }

    pub(in crate::http) fn scroll_vector_points(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<ScrollVectorPoints, _, _>(
            headers,
            body,
            now,
            RrdOperation::VectorPointScroll,
            None,
            |envelope, session, token| {
                self.service
                    .scroll_vector_points(
                        session,
                        token,
                        &envelope.payload,
                        now,
                        envelope.context.request_id.as_str(),
                        envelope.context.operation_id.as_str(),
                    )
                    .map_err(api_error)
            },
        )
    }

    pub(in crate::http) fn retrieve_vector_points(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<RetrieveVectorPoints, _, _>(
            headers,
            body,
            now,
            RrdOperation::VectorPointRetrieve,
            None,
            |envelope, session, token| {
                self.service
                    .retrieve_vector_points(
                        session,
                        token,
                        &envelope.payload,
                        now,
                        envelope.context.request_id.as_str(),
                        envelope.context.operation_id.as_str(),
                    )
                    .map_err(api_error)
            },
        )
    }

    pub(in crate::http) fn build_vector_quantization(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<BuildVectorQuantizationArtifact, _, _>(
            headers,
            body,
            now,
            RrdOperation::VectorCollectionEnsure,
            None,
            |envelope, session, token| {
                self.service
                    .build_vector_quantization_artifact(
                        session,
                        token,
                        &envelope.payload,
                        &envelope.context,
                        now,
                    )
                    .map_err(api_error)
            },
        )
    }

    pub(in crate::http) fn list_vector_quantizations(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<ListVectorQuantizationArtifacts, _, _>(
            headers,
            body,
            now,
            RrdOperation::VectorCollectionList,
            None,
            |envelope, session, token| {
                self.service
                    .list_vector_quantization_artifacts(
                        session,
                        token,
                        &envelope.payload,
                        now,
                        envelope.context.request_id.as_str(),
                        envelope.context.operation_id.as_str(),
                    )
                    .map_err(api_error)
            },
        )
    }

    pub(in crate::http) fn activate_vector_quantization(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<ActivateVectorQuantizationArtifact, _, _>(
            headers,
            body,
            now,
            RrdOperation::VectorCollectionEnsure,
            None,
            |envelope, session, token| {
                self.service
                    .activate_vector_quantization_artifact(
                        session,
                        token,
                        &envelope.payload,
                        &envelope.context,
                        now,
                    )
                    .map_err(api_error)
            },
        )
    }

    pub(in crate::http) fn retire_vector_quantization(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<RetireVectorQuantizationArtifact, _, _>(
            headers,
            body,
            now,
            RrdOperation::VectorCollectionEnsure,
            None,
            |envelope, session, token| {
                self.service
                    .retire_vector_quantization_artifact(
                        session,
                        token,
                        &envelope.payload,
                        &envelope.context,
                        now,
                    )
                    .map_err(api_error)
            },
        )
    }
}
