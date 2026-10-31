//! Typed attunement calls use generated catalogue bindings and engine-owned plans.
use crate::error::contract;
use crate::operations::{self as generated, GeneratedOperation};
use crate::{Error, RequestOptions, Result, RrdClient, Session};
use hyper::Method;
use rrd_contract::{
    AcceptAttunementReplan, AttunementControlResponse, AttunementJobState, AttunementPlan,
    AttunementReplanPreview, AttunementReplanReceiptReadResult, AttunementReplanResult,
    CanonicalId, EndpointAuthentication, HttpMethod, PauseAttunement, PreviewAttunementReplan,
    ReadAttunementReceipt, RecoverExpiredAttunement, ResumeAttunementWithLease, StartAttunement,
    StepAttunement,
};

impl RrdClient {
    pub(crate) async fn generated_attunement_call<O: GeneratedOperation>(
        &self,
        session: &Session,
        request: &O::Request,
        options: RequestOptions,
    ) -> Result<O::Response> {
        let catalogue = rrd_contract::endpoint_catalogue();
        let endpoint = catalogue
            .endpoints
            .iter()
            .find(|endpoint| endpoint.operation.as_str() == O::ID)
            .ok_or_else(|| Error::Contract("generated operation is unavailable".into()))?;
        if endpoint.method != HttpMethod::Post
            || endpoint.authentication != EndpointAuthentication::SessionBearer
            || endpoint.path.contains('{')
        {
            return Err(Error::Contract(
                "generated attunement operation has an incompatible transport".into(),
            ));
        }
        self.session_call(
            Method::POST,
            &endpoint.path,
            session,
            request,
            options,
            endpoint.mutation,
        )
        .await
    }

    fn validate_attunement_plan(&self, plan: &AttunementPlan) -> Result<()> {
        plan.validate().map_err(contract)?;
        if plan.target.instance_id != self.instance {
            return Err(Error::ResponseIdentityMismatch);
        }
        Ok(())
    }

    fn validate_control(
        &self,
        result: &AttunementControlResponse,
        job: &CanonicalId,
        previous_revision: u64,
        plan: &AttunementPlan,
        expected_state: Option<AttunementJobState>,
    ) -> Result<()> {
        self.validate_attunement_plan(plan)?;
        result
            .validate_for(job, previous_revision, plan)
            .map_err(contract)?;
        if expected_state.is_some_and(|state| result.status.job.state != state) {
            return Err(Error::ResponseIdentityMismatch);
        }
        Ok(())
    }

    pub async fn start_attunement(
        &self,
        session: &Session,
        request: StartAttunement,
        plan: &AttunementPlan,
        options: RequestOptions,
    ) -> Result<AttunementControlResponse> {
        request.validate().map_err(contract)?;
        self.validate_attunement_plan(plan)?;
        if request.plan_sha256 != plan.plan_sha256 {
            return Err(Error::ResponseIdentityMismatch);
        }
        let result = self
            .generated_attunement_call::<generated::AttunementStart>(session, &request, options)
            .await?;
        self.validate_control(
            &result,
            &request.job_id,
            request.expected_revision,
            plan,
            Some(AttunementJobState::Running),
        )?;
        Ok(result)
    }

    pub async fn step_attunement(
        &self,
        session: &Session,
        request: StepAttunement,
        plan: &AttunementPlan,
        options: RequestOptions,
    ) -> Result<AttunementControlResponse> {
        request.validate().map_err(contract)?;
        self.validate_attunement_plan(plan)?;
        let result = self
            .generated_attunement_call::<generated::AttunementStep>(session, &request, options)
            .await?;
        self.validate_control(
            &result,
            &request.job_id,
            request.expected_revision,
            plan,
            None,
        )?;
        Ok(result)
    }

    pub async fn pause_attunement(
        &self,
        session: &Session,
        request: PauseAttunement,
        plan: &AttunementPlan,
        options: RequestOptions,
    ) -> Result<AttunementControlResponse> {
        request.validate().map_err(contract)?;
        self.validate_attunement_plan(plan)?;
        let result = self
            .generated_attunement_call::<generated::AttunementPause>(session, &request, options)
            .await?;
        self.validate_control(
            &result,
            &request.job_id,
            request.expected_revision,
            plan,
            Some(AttunementJobState::Paused),
        )?;
        Ok(result)
    }

    pub async fn recover_expired_attunement(
        &self,
        session: &Session,
        request: RecoverExpiredAttunement,
        plan: &AttunementPlan,
        options: RequestOptions,
    ) -> Result<AttunementControlResponse> {
        request.validate().map_err(contract)?;
        self.validate_attunement_plan(plan)?;
        let result = self
            .generated_attunement_call::<generated::AttunementRecoverExpired>(
                session, &request, options,
            )
            .await?;
        self.validate_control(
            &result,
            &request.job_id,
            request.expected_revision,
            plan,
            Some(AttunementJobState::Paused),
        )?;
        Ok(result)
    }

    pub async fn resume_attunement(
        &self,
        session: &Session,
        request: ResumeAttunementWithLease,
        plan: &AttunementPlan,
        options: RequestOptions,
    ) -> Result<AttunementControlResponse> {
        self.validate_attunement_plan(plan)?;
        request.request.validate().map_err(contract)?;
        if request.request.plan_sha256 != plan.plan_sha256 {
            return Err(Error::ResponseIdentityMismatch);
        }
        let result = self
            .generated_attunement_call::<generated::AttunementResume>(session, &request, options)
            .await?;
        self.validate_control(
            &result,
            &request.request.job_id,
            request.request.expected_job_revision,
            plan,
            Some(AttunementJobState::Running),
        )?;
        Ok(result)
    }

    pub async fn preview_attunement_replan(
        &self,
        session: &Session,
        request: PreviewAttunementReplan,
        old_plan: &AttunementPlan,
        options: RequestOptions,
    ) -> Result<AttunementReplanPreview> {
        request.validate().map_err(contract)?;
        self.validate_attunement_plan(old_plan)?;
        let preview = self
            .generated_attunement_call::<generated::AttunementReplanPreview>(
                session, &request, options,
            )
            .await?;
        preview
            .validate_for(&request.job_id, request.expected_revision, old_plan)
            .map_err(contract)?;
        Ok(preview)
    }

    pub async fn accept_attunement_replan(
        &self,
        session: &Session,
        request: AcceptAttunementReplan,
        old_plan: &AttunementPlan,
        options: RequestOptions,
    ) -> Result<AttunementReplanResult> {
        request.validate().map_err(contract)?;
        self.validate_attunement_plan(old_plan)?;
        request
            .preview
            .validate_for(
                &request.preview.predecessor_job_id,
                request.preview.predecessor_revision,
                old_plan,
            )
            .map_err(contract)?;
        if request.proposal_sha256 != request.preview.proposal_sha256 {
            return Err(Error::ResponseIdentityMismatch);
        }
        let result = self
            .generated_attunement_call::<generated::AttunementReplanAccept>(
                session, &request, options,
            )
            .await?;
        result
            .validate_for(&request.preview, old_plan)
            .map_err(contract)?;
        Ok(result)
    }

    pub async fn read_attunement_replan_receipt(
        &self,
        session: &Session,
        request: ReadAttunementReceipt,
        options: RequestOptions,
    ) -> Result<AttunementReplanReceiptReadResult> {
        request.validate().map_err(contract)?;
        let result = self
            .generated_attunement_call::<generated::AttunementReplanReceiptRead>(
                session, &request, options,
            )
            .await?;
        result
            .validate_for(&request, session.principal_id(), &self.instance)
            .map_err(contract)?;
        Ok(result)
    }
}
