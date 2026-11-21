use super::super::*;

impl AppState {
    pub(in crate::http) fn read_project_configuration(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<rrd_contract::ReadProjectConfiguration, _, _>(
            headers,
            body,
            now,
            RrdOperation::EngramRead,
            None,
            |envelope, session, token| {
                self.service
                    .read_project_configuration(session, token, &envelope.context, now)
                    .map_err(api_error)
            },
        )
    }
    pub(in crate::http) fn preview_project_configuration(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<rrd_contract::PreviewProjectConfiguration, _, _>(
            headers,
            body,
            now,
            RrdOperation::ConfigurationPreview,
            None,
            |envelope, session, token| {
                self.service
                    .preview_project_configuration(
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
    pub(in crate::http) fn apply_project_configuration(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        now: u64,
    ) -> HttpResponse {
        self.with_authenticated_envelope::<rrd_contract::ApplyProjectConfiguration, _, _>(
            headers,
            body,
            now,
            RrdOperation::EngramAdmin,
            None,
            |envelope, session, token| {
                self.service
                    .apply_project_configuration(
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
