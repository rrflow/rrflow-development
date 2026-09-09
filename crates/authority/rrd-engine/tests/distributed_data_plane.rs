use rrd_cluster::{
    plan_distributed_authority, ClusterId, DistributedAuthorityCatalogue, LogicalTableId,
    LogicalTablePolicy, NodeId, PlacementNode, PlacementNodeState, PlacementPolicy,
    ReadConsistency, RegionId, ReplicaHealth, ReplicaObservation, ReplicaRole, ShardId,
    ShardReadStamp, TenantId, TenantPlacementPolicy, WriteConsistency, ZoneId,
};
use rrd_contract::CanonicalId;
use rrd_core::digest;
use rrd_engine::RrdEngine;
use rrd_store::{RrflowKvStore, StorageEngine};
use std::collections::{BTreeMap, BTreeSet};

fn node(index: u8) -> PlacementNode {
    let region = (index - 1) % 3 + 1;
    PlacementNode {
        node: NodeId::new(format!("node-{index}")).unwrap(),
        region: RegionId::new(format!("region-{region}")).unwrap(),
        zone: ZoneId::new(format!("zone-{index}")).unwrap(),
        state: PlacementNodeState::Active,
        voter_capacity: 32,
        learner_capacity: 32,
        observed_at: 100,
    }
}

fn catalogue() -> DistributedAuthorityCatalogue {
    plan_distributed_authority(
        ClusterId::new("cluster-a").unwrap(),
        1,
        100,
        (1..=6).map(node).collect(),
        vec![TenantPlacementPolicy {
            tenant: TenantId::new("tenant-a").unwrap(),
            allowed_regions: ["region-1", "region-2", "region-3"]
                .into_iter()
                .map(|value| RegionId::new(value).unwrap())
                .collect::<BTreeSet<_>>(),
            minimum_voter_regions: 3,
            placement: PlacementPolicy {
                voter_count: 3,
                minimum_voter_zones: 3,
                maximum_voters_per_zone: 1,
            },
            read_replica_count: 2,
        }],
        vec![LogicalTablePolicy {
            table: LogicalTableId::new("orders").unwrap(),
            tenant: TenantId::new("tenant-a").unwrap(),
            partition_key: "tenant_id/order_id".into(),
            shard_count: 4,
            first_shard: ShardId(10),
            default_read: ReadConsistency::BoundedStale {
                maximum_index_lag: 32,
            },
            write: WriteConsistency::QuorumDurable,
        }],
    )
    .unwrap()
}

fn stamp(index: u64, epoch: u64, node: &NodeId) -> ShardReadStamp {
    ShardReadStamp {
        term: 3,
        commit_index: index,
        placement_epoch: epoch,
        state_digest: digest::sha256_hex(format!("{node}:{index}").as_bytes()),
    }
}

#[test]
fn metadata_commit_and_consistency_routes_share_one_engine_authority() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().join("engine");
    let instance = CanonicalId::new("cluster-metadata").unwrap();
    let cluster = ClusterId::new("cluster-a").unwrap();
    let table = LogicalTableId::new("orders").unwrap();
    let first = catalogue();

    let engine = RrdEngine::open(&root, instance.clone(), [7_u8; 32]).unwrap();
    let prepared = engine
        .prepare_distributed_authority(&first, "metadata-controller", 10_000)
        .unwrap();
    assert_eq!(prepared.metadata_shard, ShardId(0));
    assert_eq!(prepared.catalogue_sha256, first.digest().unwrap());
    assert_eq!(prepared.commit.expected_cursor, prepared.read.commit_cursor);
    #[cfg(feature = "cluster-transfer")]
    {
        let command = prepared
            .raft_command("metadata-revision-1", 1, None)
            .unwrap();
        assert_eq!(command.shard, ShardId(0));
        assert!(matches!(
            command.operation,
            rrd_cluster::RrdRaftOperation::RuntimeCommit { .. }
        ));
    }
    drop(engine);

    {
        let store = RrflowKvStore::open(&root).unwrap();
        store.runtime().commit(&prepared.commit).unwrap();
    }

    let engine = RrdEngine::open(&root, instance, [7_u8; 32]).unwrap();
    let restored = engine
        .read_distributed_authority(&cluster, 100, 10_000)
        .unwrap()
        .unwrap();
    assert_eq!(restored.catalogue, first);
    assert_eq!(
        restored.read.commit_cursor,
        prepared.commit.mutations.len() as u64
    );

    let shard = first.shard_for_hash(&table, 0).unwrap();
    let leader = shard
        .placement
        .replicas
        .iter()
        .find(|replica| replica.role == ReplicaRole::Voter)
        .unwrap()
        .node
        .clone();
    let learner = shard
        .placement
        .replicas
        .iter()
        .find(|replica| replica.role == ReplicaRole::Learner)
        .unwrap()
        .node
        .clone();
    let mut observations = shard
        .placement
        .replicas
        .iter()
        .enumerate()
        .map(|(ordinal, replica)| {
            let index = if replica.node == learner {
                120
            } else {
                100 + ordinal as u64
            };
            (
                replica.node.clone(),
                ReplicaObservation {
                    node: replica.node.clone(),
                    health: ReplicaHealth::Active,
                    stamp: stamp(index, shard.placement.epoch, &replica.node),
                    leader: replica.node == leader,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    let linearizable = engine
        .route_distributed_read(
            &cluster,
            &table,
            0,
            ReadConsistency::Linearizable,
            None,
            &observations,
            100,
            10_000,
        )
        .unwrap();
    assert!(linearizable.evidence.allowed);
    assert_eq!(linearizable.evidence.selected, vec![leader.clone()]);
    assert_eq!(linearizable.catalogue_revision, 1);

    let bounded = engine
        .route_distributed_read(
            &cluster,
            &table,
            0,
            ReadConsistency::BoundedStale {
                maximum_index_lag: 4,
            },
            None,
            &observations,
            100,
            10_000,
        )
        .unwrap();
    assert!(bounded.evidence.allowed);
    assert_eq!(bounded.evidence.selected, vec![learner]);

    let lost_voter = shard
        .placement
        .replicas
        .iter()
        .find(|replica| replica.role == ReplicaRole::Voter && replica.node != leader)
        .unwrap()
        .node
        .clone();
    let lost = observations.remove(&lost_voter).unwrap();
    let replica_loss = engine
        .route_distributed_read(
            &cluster,
            &table,
            0,
            ReadConsistency::Linearizable,
            None,
            &observations,
            100,
            10_000,
        )
        .unwrap();
    assert!(replica_loss.evidence.allowed);
    observations.insert(lost_voter, lost);

    let leader_region = first.nodes[&leader].region.clone();
    observations.retain(|node, _| first.nodes[node].region != leader_region);
    let region_loss = engine
        .route_distributed_read(
            &cluster,
            &table,
            0,
            ReadConsistency::Linearizable,
            None,
            &observations,
            100,
            10_000,
        )
        .unwrap();
    assert!(!region_loss.evidence.allowed);

    let survivor = observations
        .values()
        .find(|observation| {
            shard.placement.replicas.iter().any(|replica| {
                replica.node == observation.node && replica.role == ReplicaRole::Voter
            })
        })
        .unwrap()
        .clone();
    let exact = engine
        .route_distributed_read(
            &cluster,
            &table,
            0,
            ReadConsistency::ExactSnapshot,
            Some(&survivor.stamp),
            &observations,
            100,
            10_000,
        )
        .unwrap();
    assert!(exact.evidence.allowed);
    assert_eq!(exact.evidence.selected, vec![survivor.node.clone()]);

    for observation in observations.values_mut() {
        observation.leader = observation.node == survivor.node;
    }
    let recovered = engine
        .route_distributed_read(
            &cluster,
            &table,
            0,
            ReadConsistency::Linearizable,
            None,
            &observations,
            100,
            10_000,
        )
        .unwrap();
    assert!(recovered.evidence.allowed);
    assert_eq!(recovered.evidence.selected, vec![survivor.node]);

    assert!(engine
        .prepare_distributed_authority(&first, "metadata-controller", 10_000)
        .is_err());
    let mut second = first;
    second.revision = 2;
    second.updated_at = 200;
    let successor = engine
        .prepare_distributed_authority(&second, "metadata-controller", 10_000)
        .unwrap();
    assert_eq!(successor.commit.mutations.len(), 1);
}
