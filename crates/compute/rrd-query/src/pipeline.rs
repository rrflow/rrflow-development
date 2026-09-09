use crate::{
    bind, execute, plan, BoundQuery, Catalog, Error, ExecutionBudget, Parameters, PhysicalPlan,
    Query, QueryExecution, Result,
};
use rrd_core::ReadStamp;
use rrd_store::{RuntimeReadBudget, StorageEngine};

/// One provider-neutral rrflowQL pipeline bound to a caller-owned read stamp.
///
/// The caller decides when the read coordinate is captured. Every catalogue,
/// bound query, physical plan, and execution produced here must retain that
/// coordinate, including when observability writes advance the live head.
pub struct StampedQueryPipeline<'a, E: StorageEngine> {
    engine: &'a E,
    read: ReadStamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StampedQueryExecution {
    pub bound: BoundQuery,
    pub plan: PhysicalPlan,
    pub execution: QueryExecution,
}

impl<'a, E: StorageEngine> StampedQueryPipeline<'a, E> {
    pub fn new(engine: &'a E, read: ReadStamp) -> Result<Self> {
        read.validate()
            .map_err(|error| Error::Integrity(error.to_string()))?;
        Ok(Self { engine, read })
    }

    pub fn read(&self) -> &ReadStamp {
        &self.read
    }

    pub fn bind(
        &self,
        query: &Query,
        parameters: &Parameters,
        budget: &ExecutionBudget,
    ) -> Result<BoundQuery> {
        let catalog = Catalog::capture_for_query_at(
            self.engine,
            self.read.clone(),
            query,
            RuntimeReadBudget::new(budget.max_storage_keys)?,
        )?;
        let bound = bind(query, parameters, &catalog)?;
        self.require_bound_read(&bound)?;
        Ok(bound)
    }

    pub fn plan(&self, bound: &BoundQuery) -> Result<PhysicalPlan> {
        self.require_bound_read(bound)?;
        let physical = plan(bound)?;
        self.require_plan_read(&physical)?;
        Ok(physical)
    }

    pub fn execute(
        &self,
        physical: &PhysicalPlan,
        budget: &ExecutionBudget,
    ) -> Result<QueryExecution> {
        self.require_plan_read(physical)?;
        let execution = execute(self.engine, physical, budget)?;
        let contract = &physical.explanation.contract;
        if execution.plan_digest != physical.digest
            || execution.read_manifest != self.read.manifest_id
            || execution.valid_at != contract.valid_at
            || execution.known_at_cursor != contract.known_at_cursor
        {
            return Err(Error::Integrity(
                "query execution changed its stamped plan coordinate".into(),
            ));
        }
        Ok(execution)
    }

    pub fn run(
        &self,
        query: &Query,
        parameters: &Parameters,
        budget: &ExecutionBudget,
    ) -> Result<StampedQueryExecution> {
        let bound = self.bind(query, parameters, budget)?;
        let plan = self.plan(&bound)?;
        let execution = self.execute(&plan, budget)?;
        Ok(StampedQueryExecution {
            bound,
            plan,
            execution,
        })
    }

    fn require_bound_read(&self, bound: &BoundQuery) -> Result<()> {
        if bound.read != self.read {
            return Err(Error::Integrity(
                "bound query does not retain the pipeline read stamp".into(),
            ));
        }
        Ok(())
    }

    fn require_plan_read(&self, physical: &PhysicalPlan) -> Result<()> {
        physical.verify()?;
        if physical.logical.read != self.read
            || physical.explanation.contract.read_manifest != self.read.manifest_id
        {
            return Err(Error::Integrity(
                "physical plan does not retain the pipeline read stamp".into(),
            ));
        }
        Ok(())
    }
}
