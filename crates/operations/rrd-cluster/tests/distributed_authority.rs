use rrd_cluster::{
    plan_distributed_authority, ClusterError, ClusterId, DistributedAuthorityCatalogue,
    LogicalTableId, LogicalTablePolicy, PlacementNode, PlacementNodeState, PlacementPolicy,
    ReadConsistency, RegionId, ReplicaRole, ShardId, TenantId, TenantPlacementPolicy,
    WriteConsistency, ZoneId,
};
use rrd_core::ScopeId;
use rrd_store::{RrflowKvStore, StorageEngine};
use std::collections::BTreeSet;

fn node(index: u8) -> PlacementNode {
    let region = (index - 1) % 3 + 1;
    PlacementNode {
        node: rrd_cluster::NodeId::new(format!("node-{index}")).unwrap(),
        region: RegionId::new(format!("region-{region}")).unwrap(),
        zone: ZoneId::new(format!("zone-{index}")).unwrap(),
        state: PlacementNodeState::Active,
        voter_capacity: 32,
        learner_capacity: 32,
        observed_at: 100,
    }
}

fn tenant() -> TenantPlacementPolicy {
    TenantPlacementPolicy {
        tenant: TenantId::new("tenant-a").unwrap(),
        allowed_regions: ["region-1", "region-2", "region-3"]
            .into_iter()
            .map(|value| RegionId::new(value).unwrap())
            .collect(),
        minimum_voter_regions: 3,
        placement: PlacementPolicy {
            voter_count: 3,
            minimum_voter_zones: 3,
            maximum_voters_per_zone: 1,
        },
        read_replica_count: 2,
    }
}

fn table() -> LogicalTablePolicy {
    LogicalTablePolicy {
        table: LogicalTableId::new("orders").unwrap(),
        tenant: TenantId::new("tenant-a").unwrap(),
        partition_key: "tenant_id/order_id".into(),
        shard_count: 4,
        first_shard: ShardId(10),
        default_read: ReadConsistency::BoundedStale {
            maximum_index_lag: 32,
        },
        write: WriteConsistency::QuorumDurable,
    }
}

fn catalogue() -> DistributedAuthorityCatalogue {
    plan_distributed_authority(
        ClusterId::new("cluster-a").unwrap(),
        1,
        100,
        (1..=6).map(node).collect(),
        vec![tenant()],
        vec![table()],
    )
    .unwrap()
}

#[test]
fn automatic_hash_sharding_and_multi_region_placement_are_deterministic() {
    let first = catalogue();
    let mut reversed_nodes = (1..=6).map(node).collect::<Vec<_>>();
    reversed_nodes.reverse();
    let second = plan_distributed_authority(
        ClusterId::new("cluster-a").unwrap(),
        1,
        100,
        reversed_nodes,
        vec![tenant()],
        vec![table()],
    )
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(first.shards.len(), 4);
    assert_eq!(
        first
            .shards
            .values()
            .map(|shard| shard.range)
            .collect::<Vec<_>>(),
        vec![
            rrd_cluster::ShardHashRange {
                start_inclusive: 0,
                end_inclusive: 4_611_686_018_427_387_903,
            },
            rrd_cluster::ShardHashRange {
                start_inclusive: 4_611_686_018_427_387_904,
                end_inclusive: 9_223_372_036_854_775_807,
            },
            rrd_cluster::ShardHashRange {
                start_inclusive: 9_223_372_036_854_775_808,
                end_inclusive: 13_835_058_055_282_163_711,
            },
            rrd_cluster::ShardHashRange {
                start_inclusive: 13_835_058_055_282_163_712,
                end_inclusive: u64::MAX,
            },
        ]
    );
    for shard in first.shards.values() {
        assert_eq!(
            shard
                .placement
                .replicas
                .iter()
                .filter(|replica| replica.role == ReplicaRole::Voter)
                .count(),
            3
        );
        assert_eq!(
            shard
                .placement
                .replicas
                .iter()
                .filter(|replica| replica.role == ReplicaRole::Learner)
                .count(),
            2
        );
        let voter_regions = shard
            .placement
            .voters()
            .map(|replica| first.nodes[&replica.node].region.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(voter_regions.len(), 3);
    }
    assert_eq!(
        first
            .shard_for_hash(&LogicalTableId::new("orders").unwrap(), 0)
            .unwrap()
            .shard,
        ShardId(10)
    );
    assert_eq!(
        first
            .shard_for_hash(&LogicalTableId::new("orders").unwrap(), u64::MAX)
            .unwrap()
            .shard,
        ShardId(13)
    );
}

#[test]
fn capacity_and_region_failures_are_denied_without_partial_authority() {
    let mut nodes = (1..=6).map(node).collect::<Vec<_>>();
    for node in &mut nodes {
        if node.region.as_str() == "region-3" {
            node.state = PlacementNodeState::Unavailable;
        }
    }
    let error = plan_distributed_authority(
        ClusterId::new("cluster-a").unwrap(),
        1,
        100,
        nodes,
        vec![tenant()],
        vec![table()],
    )
    .unwrap_err();
    assert!(matches!(error, ClusterError::Unavailable(_)));
}

#[test]
fn metadata_catalogue_is_one_schema_governed_rrd_record_and_survives_reopen() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().join("metadata");
    let scope = ScopeId::new("instance:cluster-metadata").unwrap();
    let cluster = ClusterId::new("cluster-a").unwrap();
    let first = catalogue();

    {
        let engine = RrflowKvStore::open(&root).unwrap();
        let read = engine.runtime().read_stamp(&scope).unwrap();
        let commit = first
            .prepare_runtime_commit(
                scope.clone(),
                "metadata-controller",
                read.commit_cursor,
                None,
            )
            .unwrap();
        assert_eq!(commit.mutations.len(), 2);
        engine.runtime().commit(&commit).unwrap();
        let (_, snapshot) = engine.runtime().data_snapshot(&scope, 100, 10_000).unwrap();
        assert_eq!(
            DistributedAuthorityCatalogue::from_runtime_snapshot(&snapshot, &cluster)
                .unwrap()
                .unwrap(),
            first
        );

        let mut second = first.clone();
        second.revision = 2;
        second.updated_at = 200;
        second.validate_successor(&first).unwrap();
        let read = engine.runtime().read_stamp(&scope).unwrap();
        let schema = engine.runtime().schema(&scope).unwrap().unwrap();
        let commit = second
            .prepare_runtime_commit(
                scope.clone(),
                "metadata-controller",
                read.commit_cursor,
                Some(&schema),
            )
            .unwrap();
        assert_eq!(commit.mutations.len(), 1);
        engine.runtime().commit(&commit).unwrap();
    }

    let reopened = RrflowKvStore::open(&root).unwrap();
    let (_, snapshot) = reopened
        .runtime()
        .data_snapshot(&scope, 200, 10_000)
        .unwrap();
    let restored = DistributedAuthorityCatalogue::from_runtime_snapshot(&snapshot, &cluster)
        .unwrap()
        .unwrap();
    assert_eq!(restored.revision, 2);
    assert_eq!(restored.updated_at, 200);
    assert_eq!(restored.shards.len(), 4);
    assert_eq!(restored.digest().unwrap(), {
        let mut expected = first;
        expected.revision = 2;
        expected.updated_at = 200;
        expected.digest().unwrap()
    });
}
