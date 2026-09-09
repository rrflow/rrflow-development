use crate::access::runtime_state::{
    checked_key, get, put_sequence, read_sequence, scan_space, scan_space_from,
};
use crate::gc::{build_report, RemovalReport, Tally};
use crate::key_codec::{prefix_end, KeyCodec};
use crate::keyspaces::{self, Durability};
use crate::{AppendOutcome, Error, IdempotentAppendOutcome, Result, StorageEngine};
use rrd_core::{resolve_as_of, Claim, ClaimSource, Millis, Predicate, Reader, Subject};
use std::collections::BTreeMap;

/// Temporal-claim and access-observation repository shared byte-for-byte by
/// rrflowMX and rrflowKV.
pub struct ClaimRepository<'a> {
    storage: &'a dyn StorageEngine,
}

impl<'a> ClaimRepository<'a> {
    pub(crate) const fn new(storage: &'a dyn StorageEngine) -> Self {
        Self { storage }
    }

    pub fn append_batch(&self, claims: &[Claim]) -> Result<AppendOutcome> {
        for claim in claims {
            claim.validate()?;
        }
        let mut transaction = self.storage.begin_transaction()?;
        let start = read_sequence(&*transaction, &keyspaces::sequence_watermark_key())?;
        if claims.is_empty() {
            return Ok(AppendOutcome {
                first_sequence: start,
                last_sequence: start,
                count: 0,
            });
        }
        let mut sequence = start;
        for claim in claims {
            sequence = sequence.checked_add(1).ok_or(Error::SequenceOverflow)?;
            let claim_key = keyspaces::claim_key(
                &claim.subject,
                &claim.predicate,
                claim.valid_from,
                claim.tx_time,
            );
            transaction.put(
                checked_key(keyspaces::CLAIMS, &claim_key)?,
                serde_json::to_vec(claim)?,
            )?;
            transaction.put(
                checked_key(
                    keyspaces::SEQUENCE_INDEX,
                    &keyspaces::sequence_key(sequence),
                )?,
                claim_key,
            )?;
        }
        put_sequence(
            &mut *transaction,
            &keyspaces::sequence_watermark_key(),
            sequence,
        )?;
        transaction.commit(Durability::Authoritative)?;
        Ok(AppendOutcome {
            first_sequence: start + 1,
            last_sequence: sequence,
            count: claims.len(),
        })
    }

    pub fn append_batch_idempotent(
        &self,
        idempotency_key: &str,
        operation_sha256: &str,
        claims: &[Claim],
    ) -> Result<IdempotentAppendOutcome> {
        crate::engine::validate_idempotency(idempotency_key, operation_sha256)?;
        if claims.is_empty() {
            return Err(Error::Substrate(
                "idempotent claim append must not be empty".into(),
            ));
        }
        for claim in claims {
            claim.validate()?;
        }
        let mut transaction = self.storage.begin_transaction()?;
        let receipt_key = keyspaces::accepted_append_key(idempotency_key);
        if let Some(bytes) = get(&*transaction, keyspaces::META, &receipt_key)? {
            let mut outcome: IdempotentAppendOutcome = serde_json::from_slice(&bytes)?;
            if outcome.operation_sha256 != operation_sha256 {
                return Err(Error::IdempotencyConflict(idempotency_key.into()));
            }
            outcome.idempotent_replay = true;
            return Ok(outcome);
        }
        let start = read_sequence(&*transaction, &keyspaces::sequence_watermark_key())?;
        let mut sequence = start;
        for claim in claims {
            sequence = sequence.checked_add(1).ok_or(Error::SequenceOverflow)?;
            let claim_key = keyspaces::claim_key(
                &claim.subject,
                &claim.predicate,
                claim.valid_from,
                claim.tx_time,
            );
            transaction.put(
                checked_key(keyspaces::CLAIMS, &claim_key)?,
                serde_json::to_vec(claim)?,
            )?;
            transaction.put(
                checked_key(
                    keyspaces::SEQUENCE_INDEX,
                    &keyspaces::sequence_key(sequence),
                )?,
                claim_key,
            )?;
        }
        put_sequence(
            &mut *transaction,
            &keyspaces::sequence_watermark_key(),
            sequence,
        )?;
        let outcome = IdempotentAppendOutcome {
            operation_sha256: operation_sha256.into(),
            append: AppendOutcome {
                first_sequence: start + 1,
                last_sequence: sequence,
                count: claims.len(),
            },
            idempotent_replay: false,
        };
        transaction.put(
            checked_key(keyspaces::META, &receipt_key)?,
            serde_json::to_vec(&outcome)?,
        )?;
        match transaction.commit(Durability::Authoritative) {
            Ok(_) => Ok(outcome),
            Err(error @ Error::TransactionConflict { .. }) => {
                let retry = self.storage.begin_transaction()?;
                let bytes = get(&*retry, keyspaces::META, &receipt_key)?.ok_or(error)?;
                let mut accepted: IdempotentAppendOutcome = serde_json::from_slice(&bytes)?;
                if accepted.operation_sha256 != operation_sha256 {
                    return Err(Error::IdempotencyConflict(idempotency_key.into()));
                }
                accepted.idempotent_replay = true;
                Ok(accepted)
            }
            Err(error) => Err(error),
        }
    }

    pub fn assert(&self, claim: &Claim) -> Result<AppendOutcome> {
        let candidates =
            self.versions_at_or_before(&claim.subject, &claim.predicate, claim.valid_from)?;
        let previous = resolve_as_of(&candidates, claim.valid_from).cloned();
        match previous {
            Some(previous) if previous.valid_from < claim.valid_from => {
                self.append_batch(&rrd_core::supersede(&previous, claim.clone())?)
            }
            _ => self.append_batch(std::slice::from_ref(claim)),
        }
    }

    pub fn sequence(&self) -> Result<u64> {
        let transaction = self.storage.begin_transaction()?;
        read_sequence(&*transaction, &keyspaces::sequence_watermark_key())
    }

    pub fn claims_in_range(&self, from: u64, to: u64) -> Result<Vec<Claim>> {
        if from >= to {
            return Ok(Vec::new());
        }
        let transaction = self.storage.begin_transaction()?;
        let head = read_sequence(&*transaction, &keyspaces::sequence_watermark_key())?;
        let last = to.min(head);
        if from >= last {
            return Ok(Vec::new());
        }
        let start = keyspaces::sequence_key(from.saturating_add(1));
        let inclusive_end = keyspaces::sequence_key(last);
        let end = prefix_end(&inclusive_end)
            .ok_or_else(|| Error::Substrate("claim sequence range has no upper bound".into()))?;
        let rows = transaction.scan(&start, &end, usize::MAX)?;
        let expected = usize::try_from(last - from)
            .map_err(|_| Error::Substrate("claim sequence range exceeds usize".into()))?;
        let mut claims = Vec::with_capacity(expected);
        let mut expected_sequence = from.saturating_add(1);
        for (sequence_key, claim_key) in rows {
            let actual_sequence = keyspaces::parse_sequence_key(KeyCodec, &sequence_key)?;
            if actual_sequence != expected_sequence {
                return Err(Error::Substrate(format!(
                    "claim sequence index expected {expected_sequence} but found {actual_sequence}"
                )));
            }
            keyspaces::validate_space(KeyCodec, keyspaces::CLAIMS, &claim_key)?;
            let encoded = transaction.get(&claim_key)?.ok_or_else(|| {
                Error::Substrate(format!(
                    "claim sequence index references an absent claim in ({from}, {last}]"
                ))
            })?;
            claims.push(serde_json::from_slice(&encoded)?);
            expected_sequence = expected_sequence
                .checked_add(1)
                .ok_or(Error::SequenceOverflow)?;
        }
        if claims.len() != expected {
            return Err(Error::Substrate(format!(
                "claim sequence index returned {} rows for expected interval ({from}, {last}]",
                claims.len()
            )));
        }
        Ok(claims)
    }

    pub fn subjects(&self) -> Result<Vec<Subject>> {
        let transaction = self.storage.begin_transaction()?;
        let mut subjects = Vec::new();
        for (stored_key, _) in scan_space(&*transaction, keyspaces::CLAIMS, &[])? {
            let (subject, _, _, _) = keyspaces::parse_claim_key(KeyCodec, &stored_key)?;
            if subjects
                .last()
                .is_none_or(|prior: &Subject| prior.as_str() != subject.as_str())
            {
                subjects.push(subject);
            }
        }
        Ok(subjects)
    }

    pub fn observe(
        &self,
        reader: &Reader,
        subject: &Subject,
        predicate: &Predicate,
        at: Millis,
    ) -> Result<()> {
        let mut transaction = self.storage.begin_transaction()?;
        let key = keyspaces::access_key(at, reader, subject, predicate);
        transaction.put(checked_key(keyspaces::ACCESS, &key)?, Vec::new())?;
        transaction.commit(Durability::Buffered)?;
        Ok(())
    }

    pub fn access_count(&self) -> Result<usize> {
        let transaction = self.storage.begin_transaction()?;
        Ok(scan_space(&*transaction, keyspaces::ACCESS, &[])?.len())
    }

    pub fn removal_report(&self, since: Millis, evaluated_at: Millis) -> Result<RemovalReport> {
        let transaction = self.storage.begin_transaction()?;
        let mut tallies = BTreeMap::<(String, String), Tally>::new();
        for (stored_key, _) in scan_space(&*transaction, keyspaces::CLAIMS, &[])? {
            let (subject, predicate, _, _) = keyspaces::parse_claim_key(KeyCodec, &stored_key)?;
            tallies
                .entry((subject.to_string(), predicate.to_string()))
                .or_default()
                .claim_count += 1;
        }
        for (stored_key, _) in scan_space_from(
            &*transaction,
            keyspaces::ACCESS,
            &keyspaces::access_bound(since),
        )? {
            let (at, reader, subject, predicate) =
                keyspaces::parse_access_key(KeyCodec, &stored_key)?;
            if at > evaluated_at {
                break;
            }
            let tally = tallies
                .entry((subject.to_string(), predicate.to_string()))
                .or_default();
            tally.access_count += 1;
            if tally.last_access.is_none_or(|previous| at >= previous) {
                tally.last_access = Some(at);
                tally.last_reader = Some(reader);
            }
        }
        build_report(tallies, since, evaluated_at).map_err(Error::from)
    }

    fn scan_claims(&self, prefix: Vec<u8>, from: Vec<u8>) -> Result<Vec<Claim>> {
        let transaction = self.storage.begin_transaction()?;
        let start = checked_key(keyspaces::CLAIMS, &from)?;
        let full_prefix = checked_key(keyspaces::CLAIMS, &prefix)?;
        let end = prefix_end(&full_prefix)
            .ok_or_else(|| Error::Substrate("claim prefix has no upper bound".into()))?;
        transaction
            .scan(&start, &end, usize::MAX)?
            .into_iter()
            .map(|(_, value)| serde_json::from_slice(&value).map_err(Error::from))
            .collect()
    }
}

impl ClaimSource for ClaimRepository<'_> {
    type Error = Error;

    fn versions_at_or_before(
        &self,
        subject: &Subject,
        predicate: &Predicate,
        as_of: Millis,
    ) -> Result<Vec<Claim>> {
        self.scan_claims(
            keyspaces::claim_version_prefix(subject, predicate),
            keyspaces::claim_seek_key(subject, predicate, as_of),
        )
    }

    fn all_versions(&self, subject: &Subject, predicate: &Predicate) -> Result<Vec<Claim>> {
        let prefix = keyspaces::claim_version_prefix(subject, predicate);
        self.scan_claims(prefix.clone(), prefix)
    }

    fn subject_versions(&self, subject: &Subject) -> Result<Vec<Claim>> {
        let prefix = keyspaces::claim_subject_prefix(subject);
        self.scan_claims(prefix.clone(), prefix)
    }

    fn subject_versions_batch(&self, subjects: &[Subject]) -> Result<Vec<Vec<Claim>>> {
        if subjects.is_empty() {
            return Ok(Vec::new());
        }
        let transaction = self.storage.begin_transaction()?;
        let mut grouped = BTreeMap::<Subject, Vec<Claim>>::new();
        for subject in subjects {
            let prefix = keyspaces::claim_subject_prefix(subject);
            let start = checked_key(keyspaces::CLAIMS, &prefix)?;
            let end = prefix_end(&start)
                .ok_or_else(|| Error::Substrate("claim prefix has no upper bound".into()))?;
            for (_, value) in transaction.scan(&start, &end, usize::MAX)? {
                let claim: Claim = serde_json::from_slice(&value)?;
                grouped
                    .entry(claim.subject.clone())
                    .or_default()
                    .push(claim);
            }
        }
        Ok(subjects
            .iter()
            .map(|subject| grouped.get(subject).cloned().unwrap_or_default())
            .collect())
    }
}
