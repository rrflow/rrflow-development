use super::*;
use rrd_contract::{
    endpoint_catalogue, ApplyProjectConfiguration, PreviewProjectConfiguration,
    ProjectConfiguration, ProjectConfigurationPlan, ReadProjectConfiguration,
};

impl RrdClient {
    pub async fn read_project_configuration(
        &self,
        session: &Session,
        request: ReadProjectConfiguration,
        options: RequestOptions,
    ) -> Result<ProjectConfiguration> {
        let catalogue = endpoint_catalogue();
        let endpoint = catalogue
            .endpoints
            .iter()
            .find(|e| e.operation.as_str() == "project-configuration-read")
            .ok_or_else(|| Error::Contract("project configuration operation unavailable".into()))?;
        let result: ProjectConfiguration = self
            .session_call(
                Method::POST,
                &endpoint.path,
                session,
                request,
                options,
                endpoint.mutation,
            )
            .await?;
        result.validate().map_err(contract)?;
        if result.identity.instance_id != self.instance {
            return Err(Error::ResponseIdentityMismatch);
        }
        Ok(result)
    }
    pub async fn preview_project_configuration(
        &self,
        session: &Session,
        request: PreviewProjectConfiguration,
        options: RequestOptions,
    ) -> Result<ProjectConfigurationPlan> {
        request.validate().map_err(contract)?;
        let expected = request.clone();
        let catalogue = endpoint_catalogue();
        let endpoint = catalogue
            .endpoints
            .iter()
            .find(|e| e.operation.as_str() == "project-configuration-preview")
            .ok_or_else(|| Error::Contract("project configuration operation unavailable".into()))?;
        let result: ProjectConfigurationPlan = self
            .session_call(
                Method::POST,
                &endpoint.path,
                session,
                request,
                options,
                endpoint.mutation,
            )
            .await?;
        result.validate().map_err(contract)?;
        if result.current.identity.instance_id != self.instance {
            return Err(Error::ResponseIdentityMismatch);
        }
        if result.current.revision() != expected.expected_revision
            || result.change != expected.change
        {
            return Err(Error::ResponseIdentityMismatch);
        }
        Ok(result)
    }
    pub async fn apply_project_configuration(
        &self,
        session: &Session,
        request: ApplyProjectConfiguration,
        options: RequestOptions,
    ) -> Result<ProjectConfiguration> {
        request.validate().map_err(contract)?;
        let expected = request.clone();
        let catalogue = endpoint_catalogue();
        let endpoint = catalogue
            .endpoints
            .iter()
            .find(|e| e.operation.as_str() == "project-configuration-apply")
            .ok_or_else(|| Error::Contract("project configuration operation unavailable".into()))?;
        let result: ProjectConfiguration = self
            .session_call(
                Method::POST,
                &endpoint.path,
                session,
                request,
                options,
                endpoint.mutation,
            )
            .await?;
        result.validate().map_err(contract)?;
        if result.identity.instance_id != self.instance {
            return Err(Error::ResponseIdentityMismatch);
        }
        if result.identity != expected.plan.current.identity
            || result.revision() != expected.plan.current.revision() + 1
            || result.head.as_ref().is_none_or(|head| {
                head.applied_plan_sha256 != expected.plan.plan_sha256
                    || head.change != expected.plan.change
            })
        {
            return Err(Error::ResponseIdentityMismatch);
        }
        Ok(result)
    }
}
