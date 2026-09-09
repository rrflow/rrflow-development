use super::common::{checked_key, read_sequence, scan_space_from};
use crate::invocation::{Invocation, InvocationInput};
use crate::keyspaces::{self, Durability};
use crate::{Error, Result, StorageEngine};
use rrd_core::Millis;

/// Durable operator-invocation receipts over the shared transaction port.
pub struct InvocationRepository<'a> {
    storage: &'a dyn StorageEngine,
}

impl<'a> InvocationRepository<'a> {
    pub(crate) const fn new(storage: &'a dyn StorageEngine) -> Self {
        Self { storage }
    }

    #[tracing::instrument(level = "debug", skip_all, fields(command = input.command))]
    pub fn record(&self, input: InvocationInput<'_>) -> Result<Invocation> {
        let mut transaction = self.storage.begin_transaction()?;
        let previous = read_sequence(&*transaction, &keyspaces::invocation_watermark_key())?;
        let ordinal = previous.checked_add(1).ok_or(Error::SequenceOverflow)?;
        let record = Invocation {
            ordinal,
            at: input.at,
            trigger: input.trigger,
            command: input.command.to_owned(),
            arguments: input.arguments.to_vec(),
            outcome: input.outcome,
            duration_ms: input.duration_ms,
            detail: input.detail,
        };
        transaction.put(
            checked_key(
                keyspaces::INVOCATIONS,
                &keyspaces::invocation_key(input.at, ordinal),
            )?,
            serde_json::to_vec(&record)?,
        )?;
        super::common::put_sequence(
            &mut *transaction,
            &keyspaces::invocation_watermark_key(),
            ordinal,
        )?;
        transaction.commit(Durability::Authoritative)?;
        tracing::debug!(ordinal, "invocation recorded");
        Ok(record)
    }

    pub fn since(&self, since: Millis) -> Result<Vec<Invocation>> {
        let transaction = self.storage.begin_transaction()?;
        scan_space_from(
            &*transaction,
            keyspaces::INVOCATIONS,
            &keyspaces::invocation_bound(since),
        )?
        .into_iter()
        .map(|(_, value)| serde_json::from_slice(&value).map_err(Error::from))
        .collect()
    }

    pub fn count(&self) -> Result<u64> {
        let transaction = self.storage.begin_transaction()?;
        read_sequence(&*transaction, &keyspaces::invocation_watermark_key())
    }
}
