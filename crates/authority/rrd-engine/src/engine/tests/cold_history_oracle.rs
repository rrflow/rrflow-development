use super::RrdEngine;
use rrd_core::{ReadStamp, RuntimeChange};
use rrd_store::StorageEngine;

pub(in crate::engine) fn expected_changes(
    engine: &RrdEngine,
    stamp: &ReadStamp,
    after: u64,
    limit: usize,
) -> Vec<RuntimeChange> {
    engine
        .storage
        .runtime()
        .read_changes(stamp, after, limit)
        .unwrap()
        .changes
}
