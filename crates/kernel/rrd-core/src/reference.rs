//! In-memory reference implementation of [`ClaimSource`].
//!
//! This implementation serves two functions:
//!
//! 1. It allows `rrd-core` to verify semantic resolution independently of any
//!    physical key codec or substrate, satisfying the modularity criterion in
//!    `SPEC.md` §5.
//! 2. It is the grounding reference of `SPEC.md` §8.3. A substrate adapter is
//!    correct if and only if it returns what this implementation returns for the
//!    same claims. Divergence must halt rather than be repaired.
//!
use crate::claim::{Claim, Millis};
use crate::error::Result;
use crate::ident::{Predicate, Subject};
use crate::temporal::ClaimSource;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::convert::Infallible;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ClaimOrder {
    subject: Subject,
    predicate: Predicate,
    valid_from: Reverse<Millis>,
    tx_time: Reverse<Millis>,
}

impl From<&Claim> for ClaimOrder {
    fn from(claim: &Claim) -> Self {
        Self {
            subject: claim.subject.clone(),
            predicate: claim.predicate.clone(),
            valid_from: Reverse(claim.valid_from),
            tx_time: Reverse(claim.tx_time),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MemoryClaims {
    rows: BTreeMap<ClaimOrder, Claim>,
}

impl MemoryClaims {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, claim: Claim) -> Result<()> {
        claim.validate()?;
        self.rows.insert(ClaimOrder::from(&claim), claim);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Every claim in key order. Used by grounding to recompute from scratch.
    pub fn iter(&self) -> impl Iterator<Item = &Claim> {
        self.rows.values()
    }

    fn versions(
        &self,
        subject: &Subject,
        predicate: &Predicate,
        as_of: Option<Millis>,
    ) -> Vec<Claim> {
        self.rows
            .values()
            .filter(|claim| {
                &claim.subject == subject
                    && &claim.predicate == predicate
                    && as_of.is_none_or(|at| claim.valid_from <= at)
            })
            .cloned()
            .collect()
    }
}

impl ClaimSource for MemoryClaims {
    type Error = Infallible;

    fn versions_at_or_before(
        &self,
        subject: &Subject,
        predicate: &Predicate,
        as_of: Millis,
    ) -> std::result::Result<Vec<Claim>, Self::Error> {
        Ok(self.versions(subject, predicate, Some(as_of)))
    }

    fn all_versions(
        &self,
        subject: &Subject,
        predicate: &Predicate,
    ) -> std::result::Result<Vec<Claim>, Self::Error> {
        Ok(self.versions(subject, predicate, None))
    }

    fn subject_versions(&self, subject: &Subject) -> std::result::Result<Vec<Claim>, Self::Error> {
        Ok(self
            .rows
            .values()
            .filter(|claim| &claim.subject == subject)
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::Producer;
    use crate::temporal::ClaimReader;

    fn store() -> MemoryClaims {
        let mut s = MemoryClaims::new();
        let p = Producer {
            actor: "test".into(),
            on_behalf_of: None,
            session: None,
        };
        let subj = Subject::new("wp3").unwrap();
        let pred = Predicate::new("status").unwrap();
        for (obj, vf, vt) in [
            ("v1", 100u64, Some(200u64)),
            ("v2", 200, Some(300)),
            ("v3", 300, None),
        ] {
            let mut c = Claim::new(subj.clone(), pred.clone(), obj, vf, vf, p.clone());
            c.valid_to = vt;
            s.insert(c).unwrap();
        }
        // adversarial neighbours that must never appear in wp3/status results
        for (s_, p_) in [("wp3x", "status"), ("wp3", "statusx"), ("wp", "status")] {
            let c = Claim::new(
                Subject::new(s_).unwrap(),
                Predicate::new(p_).unwrap(),
                "WRONG",
                250,
                250,
                p.clone(),
            );
            s.insert(c).unwrap();
        }
        s
    }

    #[test]
    fn as_of_matches_the_canonical_boundary_expectations() {
        // Seven cases cover the canonical point-in-time boundary behavior.
        let s = store();
        let subj = Subject::new("wp3").unwrap();
        let pred = Predicate::new("status").unwrap();
        let at = |t| s.as_of(&subj, &pred, t).unwrap().map(|c| c.object);
        assert_eq!(at(99), None);
        assert_eq!(at(100), Some("v1".into()));
        assert_eq!(at(150), Some("v1".into()));
        assert_eq!(at(200), Some("v2".into()));
        assert_eq!(at(250), Some("v2".into()));
        assert_eq!(at(300), Some("v3".into()));
        assert_eq!(at(9999), Some("v3".into()));
    }

    #[test]
    fn history_is_newest_first_and_isolated() {
        let s = store();
        let subj = Subject::new("wp3").unwrap();
        let pred = Predicate::new("status").unwrap();
        let objs: Vec<_> = s
            .history(&subj, &pred)
            .unwrap()
            .into_iter()
            .map(|c| c.object)
            .collect();
        assert_eq!(objs, vec!["v3", "v2", "v1"]);
        assert!(
            !objs.iter().any(|o| o == "WRONG"),
            "neighbour leaked into results"
        );
    }
}
