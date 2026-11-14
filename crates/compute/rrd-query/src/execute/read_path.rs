//! Loading each read path's rows within the budget.

use super::*;

impl ReadPath {
    pub(super) fn from_plan(plan: &PhysicalPlan, shape: &PlanShape) -> Result<Self> {
        let contract = &plan.explanation.contract;
        if contract.read_manifest != plan.logical.read.manifest_id
            || contract.scope != plan.logical.read.scope.as_str()
            || contract.valid_at != shape.valid_at
            || contract.known_at_cursor != shape.known_at_cursor
            || contract.source_cursor != plan.logical.source_cursor
            || contract.schema_revision != plan.logical.schema_revision
            || contract.authorization_boundary != format!("scope:{}", plan.logical.read.scope)
            || contract.read_authentication != "rfc9162_authenticated_semantic_versions"
            || contract.read_budget_unit != "physical_keys_examined"
        {
            return Err(Error::Integrity(
                "logical plan, read stamp, and execution contract disagree".into(),
            ));
        }
        let selected = plan
            .explanation
            .candidates
            .iter()
            .filter(|candidate| candidate.selected)
            .collect::<Vec<_>>();
        if selected.len() != 1 || !selected[0].exact {
            return Err(Error::Integrity(
                "physical plan must report exactly one exact selected path".into(),
            ));
        }
        let [access, PhysicalOperator::DataFusionEvaluate] = plan.operators.as_slice() else {
            return Err(Error::Integrity(
                "physical plan must contain one authoritative access path followed by DataFusion evaluation"
                    .into(),
            ));
        };
        let traversals = std::iter::once(&shape.source)
            .chain(shape.join.as_ref().map(|join| &join.source))
            .filter_map(TraversalPlan::from_source)
            .collect::<Vec<_>>();
        match access {
            PhysicalOperator::VersionedRead {
                sources,
                exact,
                stable_order,
            } if *exact
                && traversals.is_empty()
                && stable_order == "global_cursor"
                && sources == &plan.logical.version_sources
                && selected[0].name
                    == if matches!(
                        sources.as_slice(),
                        [RuntimeVersionedSource::Identity { .. }]
                    ) {
                        "versioned_identity_read"
                    } else if matches!(
                        sources.as_slice(),
                        [RuntimeVersionedSource::IdentityRange { .. }]
                    ) {
                        "versioned_identity_range_read"
                    } else if matches!(
                        sources.as_slice(),
                        [RuntimeVersionedSource::Adjacency { .. }]
                    ) {
                        "versioned_adjacency_read"
                    } else {
                        "versioned_source_read"
                    } =>
            {
                Ok(Self::Versioned {
                    sources: sources.clone(),
                })
            }
            PhysicalOperator::AdjacencyTraversal {
                sources,
                exact,
                stable_order,
            } if *exact
                && !traversals.is_empty()
                && stable_order == "global_cursor"
                && sources == &plan.logical.version_sources
                && selected[0].name == "adjacency_traversal"
                && traversals.iter().all(|traversal| {
                    sources.iter().any(|source| match source {
                        RuntimeVersionedSource::Identity { model, reference } => {
                            model.is_record_like() && reference == &traversal.start
                        }
                        RuntimeVersionedSource::Records { kind } => kind
                            .as_ref()
                            .is_none_or(|kind| kind == &traversal.start.kind),
                        _ => false,
                    })
                }) =>
            {
                Ok(Self::Adjacency {
                    sources: sources.clone(),
                    traversals,
                })
            }
            PhysicalOperator::CommitIndexRead {
                id,
                probe,
                model,
                fallback,
                exact,
                stable_order,
            } if *exact
                && traversals.is_empty()
                && shape.join.is_none()
                && fallback == &plan.logical.version_sources
                && stable_order
                    == if matches!(probe, crate::plan::CommitIndexProbe::Bm25 { .. }) {
                        "bm25_score_desc_identity_ascending"
                    } else {
                        "global_cursor"
                    }
                && selected[0].name == format!("commit_index:{id}") =>
            {
                Ok(Self::CommitIndex {
                    id: id.clone(),
                    probe: probe.clone(),
                    model: *model,
                    fallback: fallback.clone(),
                })
            }
            PhysicalOperator::MaterializedIndex {
                id,
                kind,
                generation,
                source_cursor,
                schema_revision,
                valid_at,
                config_digest,
                artifact_digest,
                artifact_rows,
                exact,
                stable_order,
            } if *source_cursor == plan.logical.source_cursor
                && *schema_revision == plan.logical.schema_revision
                && *valid_at == shape.valid_at
                && *exact
                && stable_order
                    == if matches!(kind, IndexKind::Bm25 { .. }) {
                        "bm25_score_desc_identity_ascending"
                    } else {
                        "identity_ascending"
                    }
                && selected[0].name == format!("index:{id}") =>
            {
                Ok(Self::Index {
                    id: id.clone(),
                    kind: kind.clone(),
                    generation: *generation,
                    source_cursor: *source_cursor,
                    schema_revision: *schema_revision,
                    valid_at: *valid_at,
                    config_digest: config_digest.clone(),
                    artifact_digest: artifact_digest.clone(),
                    artifact_rows: *artifact_rows,
                })
            }
            _ => Err(Error::Integrity(
                "physical access path does not match the stamped logical query".into(),
            )),
        }
    }

    pub(super) fn load<E: StorageEngine>(
        &self,
        engine: &E,
        stamp: &rrd_core::ReadStamp,
        shape: &PlanShape,
        budget: RuntimeReadBudget,
    ) -> Result<LoadedAccess> {
        if let Self::CommitIndex {
            id,
            probe,
            model,
            fallback,
        } = self
        {
            let max_keys = budget.max_keys;
            let (selected, ranked, index_keys) = match probe {
                crate::plan::CommitIndexProbe::Scalar { values } => {
                    match engine.runtime().index_lookup(
                        stamp,
                        id,
                        values,
                        shape.valid_at,
                        usize::try_from(max_keys).unwrap_or(usize::MAX),
                    )? {
                        rrd_store::IndexLookup::Records {
                            references,
                            keys_read,
                        } => (Some(references), None, keys_read),
                        rrd_store::IndexLookup::NotApplicable(_) => (None, None, 0),
                    }
                }
                crate::plan::CommitIndexProbe::Bm25 {
                    query,
                    config,
                    top_k,
                } => match engine.runtime().lexical_search(
                    stamp,
                    rrd_store::LexicalQuery {
                        index: id,
                        text: query,
                        valid_at: shape.valid_at,
                        top_k: *top_k,
                        max_keys,
                    },
                )? {
                    rrd_store::LexicalSearch::Hits {
                        hits, keys_read, ..
                    } => {
                        let references = hits.iter().map(|hit| hit.record.clone()).collect();
                        let ranked = Ranked {
                            config: config.clone(),
                            hits: hits
                                .into_iter()
                                .map(|hit| {
                                    (
                                        result_identity::typed(
                                            ResultIdentityFamily::Record,
                                            &hit.record,
                                        ),
                                        (hit.score, hit.matched_terms),
                                    )
                                })
                                .collect(),
                        };
                        (Some(references), Some(ranked), keys_read)
                    }
                    rrd_store::LexicalSearch::NotApplicable(_) => (None, None, 0),
                },
            };
            // The index's keys count against the same read budget.
            let budget = RuntimeReadBudget::new(
                max_keys
                    .checked_sub(index_keys)
                    .filter(|keys| *keys > 0)
                    .ok_or_else(|| {
                        Error::Budget(format!(
                            "index {id} read {index_keys} keys of a {max_keys}-key budget"
                        ))
                    })?,
            )?;
            let Some(references) = selected else {
                return Self::Versioned {
                    sources: fallback.clone(),
                }
                .load(engine, stamp, shape, budget);
            };
            if references.is_empty() {
                let validation = engine.runtime().read_versioned(
                    stamp,
                    &[RuntimeVersionedSource::Schema],
                    budget,
                )?;
                return Ok(LoadedAccess {
                    rows: Vec::new(),
                    right_rows: None,
                    bm25: None,
                    ranked: ranked.map(|ranked| Ranked {
                        hits: BTreeMap::new(),
                        ..ranked
                    }),
                    selected_versions: 0,
                    read_evidence: validation.evidence,
                });
            }
            let sources = references
                .into_iter()
                .map(|reference| RuntimeVersionedSource::Identity {
                    model: *model,
                    reference,
                })
                .collect::<Vec<_>>();
            let mut loaded = Self::Versioned { sources }.load(engine, stamp, shape, budget)?;
            loaded.ranked = ranked;
            return Ok(loaded);
        }
        if let Self::Index {
            id,
            kind,
            generation,
            source_cursor,
            schema_revision,
            valid_at,
            config_digest,
            artifact_digest,
            artifact_rows,
        } = self
        {
            let validation = engine.runtime().read_versioned(
                stamp,
                &[RuntimeVersionedSource::Schema],
                budget,
            )?;
            let name = index_artifact_name(&stamp.scope, id, *generation, artifact_digest);
            let bytes = engine
                .projections()
                .get(&name)?
                .ok_or_else(|| Error::Integrity(format!("index artifact {id} is missing")))?;
            if rrd_core::digest::sha256_hex(&bytes) != *artifact_digest {
                return Err(Error::Integrity(format!(
                    "index artifact {id} digest does not verify"
                )));
            }
            let row_limit = if shape.filters.is_empty()
                && !matches!(kind, IndexKind::Bm25 { .. })
                && shape.join.is_none()
            {
                shape.limit
            } else {
                None
            };
            let artifact = decode_index_artifact_for_read(&bytes, row_limit)?;
            if artifact.scope != stamp.scope
                || artifact.definition.id != *id
                || artifact.definition.kind != *kind
                || artifact.definition.source != shape.source
                || artifact.definition.config_digest()? != *config_digest
                || artifact.generation != *generation
                || artifact.source_cursor != *source_cursor
                || artifact.read_cursor > shape.known_at_cursor
                || artifact.schema_revision != *schema_revision
                || artifact.valid_at != *valid_at
                || artifact.total_rows
                    != usize::try_from(*artifact_rows)
                        .map_err(|_| Error::Budget("index artifact rows exceed usize".into()))?
            {
                return Err(Error::Integrity(format!(
                    "index artifact {id} does not satisfy the physical plan"
                )));
            }
            return Ok(LoadedAccess {
                rows: artifact.rows,
                right_rows: None,
                bm25: artifact.bm25,
                ranked: None,
                selected_versions: 0,
                read_evidence: validation.evidence,
            });
        }

        struct LoadedIndexArtifact {
            scope: rrd_core::ScopeId,
            definition: IndexDefinition,
            generation: u64,
            source_cursor: u64,
            read_cursor: u64,
            schema_revision: u64,
            valid_at: u64,
            rows: Vec<QueryRow>,
            total_rows: usize,
            bm25: Option<Bm25Artifact>,
        }

        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct IndexArtifactEnvelope<'a> {
            contract_version: u16,
            result_identity_version: u16,
            scope: rrd_core::ScopeId,
            definition: IndexDefinition,
            generation: u64,
            source_cursor: u64,
            read_cursor: u64,
            schema_revision: u64,
            valid_at: u64,
            #[serde(borrow)]
            rows: &'a RawValue,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            bm25: Option<Bm25Artifact>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            analytics: Option<crate::IndexAnalyticsArtifact>,
            maintenance: IndexMaintenanceEvidence,
        }

        fn decode_index_artifact_for_read(
            bytes: &[u8],
            row_limit: Option<usize>,
        ) -> Result<LoadedIndexArtifact> {
            let envelope: IndexArtifactEnvelope<'_> = serde_json::from_slice(bytes)?;
            if envelope.contract_version != INDEX_ARTIFACT_CONTRACT_VERSION {
                return Err(Error::Integrity(format!(
                    "unsupported index artifact contract version {}",
                    envelope.contract_version
                )));
            }
            if envelope.result_identity_version != RESULT_IDENTITY_VERSION {
                return Err(Error::Integrity(format!(
                    "unsupported index result identity version {}",
                    envelope.result_identity_version
                )));
            }
            envelope.definition.validate()?;
            if envelope.generation == 0
                || envelope.source_cursor == 0
                || envelope.read_cursor < envelope.source_cursor
            {
                return Err(Error::Integrity(
                    "index artifact generation/source cursor/read cursor are invalid".into(),
                ));
            }
            if envelope.maintenance.source_cursor != envelope.source_cursor
                || (envelope.maintenance.mode != "full_build"
                    && envelope.maintenance.mode != "incremental_reconciliation")
            {
                return Err(Error::Integrity(
                    "index maintenance evidence does not cover this artifact".into(),
                ));
            }
            match (
                &envelope.definition.kind,
                &envelope.bm25,
                &envelope.analytics,
            ) {
                (IndexKind::Scalar | IndexKind::Geo | IndexKind::MaterializedView, None, None) => {}
                (IndexKind::Count | IndexKind::AggregateCount, None, Some(analytics)) => {
                    validate_count_analytics(&envelope.definition, analytics)?;
                }
                (IndexKind::Bm25 { config }, Some(artifact), None) => {
                    artifact.validate()?;
                    if &artifact.config != config
                        || artifact.source_cursor != envelope.source_cursor
                        || artifact.schema_revision != envelope.schema_revision
                        || artifact.valid_at != envelope.valid_at
                    {
                        return Err(Error::Integrity(
                            "BM25 artifact coordinates differ from the containing query index"
                                .into(),
                        ));
                    }
                }
                _ => {
                    return Err(Error::Integrity(
                        "query index kind and specialized artifact disagree".into(),
                    ))
                }
            }
            let (rows, total_rows) = decode_index_rows(envelope.rows.get(), row_limit)?;
            if total_rows > 1_000_000 {
                return Err(Error::Integrity("index artifact row limit exceeded".into()));
            }
            Ok(LoadedIndexArtifact {
                scope: envelope.scope,
                definition: envelope.definition,
                generation: envelope.generation,
                source_cursor: envelope.source_cursor,
                read_cursor: envelope.read_cursor,
                schema_revision: envelope.schema_revision,
                valid_at: envelope.valid_at,
                rows,
                total_rows,
                bm25: envelope.bm25,
            })
        }

        fn decode_index_rows(
            rows_json: &str,
            row_limit: Option<usize>,
        ) -> Result<(Vec<QueryRow>, usize)> {
            let mut deserializer = serde_json::Deserializer::from_str(rows_json);
            let seed = RowCollector { row_limit };
            let (rows, total) = seed.deserialize(&mut deserializer)?;
            deserializer.end()?;
            Ok((rows, total))
        }

        struct RowCollector {
            row_limit: Option<usize>,
        }

        impl<'de> DeserializeSeed<'de> for RowCollector {
            type Value = (Vec<QueryRow>, usize);

            fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_seq(RowCollectorVisitor {
                    row_limit: self.row_limit,
                })
            }
        }

        struct RowCollectorVisitor {
            row_limit: Option<usize>,
        }

        impl<'de> Visitor<'de> for RowCollectorVisitor {
            type Value = (Vec<QueryRow>, usize);

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an array of query rows")
            }

            fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut rows = Vec::new();
                let mut total = 0usize;
                let mut previous_identity: Option<String> = None;
                while self.row_limit.is_none_or(|limit| rows.len() < limit) {
                    let Some(row) = seq.next_element::<QueryRow>()? else {
                        return Ok((rows, total));
                    };
                    total = total.saturating_add(1);
                    if previous_identity
                        .as_ref()
                        .is_some_and(|previous| previous >= &row.identity)
                    {
                        return Err(serde::de::Error::custom(
                            "index artifact row identities must be unique and sorted",
                        ));
                    }
                    previous_identity = Some(row.identity.clone());
                    rows.push(row);
                }
                while seq.next_element::<IgnoredAny>()?.is_some() {
                    total = total.saturating_add(1);
                }
                Ok((rows, total))
            }
        }

        fn validate_count_analytics(
            definition: &IndexDefinition,
            analytics: &crate::IndexAnalyticsArtifact,
        ) -> Result<()> {
            if matches!(definition.kind, IndexKind::Count) && !analytics.groups.is_empty() {
                return Err(Error::Integrity(
                    "a count index cannot contain groups".into(),
                ));
            }
            if matches!(definition.kind, IndexKind::AggregateCount) {
                let grouped_total = analytics.groups.iter().try_fold(0_u64, |sum, group| {
                    if group.values.len() != definition.fields.len() || group.count == 0 {
                        return Err(Error::Integrity("grouped-count entry is invalid".into()));
                    }
                    sum.checked_add(group.count)
                        .ok_or_else(|| Error::Integrity("grouped-count total overflow".into()))
                })?;
                if grouped_total != analytics.total_count {
                    return Err(Error::Integrity(
                        "grouped-count total does not match its groups".into(),
                    ));
                }
            }
            Ok(())
        }
        let selected = match self {
            Self::Versioned { sources } => {
                if matches!(
                    sources.as_slice(),
                    [RuntimeVersionedSource::Adjacency { .. }]
                ) {
                    let mut expanded = false;
                    engine.runtime().read_versioned_expanding(
                        stamp,
                        sources,
                        budget,
                        &mut |changes| {
                            if std::mem::replace(&mut expanded, true) {
                                return Vec::new();
                            }
                            // An adjacency entry names an identity, not its final
                            // temporal state. Later endpoint moves and retirements
                            // outside this adjacency must still govern visibility.
                            changes
                                .iter()
                                .filter_map(|change| match &change.mutation {
                                    RuntimeMutation::Relation { relation } => {
                                        Some(relation.reference.clone())
                                    }
                                    RuntimeMutation::Retire { retirement }
                                        if retirement.model
                                            == rrd_core::RuntimeLogicalModel::GraphRelation =>
                                    {
                                        Some(retirement.reference.clone())
                                    }
                                    _ => None,
                                })
                                .collect::<std::collections::BTreeSet<_>>()
                                .into_iter()
                                .map(|reference| RuntimeVersionedSource::Identity {
                                    model: rrd_core::RuntimeLogicalModel::GraphRelation,
                                    reference,
                                })
                                .collect()
                        },
                    )?
                } else {
                    engine.runtime().read_versioned(stamp, sources, budget)?
                }
            }
            Self::Adjacency {
                sources,
                traversals,
            } => {
                let mut expansion = AdjacencyExpansion::new(
                    &stamp.scope,
                    shape.valid_at,
                    shape.known_at_cursor,
                    sources,
                    traversals,
                );
                engine.runtime().read_versioned_expanding(
                    stamp,
                    sources,
                    budget,
                    &mut |changes| expansion.next(changes),
                )?
            }
            Self::Index { .. } | Self::CommitIndex { .. } => {
                unreachable!("indexes return above")
            }
        };
        let selected_versions = selected.changes.len();
        let rows = rows_for_source(
            &selected.changes,
            &stamp.scope,
            &shape.source,
            shape.valid_at,
            shape.known_at_cursor,
        );
        let right_rows = shape.join.as_ref().map(|join| {
            rows_for_source(
                &selected.changes,
                &stamp.scope,
                &join.source,
                shape.valid_at,
                shape.known_at_cursor,
            )
        });
        Ok(LoadedAccess {
            rows,
            right_rows,
            bm25: None,
            ranked: None,
            selected_versions,
            read_evidence: selected.evidence,
        })
    }
}
