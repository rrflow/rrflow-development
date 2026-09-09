use crate::access::runtime_state::{checked_key, get};
use crate::keyspaces::{self, Durability};
use crate::projection::{
    difference, CurrentProjection, GroundedStamp, GroundingReport, ProjectionStatus,
    RebuildOutcome, CURRENT_PROJECTION,
};
use crate::{Error, Result, StorageEngine};
use rrd_core::Millis;

/// Derived claim projections. Authoritative claims are always read through
/// [`crate::ClaimRepository`]; projection bytes never become source truth.
pub struct ProjectionRepository<'a> {
    storage: &'a dyn StorageEngine,
}

impl<'a> ProjectionRepository<'a> {
    pub(crate) const fn new(storage: &'a dyn StorageEngine) -> Self {
        Self { storage }
    }

    pub fn get(&self, name: &str) -> Result<Option<Vec<u8>>> {
        let transaction = self.storage.begin_transaction()?;
        get(
            &*transaction,
            keyspaces::PROJECTIONS,
            &keyspaces::projection_key(name),
        )
    }

    pub fn put_with(&self, name: &str, bytes: &[u8], durability: Durability) -> Result<()> {
        let mut transaction = self.storage.begin_transaction()?;
        transaction.put(
            checked_key(keyspaces::PROJECTIONS, &keyspaces::projection_key(name))?,
            bytes.to_vec(),
        )?;
        transaction.commit(durability)?;
        Ok(())
    }

    pub fn put(&self, name: &str, bytes: &[u8]) -> Result<()> {
        self.put_with(name, bytes, Durability::Buffered)
    }

    pub fn current(&self) -> Result<CurrentProjection> {
        match self.get(CURRENT_PROJECTION)? {
            Some(bytes) => CurrentProjection::from_stored_bytes(&bytes),
            None => Ok(CurrentProjection::empty()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn rebuild_current(&self) -> Result<RebuildOutcome> {
        let mut projection = self.current()?;
        if let ProjectionStatus::Quarantined { at, .. } = &projection.status {
            return Err(Error::Quarantined(format!(
                "projection `{CURRENT_PROJECTION}` quarantined at {at}; reset to recover"
            )));
        }
        let from = projection.watermark;
        let claims = self.storage.claims();
        let to = claims.sequence()?;
        let interval = claims.claims_in_range(from, to)?;
        let applied = interval.len();
        projection.apply(&interval);
        projection.watermark = to;
        self.put_with(
            CURRENT_PROJECTION,
            &projection.to_stored_bytes()?,
            Durability::Buffered,
        )?;
        tracing::debug!(from, to, applied, "rebuild advanced the watermark");
        Ok(RebuildOutcome { from, to, applied })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn ground_current(&self, at: Millis) -> Result<GroundingReport> {
        let mut projection = self.current()?;
        if let ProjectionStatus::Quarantined { at, .. } = &projection.status {
            return Err(Error::Quarantined(format!(
                "projection `{CURRENT_PROJECTION}` quarantined at {at}; reset to recover"
            )));
        }
        let mut recomputed = CurrentProjection::empty();
        recomputed.apply(
            &self
                .storage
                .claims()
                .claims_in_range(0, projection.watermark)?,
        );
        let differences = difference(recomputed.entries(), projection.entries());
        if differences.is_empty() {
            let stamp = GroundedStamp {
                at,
                sequence: projection.watermark,
                digest: projection.digest()?,
            };
            projection.last_grounded = Some(stamp);
            self.put_with(
                CURRENT_PROJECTION,
                &projection.to_stored_bytes()?,
                Durability::Buffered,
            )?;
            tracing::debug!(sequence = stamp.sequence, digest = stamp.digest, "grounded");
            return Ok(GroundingReport::Grounded(stamp));
        }
        projection.status = ProjectionStatus::Quarantined {
            at,
            differences: differences.clone(),
        };
        self.put_with(
            CURRENT_PROJECTION,
            &projection.to_stored_bytes()?,
            Durability::Authoritative,
        )?;
        tracing::warn!(
            differences = differences.len(),
            "divergence — projection quarantined"
        );
        Ok(GroundingReport::Divergence { differences })
    }

    pub fn reset_current(&self) -> Result<RebuildOutcome> {
        let claims = self.storage.claims();
        let to = claims.sequence()?;
        let mut projection = CurrentProjection::empty();
        let interval = claims.claims_in_range(0, to)?;
        let applied = interval.len();
        projection.apply(&interval);
        projection.watermark = to;
        self.put_with(
            CURRENT_PROJECTION,
            &projection.to_stored_bytes()?,
            Durability::Buffered,
        )?;
        Ok(RebuildOutcome {
            from: 0,
            to,
            applied,
        })
    }
}
