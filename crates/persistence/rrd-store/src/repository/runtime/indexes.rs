//! Commit-time index reads: BM25 search and scalar lookup.

use super::*;

impl<'a> RuntimeRepository<'a> {
    /// Exact BM25 top-k over the current postings of a commit-bound index at
    /// the head read stamp. A stamp behind the head, a valid time before the
    /// index's watermark, or an index without a BM25 binding is reported
    /// as not applicable so the caller can read the versioned history instead.
    /// Reaching `max_keys` refuses the search rather than rank partially.
    pub fn lexical_search(
        &self,
        read: &ReadStamp,
        request: crate::LexicalQuery<'_>,
    ) -> Result<crate::LexicalSearch> {
        let index = request.index;
        let transaction = self.storage.begin_transaction()?;
        read.validate()?;
        let head = read_stamp_with(&*transaction, &read.scope)?;
        if head.commit_cursor != read.commit_cursor || head.manifest_id != read.manifest_id {
            return Ok(crate::LexicalSearch::NotApplicable(format!(
                "read stamp at cursor {} is not the head at cursor {}",
                read.commit_cursor, head.commit_cursor
            )));
        }
        let Some((config, _, _)) =
            crate::access::lexical_binding(&*transaction, &read.scope, index)?
        else {
            return Ok(crate::LexicalSearch::NotApplicable(format!(
                "index {index} has no BM25 commit binding"
            )));
        };
        crate::access::lexical_index::search(&*transaction, &read.scope, &config, &request)
    }

    /// Current records of a scalar commit-bound index whose leading indexed
    /// values equal `values`, visible at `valid_at`, at the head read stamp.
    /// Like [`Self::lexical_search`], a stamp behind the head or a valid time
    /// before the index's watermark is not applicable. The caller
    /// reads the returned identities at its stamp and evaluates every
    /// predicate, so the index narrows a read and never answers alone.
    pub fn index_lookup(
        &self,
        read: &ReadStamp,
        index: &rrd_core::ProjectionId,
        values: &[rrd_core::RuntimeValue],
        valid_at: u64,
        max_entries: usize,
    ) -> Result<crate::IndexLookup> {
        let transaction = self.storage.begin_transaction()?;
        read.validate()?;
        let head = read_stamp_with(&*transaction, &read.scope)?;
        if head.commit_cursor != read.commit_cursor || head.manifest_id != read.manifest_id {
            return Ok(crate::IndexLookup::NotApplicable(format!(
                "read stamp at cursor {} is not the head at cursor {}",
                read.commit_cursor, head.commit_cursor
            )));
        }
        let Some(definition) = crate::access::commit_binding(&*transaction, &read.scope, index)?
        else {
            return Ok(crate::IndexLookup::NotApplicable(format!(
                "index {index} has no commit binding"
            )));
        };
        let watermark = crate::access::lexical_index::watermark(&*transaction, &read.scope, index)?;
        if valid_at < watermark {
            return Ok(crate::IndexLookup::NotApplicable(format!(
                "valid time {valid_at} precedes the watermark {watermark} of index {index}"
            )));
        }
        let (references, keys_read) = crate::access::scalar_lookup(
            &*transaction,
            &read.scope,
            &definition,
            values,
            valid_at,
            max_entries,
        )?;
        Ok(crate::IndexLookup::Records {
            references,
            keys_read,
        })
    }
}
