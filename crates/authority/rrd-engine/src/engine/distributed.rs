use super::*;
use rrd_cluster::{
    ClusterId, DistributedAuthorityCatalogue, LogicalTableId, NodeId, ReadConsistency,
    ReplicaObservation, RouteEvidence, ShardId, ShardReadStamp, METADATA_SHARD_ID,
};

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedDistributedAuthority {
    pub metadata_shard: ShardId,
    pub read: ReadStamp,
    pub catalogue_sha256: String,
    pub commit: RuntimeCommit,
}

impl PreparedDistributedAuthority {
    /// Binds the canonical metadata commit directly to the existing OpenRaft
    /// command contract. The active consensus group supplies its current
    /// placement epoch and optional expected log index.
    #[cfg(feature = "cluster-transfer")]
    pub fn raft_command(
        &self,
        request_id: impl Into<String>,
        placement_epoch: u64,
        expected_commit_index: Option<u64>,
    ) -> Result<rrd_cluster::RrdRaftCommand> {
        rrd_cluster::RrdRaftCommand::runtime_commit(
            request_id,
            self.metadata_shard,
            placement_epoch,
            expected_commit_index,
            self.commit.clone(),
        )
        .map_err(cluster_error)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DistributedAuthorityRead {
    pub read: ReadStamp,
    pub catalogue: DistributedAuthorityCatalogue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistributedReadRoute {
    pub authority_read: ReadStamp,
    pub catalogue_revision: u64,
    pub table: LogicalTableId,
    pub shard: ShardId,
    pub evidence: RouteEvidence,
}

impl RrdEngine {
    /// Prepares the one canonical metadata-shard transaction that an OpenRaft
    /// coordinator orders. This method never commits around consensus.
    pub fn prepare_distributed_authority(
        &self,
        catalogue: &DistributedAuthorityCatalogue,
        actor: &str,
        max_scanned_changes: usize,
    ) -> Result<PreparedDistributedAuthority> {
        catalogue.validate().map_err(cluster_error)?;
        validate_replay_limit(max_scanned_changes)?;
        let scope = distributed_scope(&self.instance)?;
        let schema = self.storage.runtime().schema(&scope)?;
        let (read, previous) = if schema.is_some() {
            let (read, snapshot) = self.storage.runtime().data_snapshot(
                &scope,
                catalogue.updated_at,
                max_scanned_changes,
            )?;
            let previous =
                DistributedAuthorityCatalogue::from_runtime_snapshot(&snapshot, &catalogue.cluster)
                    .map_err(cluster_error)?;
            (read, previous)
        } else {
            (self.storage.runtime().read_stamp(&scope)?, None)
        };
        match previous {
            Some(previous) => catalogue
                .validate_successor(&previous)
                .map_err(cluster_error)?,
            None if catalogue.revision == 1 => {}
            None => {
                return Err(ServiceError::Runtime(
                    "the first distributed authority revision must be 1".into(),
                ));
            }
        }
        let catalogue_sha256 = catalogue.digest().map_err(cluster_error)?;
        let commit = catalogue
            .prepare_runtime_commit(scope, actor, read.commit_cursor, schema.as_ref())
            .map_err(cluster_error)?;
        Ok(PreparedDistributedAuthority {
            metadata_shard: METADATA_SHARD_ID,
            read,
            catalogue_sha256,
            commit,
        })
    }

    /// Reads the topology catalogue and its authenticated RRD read stamp from
    /// the same bounded runtime snapshot.
    pub fn read_distributed_authority(
        &self,
        cluster: &ClusterId,
        valid_at: u64,
        max_scanned_changes: usize,
    ) -> Result<Option<DistributedAuthorityRead>> {
        validate_replay_limit(max_scanned_changes)?;
        let scope = distributed_scope(&self.instance)?;
        if self.storage.runtime().schema(&scope)?.is_none() {
            return Ok(None);
        }
        let (read, snapshot) =
            self.storage
                .runtime()
                .data_snapshot(&scope, valid_at, max_scanned_changes)?;
        let catalogue = DistributedAuthorityCatalogue::from_runtime_snapshot(&snapshot, cluster)
            .map_err(cluster_error)?;
        Ok(catalogue.map(|catalogue| DistributedAuthorityRead { read, catalogue }))
    }

    /// Routes through the durable topology revision observed by this engine;
    /// callers cannot supply a parallel placement map.
    #[allow(clippy::too_many_arguments)]
    pub fn route_distributed_read(
        &self,
        cluster: &ClusterId,
        table: &LogicalTableId,
        partition_hash: u64,
        requested_consistency: ReadConsistency,
        exact_snapshot: Option<&ShardReadStamp>,
        observations: &BTreeMap<NodeId, ReplicaObservation>,
        valid_at: u64,
        max_scanned_changes: usize,
    ) -> Result<DistributedReadRoute> {
        let authority = self
            .read_distributed_authority(cluster, valid_at, max_scanned_changes)?
            .ok_or_else(|| {
                ServiceError::Runtime(format!(
                    "distributed authority for cluster {cluster} was not found"
                ))
            })?;
        let evidence = authority
            .catalogue
            .route_read(
                table,
                partition_hash,
                requested_consistency,
                exact_snapshot,
                observations,
            )
            .map_err(cluster_error)?;
        Ok(DistributedReadRoute {
            authority_read: authority.read,
            catalogue_revision: authority.catalogue.revision,
            table: table.clone(),
            shard: evidence.shard,
            evidence,
        })
    }
}

fn distributed_scope(instance: &CanonicalId) -> Result<ScopeId> {
    ScopeId::new(format!("instance:{instance}"))
        .map_err(|error| ServiceError::Runtime(error.to_string()))
}

fn validate_replay_limit(max_scanned_changes: usize) -> Result<()> {
    if max_scanned_changes == 0 {
        return Err(ServiceError::Runtime(
            "distributed authority replay limit must be greater than zero".into(),
        ));
    }
    Ok(())
}

fn cluster_error(error: rrd_cluster::ClusterError) -> ServiceError {
    ServiceError::Runtime(error.to_string())
}
