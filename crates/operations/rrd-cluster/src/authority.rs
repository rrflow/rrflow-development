use crate::{
    ClusterError, ClusterId, LogicalTableId, NodeId, PlacementPolicy, ReadConsistency, RegionId,
    ReplicaHealth, ReplicaPlacement, ReplicaRole, ReplicaTransferPlan, ReshardPlan, ReshardState,
    Result, RouteEvidence, ShardId, ShardPlacement, ShardReadStamp, TenantId, WriteConsistency,
    ZoneId, CLUSTER_CONTRACT_VERSION, METADATA_SHARD_ID,
};
use rrd_core::{
    digest, RuntimeDataSnapshot, RuntimeLogicalModel, RuntimeMutation, RuntimeProperties,
    RuntimePropertySchema, RuntimeRecord, RuntimeRecordSchema, RuntimeRef, RuntimeSchemaRegistry,
    RuntimeTableSchema, RuntimeType, RuntimeValue, RuntimeValueType, ScopeId,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const DISTRIBUTED_AUTHORITY_FORMAT: u16 = 1;
pub const DISTRIBUTED_AUTHORITY_RECORD_TYPE: &str = "rrd_cluster_topology";
pub const MAX_DISTRIBUTED_NODES: usize = 4_096;
pub const MAX_DISTRIBUTED_TENANTS: usize = 4_096;
pub const MAX_DISTRIBUTED_TABLES: usize = 16_384;
pub const MAX_DISTRIBUTED_SHARDS: usize = 65_535;
pub const MAX_DISTRIBUTED_OPERATIONS: usize = 16_384;
pub const MAX_DISTRIBUTED_AUTHORITY_BYTES: usize = 4 * 1024 * 1024;
const MAX_READ_REPLICAS: u8 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacementNodeState {
    Active,
    Draining,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlacementNode {
    pub node: NodeId,
    pub region: RegionId,
    pub zone: ZoneId,
    pub state: PlacementNodeState,
    pub voter_capacity: u32,
    pub learner_capacity: u32,
    pub observed_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TenantPlacementPolicy {
    pub tenant: TenantId,
    pub allowed_regions: BTreeSet<RegionId>,
    pub minimum_voter_regions: u8,
    pub placement: PlacementPolicy,
    pub read_replica_count: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogicalTablePolicy {
    pub table: LogicalTableId,
    pub tenant: TenantId,
    pub partition_key: String,
    pub shard_count: u16,
    pub first_shard: ShardId,
    pub default_read: ReadConsistency,
    pub write: WriteConsistency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShardHashRange {
    pub start_inclusive: u64,
    pub end_inclusive: u64,
}

impl ShardHashRange {
    pub fn contains(self, value: u64) -> bool {
        self.start_inclusive <= value && value <= self.end_inclusive
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShardAuthority {
    pub shard: ShardId,
    pub table: LogicalTableId,
    pub range: ShardHashRange,
    pub placement: ShardPlacement,
    pub replica_health: BTreeMap<NodeId, ReplicaHealth>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplicaObservation {
    pub node: NodeId,
    pub health: ReplicaHealth,
    pub stamp: ShardReadStamp,
    pub leader: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplicaTransferState {
    Planned,
    Snapshot,
    Wal,
    CaughtUp,
    Joined,
    Failed,
}

impl ReplicaTransferState {
    fn rank(self) -> u8 {
        match self {
            Self::Planned => 0,
            Self::Snapshot => 1,
            Self::Wal => 2,
            Self::CaughtUp => 3,
            Self::Joined | Self::Failed => 4,
        }
    }

    fn terminal(self) -> bool {
        matches!(self, Self::Joined | Self::Failed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplicaTransferAuthority {
    pub operation_id: String,
    pub plan: ReplicaTransferPlan,
    pub state: ReplicaTransferState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt_sha256: Option<String>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DistributedAuthorityCatalogue {
    pub format: u16,
    pub cluster: ClusterId,
    pub metadata_shard: ShardId,
    pub revision: u64,
    pub updated_at: u64,
    pub nodes: BTreeMap<NodeId, PlacementNode>,
    pub tenants: BTreeMap<TenantId, TenantPlacementPolicy>,
    pub tables: BTreeMap<LogicalTableId, LogicalTablePolicy>,
    pub shards: BTreeMap<ShardId, ShardAuthority>,
    pub transfers: BTreeMap<String, ReplicaTransferAuthority>,
    pub reshards: BTreeMap<String, ReshardPlan>,
}

impl DistributedAuthorityCatalogue {
    pub fn validate(&self) -> Result<()> {
        if self.format != DISTRIBUTED_AUTHORITY_FORMAT
            || self.metadata_shard != METADATA_SHARD_ID
            || self.revision == 0
            || self.updated_at == 0
        {
            return Err(ClusterError::Invalid(
                "distributed authority requires v1, metadata shard 0, and non-zero revision/time"
                    .into(),
            ));
        }
        if self.nodes.is_empty()
            || self.nodes.len() > MAX_DISTRIBUTED_NODES
            || self.tenants.is_empty()
            || self.tenants.len() > MAX_DISTRIBUTED_TENANTS
            || self.tables.is_empty()
            || self.tables.len() > MAX_DISTRIBUTED_TABLES
            || self.shards.is_empty()
            || self.shards.len() > MAX_DISTRIBUTED_SHARDS
            || self.transfers.len() > MAX_DISTRIBUTED_OPERATIONS
            || self.reshards.len() > MAX_DISTRIBUTED_OPERATIONS
        {
            return Err(ClusterError::Invalid(
                "distributed authority cardinality is outside its bounded contract".into(),
            ));
        }

        let mut zone_regions = BTreeMap::<&ZoneId, &RegionId>::new();
        for (key, node) in &self.nodes {
            if key != &node.node
                || node.observed_at == 0
                || (node.voter_capacity == 0 && node.learner_capacity == 0)
            {
                return Err(ClusterError::Invalid(
                    "placement node identity, observation, or capacity is invalid".into(),
                ));
            }
            if let Some(region) = zone_regions.insert(&node.zone, &node.region) {
                if region != &node.region {
                    return Err(ClusterError::Invalid(
                        "one availability zone cannot belong to multiple regions".into(),
                    ));
                }
            }
        }

        for (key, tenant) in &self.tenants {
            if key != &tenant.tenant
                || tenant.allowed_regions.is_empty()
                || tenant.minimum_voter_regions == 0
                || tenant.minimum_voter_regions > tenant.placement.voter_count
                || usize::from(tenant.minimum_voter_regions) > tenant.allowed_regions.len()
                || tenant.read_replica_count > MAX_READ_REPLICAS
            {
                return Err(ClusterError::Invalid(
                    "tenant placement identity or region/read-replica policy is invalid".into(),
                ));
            }
            tenant.placement.validate()?;
        }

        let mut table_shards = BTreeMap::<LogicalTableId, Vec<&ShardAuthority>>::new();
        for (key, table) in &self.tables {
            if key != &table.table
                || !self.tenants.contains_key(&table.tenant)
                || table.shard_count == 0
                || table.first_shard == METADATA_SHARD_ID
            {
                return Err(ClusterError::Invalid(
                    "logical table identity, tenant, or shard allocation is invalid".into(),
                ));
            }
            validate_text(&table.partition_key, "partition key")?;
            if matches!(
                table.default_read,
                ReadConsistency::BoundedStale {
                    maximum_index_lag: 0
                }
            ) {
                return Err(ClusterError::Invalid(
                    "bounded-stale reads require non-zero index lag".into(),
                ));
            }
        }

        for (key, shard) in &self.shards {
            if key != &shard.shard || !self.tables.contains_key(&shard.table) {
                return Err(ClusterError::Invalid(
                    "shard authority identity or logical table is invalid".into(),
                ));
            }
            if shard.placement.cluster != self.cluster
                || shard.placement.shard != shard.shard
                || shard.replica_health.len() != shard.placement.replicas.len()
            {
                return Err(ClusterError::Invalid(
                    "shard placement differs from its authority or health inventory".into(),
                ));
            }
            shard.placement.validate()?;
            let tenant = &self.tenants[&self.tables[&shard.table].tenant];
            let mut voter_regions = BTreeSet::new();
            for replica in &shard.placement.replicas {
                let node = self.nodes.get(&replica.node).ok_or_else(|| {
                    ClusterError::Invalid("shard placement names an unknown node".into())
                })?;
                if node.zone != replica.zone
                    || node.state != PlacementNodeState::Active
                    || !tenant.allowed_regions.contains(&node.region)
                    || !shard.replica_health.contains_key(&node.node)
                {
                    return Err(ClusterError::Invalid(
                        "shard replica violates node, zone, health, or tenant policy".into(),
                    ));
                }
                if replica.role == ReplicaRole::Voter {
                    voter_regions.insert(&node.region);
                }
            }
            if voter_regions.len() < usize::from(tenant.minimum_voter_regions) {
                return Err(ClusterError::Invalid(
                    "shard voters do not satisfy tenant region diversity".into(),
                ));
            }
            table_shards
                .entry(shard.table.clone())
                .or_default()
                .push(shard);
        }

        for table in self.tables.values() {
            let mut shards = table_shards.remove(&table.table).unwrap_or_default();
            shards.sort_by_key(|shard| shard.range.start_inclusive);
            if shards.len() != usize::from(table.shard_count) {
                return Err(ClusterError::Invalid(
                    "logical table shard count differs from its authority".into(),
                ));
            }
            let mut next = 0_u128;
            for shard in shards {
                if u128::from(shard.range.start_inclusive) != next
                    || shard.range.end_inclusive < shard.range.start_inclusive
                {
                    return Err(ClusterError::Invalid(
                        "logical table hash ranges are not contiguous".into(),
                    ));
                }
                next = u128::from(shard.range.end_inclusive) + 1;
            }
            if next != (1_u128 << 64) {
                return Err(ClusterError::Invalid(
                    "logical table hash ranges do not cover the complete u64 space".into(),
                ));
            }
        }

        for (key, transfer) in &self.transfers {
            validate_text(key, "transfer operation id")?;
            if key != &transfer.operation_id || transfer.updated_at == 0 {
                return Err(ClusterError::Invalid(
                    "replica transfer authority identity or time is invalid".into(),
                ));
            }
            transfer.plan.validate()?;
            if transfer.state == ReplicaTransferState::Joined
                && transfer
                    .receipt_sha256
                    .as_deref()
                    .is_none_or(|digest| !is_sha256(digest))
            {
                return Err(ClusterError::Invalid(
                    "joined replica transfer requires a receipt digest".into(),
                ));
            }
            if let Some(receipt) = &transfer.receipt_sha256 {
                if !is_sha256(receipt) {
                    return Err(ClusterError::Invalid(
                        "replica transfer receipt must be a SHA-256 digest".into(),
                    ));
                }
            }
        }
        for (key, reshard) in &self.reshards {
            validate_text(key, "reshard operation id")?;
            if key != &reshard.operation_id {
                return Err(ClusterError::Invalid(
                    "reshard authority key differs from its operation".into(),
                ));
            }
            reshard.validate()?;
        }

        let encoded = serde_json::to_vec(self).map_err(invalid_encoding)?;
        if encoded.len() > MAX_DISTRIBUTED_AUTHORITY_BYTES {
            return Err(ClusterError::Invalid(format!(
                "distributed authority exceeds {MAX_DISTRIBUTED_AUTHORITY_BYTES} encoded bytes"
            )));
        }
        Ok(())
    }

    pub fn validate_successor(&self, previous: &Self) -> Result<()> {
        self.validate()?;
        previous.validate()?;
        if self.cluster != previous.cluster
            || self.metadata_shard != previous.metadata_shard
            || self.revision != previous.revision.saturating_add(1)
            || self.updated_at <= previous.updated_at
        {
            return Err(ClusterError::Invalid(
                "distributed authority successor must retain identity and advance revision/time exactly"
                    .into(),
            ));
        }
        for (shard, old) in &previous.shards {
            if let Some(new) = self.shards.get(shard) {
                if new.placement.epoch < old.placement.epoch {
                    return Err(ClusterError::Invalid(
                        "shard placement epoch cannot move backwards".into(),
                    ));
                }
            }
        }
        for (operation, old) in &previous.transfers {
            let new = self.transfers.get(operation).ok_or_else(|| {
                ClusterError::Invalid("replica transfer evidence cannot disappear".into())
            })?;
            if old.state.terminal() && new.state != old.state
                || !old.state.terminal() && new.state.rank() < old.state.rank()
            {
                return Err(ClusterError::Invalid(
                    "replica transfer state cannot move backwards or leave a terminal state".into(),
                ));
            }
        }
        for (operation, old) in &previous.reshards {
            let new = self
                .reshards
                .get(operation)
                .ok_or_else(|| ClusterError::Invalid("reshard evidence cannot disappear".into()))?;
            if reshard_rank(new.state) < reshard_rank(old.state) {
                return Err(ClusterError::Invalid(
                    "reshard state cannot move backwards".into(),
                ));
            }
        }
        Ok(())
    }

    pub fn digest(&self) -> Result<String> {
        self.validate()?;
        serde_json::to_vec(self)
            .map(|bytes| digest::sha256_hex(&bytes))
            .map_err(invalid_encoding)
    }

    pub fn shard_for_hash(&self, table: &LogicalTableId, hash: u64) -> Result<&ShardAuthority> {
        self.validate()?;
        self.shards
            .values()
            .find(|shard| &shard.table == table && shard.range.contains(hash))
            .ok_or_else(|| ClusterError::NotFound(format!("table {table} hash {hash}")))
    }

    /// Selects a replica from current observations without inventing a
    /// cluster-wide cursor. Missing replicas are reported as lost, and every
    /// successful route carries the exact shard stamp that was selected.
    pub fn route_read(
        &self,
        table: &LogicalTableId,
        hash: u64,
        requested_consistency: ReadConsistency,
        exact_snapshot: Option<&ShardReadStamp>,
        observations: &BTreeMap<NodeId, ReplicaObservation>,
    ) -> Result<RouteEvidence> {
        let shard = self.shard_for_hash(table, hash)?;
        let roles = shard
            .placement
            .replicas
            .iter()
            .map(|replica| (replica.node.clone(), replica.role))
            .collect::<BTreeMap<_, _>>();
        for (key, observation) in observations {
            if key != &observation.node || !roles.contains_key(key) {
                return Err(ClusterError::Invalid(
                    "replica observation identity is outside the selected placement".into(),
                ));
            }
            observation.stamp.validate()?;
            if observation.stamp.placement_epoch != shard.placement.epoch {
                return Err(ClusterError::Invalid(
                    "replica observation crosses the selected placement epoch".into(),
                ));
            }
            if observation.leader && roles[key] != ReplicaRole::Voter {
                return Err(ClusterError::Invalid(
                    "a learner cannot claim shard leadership".into(),
                ));
            }
        }
        let replica_health = shard
            .placement
            .replicas
            .iter()
            .map(|replica| {
                (
                    replica.node.clone(),
                    observations
                        .get(&replica.node)
                        .map(|observation| observation.health)
                        .unwrap_or(ReplicaHealth::Lost),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut active = observations
            .values()
            .filter(|observation| observation.health == ReplicaHealth::Active)
            .collect::<Vec<_>>();
        active.sort_by(|left, right| left.node.cmp(&right.node));

        let denied = |reason: &str| -> Result<RouteEvidence> {
            let evidence = RouteEvidence {
                contract_version: CLUSTER_CONTRACT_VERSION,
                shard: shard.shard,
                placement_epoch: shard.placement.epoch,
                requested_consistency,
                selected: Vec::new(),
                replica_health: replica_health.clone(),
                observed: None,
                allowed: false,
                reason: reason.into(),
            };
            evidence.validate()?;
            Ok(evidence)
        };

        let selected = match requested_consistency {
            ReadConsistency::Linearizable => {
                let active_voters = active
                    .iter()
                    .filter(|observation| roles[&observation.node] == ReplicaRole::Voter)
                    .copied()
                    .collect::<Vec<_>>();
                if active_voters.len() < shard.placement.policy.quorum() {
                    return denied(
                        "linearizable read denied because an active voter quorum is unavailable",
                    );
                }
                let leaders = active_voters
                    .into_iter()
                    .filter(|observation| observation.leader)
                    .collect::<Vec<_>>();
                if leaders.len() != 1 {
                    return denied(
                        "linearizable read denied without exactly one active voter leader",
                    );
                }
                leaders[0]
            }
            ReadConsistency::BoundedStale { maximum_index_lag } => {
                if maximum_index_lag == 0 {
                    return Err(ClusterError::Invalid(
                        "bounded-stale reads require non-zero index lag".into(),
                    ));
                }
                let Some(head) = active
                    .iter()
                    .map(|observation| observation.stamp.commit_index)
                    .max()
                else {
                    return denied(
                        "bounded-stale read denied because no active replica was observed",
                    );
                };
                active.sort_by(|left, right| {
                    right
                        .stamp
                        .commit_index
                        .cmp(&left.stamp.commit_index)
                        .then_with(|| left.node.cmp(&right.node))
                });
                let Some(selected) = active.into_iter().find(|observation| {
                    head.saturating_sub(observation.stamp.commit_index) <= maximum_index_lag
                }) else {
                    return denied(
                        "bounded-stale read denied because every replica exceeds the lag bound",
                    );
                };
                selected
            }
            ReadConsistency::ExactSnapshot => {
                let Some(exact) = exact_snapshot else {
                    return denied(
                        "exact-snapshot read denied because no shard stamp was supplied",
                    );
                };
                exact.validate()?;
                if exact.placement_epoch != shard.placement.epoch {
                    return denied("exact-snapshot read denied across a placement epoch");
                }
                let Some(selected) = active
                    .into_iter()
                    .find(|observation| &observation.stamp == exact)
                else {
                    return denied("exact-snapshot read denied because no active replica has the requested stamp");
                };
                selected
            }
        };
        let evidence = RouteEvidence {
            contract_version: CLUSTER_CONTRACT_VERSION,
            shard: shard.shard,
            placement_epoch: shard.placement.epoch,
            requested_consistency,
            selected: vec![selected.node.clone()],
            replica_health,
            observed: Some(selected.stamp.clone()),
            allowed: true,
            reason: "selected one active replica satisfying the requested consistency".into(),
        };
        evidence.validate()?;
        Ok(evidence)
    }

    pub fn prepare_runtime_commit(
        &self,
        scope: ScopeId,
        actor: impl Into<String>,
        expected_cursor: u64,
        current_schema: Option<&RuntimeSchemaRegistry>,
    ) -> Result<rrd_core::RuntimeCommit> {
        self.validate()?;
        let actor = actor.into();
        validate_text(&actor, "distributed authority actor")?;
        let payload = serde_json::to_string(self).map_err(invalid_encoding)?;
        if payload.len() > MAX_DISTRIBUTED_AUTHORITY_BYTES {
            return Err(ClusterError::Invalid(
                "distributed authority payload exceeds its runtime bound".into(),
            ));
        }
        let kind = RuntimeType::new(DISTRIBUTED_AUTHORITY_RECORD_TYPE)
            .map_err(|error| ClusterError::Invalid(error.to_string()))?;
        let mut mutations = Vec::new();
        if let Some(registry) = authority_schema_transition(current_schema, &kind)? {
            mutations.push(RuntimeMutation::Schema { registry });
        }
        mutations.push(RuntimeMutation::Record {
            record: RuntimeRecord {
                reference: RuntimeRef::new(kind.as_str(), self.cluster.as_str())
                    .map_err(|error| ClusterError::Invalid(error.to_string()))?,
                valid_from: self.updated_at,
                valid_to: None,
                properties: RuntimeProperties::from([
                    (
                        "contract_version".into(),
                        RuntimeValue::Unsigned(u64::from(DISTRIBUTED_AUTHORITY_FORMAT)),
                    ),
                    ("revision".into(), RuntimeValue::Unsigned(self.revision)),
                    (
                        "cluster".into(),
                        RuntimeValue::String(self.cluster.to_string()),
                    ),
                    (
                        "catalogue_sha256".into(),
                        RuntimeValue::Digest(self.digest()?),
                    ),
                    ("payload".into(), RuntimeValue::String(payload)),
                    ("updated_at".into(), RuntimeValue::Unsigned(self.updated_at)),
                ]),
            },
        });
        let commit = rrd_core::RuntimeCommit {
            scope,
            at: self.updated_at,
            actor,
            expected_cursor,
            mutations,
        };
        commit
            .validate()
            .map_err(|error| ClusterError::Invalid(error.to_string()))?;
        Ok(commit)
    }

    pub fn from_runtime_snapshot(
        snapshot: &RuntimeDataSnapshot,
        cluster: &ClusterId,
    ) -> Result<Option<Self>> {
        let record = snapshot.records.iter().find(|entry| {
            entry.value.reference.kind.as_str() == DISTRIBUTED_AUTHORITY_RECORD_TYPE
                && entry.value.reference.id.as_str() == cluster.as_str()
        });
        let Some(record) = record else {
            return Ok(None);
        };
        if record.model != RuntimeLogicalModel::KeyValue {
            return Err(ClusterError::Denied(
                "distributed authority runtime record has the wrong logical model".into(),
            ));
        }
        let payload = property_string(&record.value.properties, "payload")?;
        let expected_digest = property_digest(&record.value.properties, "catalogue_sha256")?;
        let catalogue: Self = serde_json::from_str(payload).map_err(|error| {
            ClusterError::Invalid(format!(
                "distributed authority payload decode failed: {error}"
            ))
        })?;
        catalogue.validate()?;
        if &catalogue.cluster != cluster
            || catalogue.digest()? != expected_digest
            || property_unsigned(&record.value.properties, "contract_version")?
                != u64::from(DISTRIBUTED_AUTHORITY_FORMAT)
            || property_unsigned(&record.value.properties, "revision")? != catalogue.revision
            || property_unsigned(&record.value.properties, "updated_at")? != catalogue.updated_at
            || property_string(&record.value.properties, "cluster")? != cluster.as_str()
        {
            return Err(ClusterError::Denied(
                "distributed authority runtime envelope disagrees with its payload".into(),
            ));
        }
        Ok(Some(catalogue))
    }
}

pub fn plan_distributed_authority(
    cluster: ClusterId,
    revision: u64,
    updated_at: u64,
    nodes: Vec<PlacementNode>,
    tenants: Vec<TenantPlacementPolicy>,
    tables: Vec<LogicalTablePolicy>,
) -> Result<DistributedAuthorityCatalogue> {
    let mut node_map = BTreeMap::new();
    for node in nodes {
        let key = node.node.clone();
        if node_map.insert(key, node).is_some() {
            return Err(ClusterError::Invalid(
                "placement input contains a duplicate node".into(),
            ));
        }
    }
    let mut tenant_map = BTreeMap::new();
    for tenant in tenants {
        let key = tenant.tenant.clone();
        if tenant_map.insert(key, tenant).is_some() {
            return Err(ClusterError::Invalid(
                "placement input contains a duplicate tenant".into(),
            ));
        }
    }
    let mut table_map = BTreeMap::new();
    for table in tables {
        let key = table.table.clone();
        if table_map.insert(key, table).is_some() {
            return Err(ClusterError::Invalid(
                "placement input contains a duplicate logical table".into(),
            ));
        }
    }

    let mut shards = BTreeMap::new();
    let mut voter_load = BTreeMap::<NodeId, u32>::new();
    let mut learner_load = BTreeMap::<NodeId, u32>::new();
    for table in table_map.values() {
        let tenant = tenant_map
            .get(&table.tenant)
            .ok_or_else(|| ClusterError::Invalid("logical table names an unknown tenant".into()))?;
        for ordinal in 0..table.shard_count {
            let shard_value = table
                .first_shard
                .0
                .checked_add(u64::from(ordinal))
                .ok_or_else(|| ClusterError::Invalid("shard id allocation overflowed".into()))?;
            let shard = ShardId(shard_value);
            if shard == METADATA_SHARD_ID || shards.contains_key(&shard) {
                return Err(ClusterError::Invalid(
                    "logical table shard allocation overlaps metadata or another table".into(),
                ));
            }
            let replicas = select_replicas(&node_map, tenant, &mut voter_load, &mut learner_load)?;
            let placement = ShardPlacement {
                contract_version: CLUSTER_CONTRACT_VERSION,
                cluster: cluster.clone(),
                shard,
                epoch: 1,
                policy: tenant.placement.clone(),
                replicas,
            };
            placement.validate()?;
            let replica_health = placement
                .replicas
                .iter()
                .map(|replica| (replica.node.clone(), ReplicaHealth::Active))
                .collect();
            shards.insert(
                shard,
                ShardAuthority {
                    shard,
                    table: table.table.clone(),
                    range: hash_range(ordinal, table.shard_count),
                    placement,
                    replica_health,
                },
            );
        }
    }

    let catalogue = DistributedAuthorityCatalogue {
        format: DISTRIBUTED_AUTHORITY_FORMAT,
        cluster,
        metadata_shard: METADATA_SHARD_ID,
        revision,
        updated_at,
        nodes: node_map,
        tenants: tenant_map,
        tables: table_map,
        shards,
        transfers: BTreeMap::new(),
        reshards: BTreeMap::new(),
    };
    catalogue.validate()?;
    Ok(catalogue)
}

fn select_replicas(
    nodes: &BTreeMap<NodeId, PlacementNode>,
    tenant: &TenantPlacementPolicy,
    voter_load: &mut BTreeMap<NodeId, u32>,
    learner_load: &mut BTreeMap<NodeId, u32>,
) -> Result<Vec<ReplicaPlacement>> {
    let candidates = nodes
        .values()
        .filter(|node| {
            node.state == PlacementNodeState::Active
                && tenant.allowed_regions.contains(&node.region)
        })
        .collect::<Vec<_>>();
    let mut selected = BTreeSet::<NodeId>::new();
    let mut voter_regions = BTreeSet::<RegionId>::new();
    let mut voter_zones = BTreeMap::<ZoneId, u8>::new();
    let mut replicas = Vec::new();

    while replicas.len() < usize::from(tenant.placement.voter_count) {
        let need_region = voter_regions.len() < usize::from(tenant.minimum_voter_regions);
        let need_zone = voter_zones.len() < usize::from(tenant.placement.minimum_voter_zones);
        let mut eligible = candidates
            .iter()
            .copied()
            .filter(|node| !selected.contains(&node.node))
            .filter(|node| {
                voter_load.get(&node.node).copied().unwrap_or(0) < node.voter_capacity
                    && voter_zones.get(&node.zone).copied().unwrap_or(0)
                        < tenant.placement.maximum_voters_per_zone
            })
            .collect::<Vec<_>>();
        eligible.sort_by_key(|node| {
            (
                need_region && voter_regions.contains(&node.region),
                need_zone && voter_zones.contains_key(&node.zone),
                voter_load.get(&node.node).copied().unwrap_or(0),
                voter_zones.get(&node.zone).copied().unwrap_or(0),
                node.node.clone(),
            )
        });
        let node = eligible.first().copied().ok_or_else(|| {
            ClusterError::Unavailable(
                "active node capacity cannot satisfy voter region/zone placement".into(),
            )
        })?;
        selected.insert(node.node.clone());
        voter_regions.insert(node.region.clone());
        *voter_zones.entry(node.zone.clone()).or_default() += 1;
        *voter_load.entry(node.node.clone()).or_default() += 1;
        replicas.push(ReplicaPlacement {
            node: node.node.clone(),
            zone: node.zone.clone(),
            role: ReplicaRole::Voter,
        });
    }
    if voter_regions.len() < usize::from(tenant.minimum_voter_regions) {
        return Err(ClusterError::Unavailable(
            "selected voters cannot satisfy tenant region diversity".into(),
        ));
    }

    for _ in 0..tenant.read_replica_count {
        let mut eligible = candidates
            .iter()
            .copied()
            .filter(|node| !selected.contains(&node.node))
            .filter(|node| {
                learner_load.get(&node.node).copied().unwrap_or(0) < node.learner_capacity
            })
            .collect::<Vec<_>>();
        eligible.sort_by_key(|node| {
            (
                learner_load.get(&node.node).copied().unwrap_or(0),
                node.node.clone(),
            )
        });
        let node = eligible.first().copied().ok_or_else(|| {
            ClusterError::Unavailable(
                "active node capacity cannot satisfy requested read replicas".into(),
            )
        })?;
        selected.insert(node.node.clone());
        *learner_load.entry(node.node.clone()).or_default() += 1;
        replicas.push(ReplicaPlacement {
            node: node.node.clone(),
            zone: node.zone.clone(),
            role: ReplicaRole::Learner,
        });
    }
    replicas.sort_by(|left, right| left.node.cmp(&right.node));
    Ok(replicas)
}

fn hash_range(ordinal: u16, count: u16) -> ShardHashRange {
    let space = 1_u128 << 64;
    let start = u128::from(ordinal) * space / u128::from(count);
    let end_exclusive = (u128::from(ordinal) + 1) * space / u128::from(count);
    ShardHashRange {
        start_inclusive: start as u64,
        end_inclusive: (end_exclusive - 1) as u64,
    }
}

fn authority_schema_transition(
    current: Option<&RuntimeSchemaRegistry>,
    kind: &RuntimeType,
) -> Result<Option<RuntimeSchemaRegistry>> {
    let record_schema = authority_record_schema();
    let table_schema = RuntimeTableSchema::strict(RuntimeLogicalModel::KeyValue);
    if let Some(current) = current {
        let tables = current
            .catalogue_tables()
            .map_err(|error| ClusterError::Invalid(error.to_string()))?;
        match (tables.get(kind), current.records.get(kind)) {
            (Some(table), Some(record)) if table == &table_schema && record == &record_schema => {
                return Ok(None);
            }
            (Some(_), _) | (_, Some(_)) => {
                return Err(ClusterError::Denied(
                    "distributed authority runtime type collides with another schema".into(),
                ));
            }
            (None, None) => {}
        }
        let mut next = current.clone();
        next.revision = next
            .revision
            .checked_add(1)
            .ok_or_else(|| ClusterError::Invalid("runtime schema revision overflowed".into()))?;
        next.migration = "install distributed metadata-shard authority".into();
        next.tables = tables;
        next.define_record_table(kind.clone(), RuntimeLogicalModel::KeyValue, record_schema)
            .map_err(|error| ClusterError::Invalid(error.to_string()))?;
        next.validate()
            .map_err(|error| ClusterError::Invalid(error.to_string()))?;
        return Ok(Some(next));
    }

    let mut registry =
        RuntimeSchemaRegistry::empty(1, "install distributed metadata-shard authority");
    registry
        .define_record_table(kind.clone(), RuntimeLogicalModel::KeyValue, record_schema)
        .map_err(|error| ClusterError::Invalid(error.to_string()))?;
    registry
        .validate()
        .map_err(|error| ClusterError::Invalid(error.to_string()))?;
    Ok(Some(registry))
}

fn authority_record_schema() -> RuntimeRecordSchema {
    RuntimeRecordSchema {
        properties: BTreeMap::from([
            (
                "catalogue_sha256".into(),
                RuntimePropertySchema::required(RuntimeValueType::Digest),
            ),
            (
                "cluster".into(),
                RuntimePropertySchema::required(RuntimeValueType::String),
            ),
            (
                "contract_version".into(),
                RuntimePropertySchema::required(RuntimeValueType::Unsigned),
            ),
            (
                "payload".into(),
                RuntimePropertySchema::required(RuntimeValueType::String),
            ),
            (
                "revision".into(),
                RuntimePropertySchema::required(RuntimeValueType::Unsigned),
            ),
            (
                "updated_at".into(),
                RuntimePropertySchema::required(RuntimeValueType::Unsigned),
            ),
        ]),
        allow_additional_properties: false,
        unique_properties: BTreeSet::new(),
    }
}

fn property_string<'a>(properties: &'a RuntimeProperties, key: &str) -> Result<&'a str> {
    match properties.get(key) {
        Some(RuntimeValue::String(value)) => Ok(value),
        _ => Err(ClusterError::Denied(format!(
            "distributed authority runtime property {key} has the wrong type"
        ))),
    }
}

fn property_digest<'a>(properties: &'a RuntimeProperties, key: &str) -> Result<&'a str> {
    match properties.get(key) {
        Some(RuntimeValue::Digest(value)) if is_sha256(value) => Ok(value),
        _ => Err(ClusterError::Denied(format!(
            "distributed authority runtime property {key} is not a SHA-256 digest"
        ))),
    }
}

fn property_unsigned(properties: &RuntimeProperties, key: &str) -> Result<u64> {
    match properties.get(key) {
        Some(RuntimeValue::Unsigned(value)) => Ok(*value),
        _ => Err(ClusterError::Denied(format!(
            "distributed authority runtime property {key} has the wrong type"
        ))),
    }
}

fn validate_text(value: &str, label: &str) -> Result<()> {
    if value.is_empty() || value.len() > 256 || value.as_bytes().contains(&0) {
        return Err(ClusterError::Invalid(format!(
            "{label} must contain 1..=256 non-NUL bytes"
        )));
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn reshard_rank(state: ReshardState) -> u8 {
    match state {
        ReshardState::Planned => 0,
        ReshardState::Copying => 1,
        ReshardState::CaughtUp => 2,
        ReshardState::Cutover => 3,
        ReshardState::Retired => 4,
    }
}

fn invalid_encoding(error: serde_json::Error) -> ClusterError {
    ClusterError::Invalid(format!("distributed authority encoding failed: {error}"))
}
