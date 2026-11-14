//! Assembling context at a read stamp: seed, lexical, vector and graph stages, then fusion.

use super::*;

impl RrdEngine {
    /// Resolves a bounded context packet from authenticated semantic versions.
    ///
    /// The caller supplies intent and optional record anchors. The engine
    /// discovers textual properties and graph topology from one temporal
    /// snapshot, applies deterministic rank fusion, and returns evidence tied
    /// to that snapshot's authenticated read stamp.
    #[allow(clippy::too_many_arguments)]
    pub fn assemble_context(
        &self,
        session_id: &CorrelationId,
        token: &CorrelationId,
        request: &AssembleContext,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<ContextPacket> {
        request
            .validate()
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        let (_, _, authorization) = self.authorize_resource_without_data_policy(
            session_id,
            token,
            SecurityAction::MemoryContextRead,
            &self.instance_resource(),
            now,
            request_id,
            operation_id,
        )?;
        self.assemble_context_at(request, authorization.as_ref())
    }

    pub(in crate::engine) fn assemble_context_at(
        &self,
        request: &AssembleContext,
        authorization: Option<&rrd_security::Authorization>,
    ) -> Result<ContextPacket> {
        request
            .validate()
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        self.enforce_recall_configuration("memory.context.assemble", request)?;
        let (security_policy_revision, authorization_sha256) = authorization.map_or_else(
            || {
                Ok((
                    0,
                    digest::sha256_hex(b"rrd-security-disabled-loopback-development"),
                ))
            },
            |authorization| {
                if authorization.data_policy.is_some() {
                    return Err(ServiceError::PermissionDenied);
                }
                Ok((
                    authorization.policy_revision,
                    authorization.authorization_sha256.clone(),
                ))
            },
        )?;
        let scope = self.query_scope(&request.scope)?;
        let key_budget = usize::try_from(request.max_storage_keys)
            .map_err(|_| ServiceError::Query("context key budget exceeds usize".into()))?;
        let stamp = self.storage.runtime().read_stamp(&scope)?;
        let lexical = self.commit_lexical(request, &stamp, key_budget)?;
        let RuntimeDataSnapshotRead {
            read,
            selected_versions,
            read_evidence,
            snapshot,
        } = self.storage.runtime().data_snapshot_at(
            &stamp,
            request.valid_at,
            key_budget.saturating_sub(lexical.keys_read),
            &lexical.excluded,
            &lexical.identities,
        )?;
        if snapshot.known_at_cursor != read.commit_cursor
            || snapshot.schema_revision != read.schema_revision.unwrap_or(0)
        {
            return Err(ServiceError::StorageConflict(
                "context snapshot differs from its captured read stamp".into(),
            ));
        }

        let records = snapshot
            .records
            .iter()
            .map(|entry| (entry.value.reference.clone(), entry.value.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut candidates = BTreeMap::<String, ContextCandidate>::new();
        let mut stages = Vec::with_capacity(5);
        let mut retrieval_truncated = false;
        let seed_plan = context_digest(&(
            "rrd-context-seed-v1",
            read.manifest_id.as_str(),
            request.valid_at,
        ))?;
        let mut visible_seed_count = 0_usize;
        let mut graph_roots = Vec::<RuntimeRef>::new();
        for (ordinal, seed) in request.seeds.iter().enumerate() {
            let reference = runtime_ref(seed)?;
            let Some(record) = records.get(&reference) else {
                continue;
            };
            visible_seed_count += 1;
            graph_roots.push(reference.clone());
            let rank = u64::try_from(ordinal + 1)
                .map_err(|_| ServiceError::Contract("context seed rank exceeds u64".into()))?;
            let evidence = ContextEvidence {
                kind: ContextEvidenceKind::Seed,
                source: "request.seed".into(),
                source_rank: rank,
                source_score: 1.0,
                contribution: reciprocal_rank(SEED_WEIGHT, rank),
                plan_sha256: seed_plan.clone(),
                evidence_sha256: context_digest(&(
                    "rrd-context-seed-evidence-v1",
                    read.manifest_id.as_str(),
                    &reference,
                    rank,
                ))?,
            };
            add_record_candidate(&mut candidates, &reference, record, evidence)?;
        }
        stages.push(context_plan_stage(
            ContextPlanStageKind::Seed,
            if request.seeds.is_empty() {
                ContextPlanStageStatus::Skipped
            } else {
                ContextPlanStageStatus::Selected
            },
            (!request.seeds.is_empty()).then_some(ContextAccessPath::RequestSeed),
            !request.seeds.is_empty(),
            if request.seeds.is_empty() {
                "request supplied no record anchors".into()
            } else {
                format!(
                    "resolved caller-supplied anchors against the captured temporal snapshot (requested={}, visible={})",
                    request.seeds.len(),
                    visible_seed_count
                )
            },
        )?);

        let mut text_roots = Vec::<(RuntimeRef, u64)>::new();
        let mut text_source_count = 0_usize;
        if !request.query.trim().is_empty() && (!records.is_empty() || !snapshot.claims.is_empty())
        {
            let mut sources = BTreeMap::<String, TextSource>::new();
            for (reference, record) in &records {
                if lexical.excluded.contains(&reference.kind) {
                    continue;
                }
                let text = record_text(&record.properties);
                if !text.is_empty() {
                    sources.insert(
                        record_identity(reference),
                        TextSource {
                            graph_root: Some(reference.clone()),
                            evidence_source: format!("record:{}:dynamic_text", reference.kind),
                            text,
                            values: public_properties(&record.properties)?,
                        },
                    );
                }
            }
            for claim in &snapshot.claims {
                let text = claim_text(claim);
                if !text.is_empty() {
                    sources.insert(
                        claim_identity(claim),
                        TextSource {
                            graph_root: None,
                            evidence_source: "claim:dynamic_text".into(),
                            text,
                            values: claim_values(claim),
                        },
                    );
                }
            }
            let documents = sources
                .iter()
                .map(|(identity, source)| (identity.clone(), source.text.clone()))
                .collect::<Vec<_>>();
            if !documents.is_empty() {
                text_source_count = documents.len();
                let artifact = rrd_query::Bm25Artifact::build(
                    rrd_query::Bm25Config::default(),
                    read.commit_cursor,
                    snapshot.schema_revision,
                    request.valid_at,
                    documents,
                )
                .map_err(|error| ServiceError::Query(error.to_string()))?;
                let plan_sha256 = artifact
                    .digest()
                    .map_err(|error| ServiceError::Query(error.to_string()))?;
                let top_k = usize::try_from(request.max_items)
                    .map_err(|_| ServiceError::Query("context item bound exceeds usize".into()))?;
                let mut hits = artifact
                    .search(&request.query, top_k.saturating_add(1))
                    .map_err(|error| ServiceError::Query(error.to_string()))?;
                if hits.len() > top_k {
                    retrieval_truncated = true;
                    hits.truncate(top_k);
                }
                for (ordinal, hit) in hits.into_iter().enumerate() {
                    let Some(source) = sources.get(&hit.identity) else {
                        return Err(ServiceError::Storage(
                            "context text projection returned an unknown source".into(),
                        ));
                    };
                    let rank = u64::try_from(ordinal + 1).map_err(|_| {
                        ServiceError::Contract("context text rank exceeds u64".into())
                    })?;
                    let evidence = ContextEvidence {
                        kind: ContextEvidenceKind::Text,
                        source: source.evidence_source.clone(),
                        source_rank: rank,
                        source_score: hit.score,
                        contribution: reciprocal_rank(TEXT_WEIGHT, rank),
                        plan_sha256: plan_sha256.clone(),
                        evidence_sha256: context_digest(&(
                            "rrd-context-text-evidence-v1",
                            read.manifest_id.as_str(),
                            &hit.identity,
                            &hit.matched_terms,
                            rank,
                            hit.score.to_bits(),
                        ))?,
                    };
                    add_candidate(
                        &mut candidates,
                        hit.identity,
                        source.values.clone(),
                        evidence,
                    );
                    if let Some(reference) = &source.graph_root {
                        text_roots.push((reference.clone(), rank));
                    }
                }
            }
        }
        let mut commit_hits = 0_usize;
        for (index, hits) in &lexical.hits {
            let top_k = usize::try_from(request.max_items)
                .map_err(|_| ServiceError::Query("context item bound exceeds usize".into()))?;
            if hits.len() > top_k {
                retrieval_truncated = true;
            }
            for (ordinal, hit) in hits.iter().take(top_k).enumerate() {
                let Some(record) = records.get(&hit.record) else {
                    // The posting named a version the stamp's snapshot does
                    // not show at this valid time; it contributes nothing.
                    continue;
                };
                let rank = u64::try_from(ordinal + 1)
                    .map_err(|_| ServiceError::Contract("context text rank exceeds u64".into()))?;
                let identity = record_identity(&hit.record);
                let evidence = ContextEvidence {
                    kind: ContextEvidenceKind::Text,
                    source: format!("record:{}:commit_bm25", hit.record.kind),
                    source_rank: rank,
                    source_score: hit.score,
                    contribution: reciprocal_rank(TEXT_WEIGHT, rank),
                    plan_sha256: context_digest(&(
                        "rrd-context-commit-bm25-plan-v1",
                        index.as_str(),
                        read.manifest_id.as_str(),
                        request.query.as_str(),
                    ))?,
                    evidence_sha256: context_digest(&(
                        "rrd-context-commit-bm25-evidence-v1",
                        read.manifest_id.as_str(),
                        &identity,
                        &hit.matched_terms,
                        rank,
                        hit.score.to_bits(),
                    ))?,
                };
                add_candidate(
                    &mut candidates,
                    identity,
                    public_properties(&record.properties)?,
                    evidence,
                );
                text_roots.push((hit.record.clone(), rank));
                commit_hits += 1;
            }
        }
        let lexical_selected = text_source_count > 0 || !lexical.hits.is_empty();
        stages.push(context_plan_stage(
            ContextPlanStageKind::Lexical,
            if lexical_selected {
                ContextPlanStageStatus::Selected
            } else {
                ContextPlanStageStatus::Skipped
            },
            lexical_selected.then_some(if lexical.hits.is_empty() {
                ContextAccessPath::SnapshotBm25
            } else {
                ContextAccessPath::CommitBm25
            }),
            lexical_selected,
            if request.query.trim().is_empty() {
                "request supplied no lexical intent".into()
            } else if !lexical_selected {
                "captured temporal snapshot contained no visible textual records or claims".into()
            } else {
                format!(
                    "ranked {} kind(s) from commit-maintained BM25 postings ({commit_hits} hit(s), {} key(s)) and rebuilt BM25 over {text_source_count} other visible textual source(s) at the same stamp",
                    lexical.covered.len(),
                    lexical.keys_read
                )
            },
        )?);

        let mut retrieval_roots = text_roots;
        let mut vector_source_count = 0_usize;
        if !request.query.trim().is_empty() && !snapshot.vectors.is_empty() {
            let (vector_roots, vector_truncated, selected_sources) =
                self.add_vector_context(request, &read, &snapshot, &records, &mut candidates)?;
            retrieval_roots.extend(vector_roots);
            retrieval_truncated |= vector_truncated;
            vector_source_count = selected_sources;
        }
        stages.push(context_plan_stage(
            ContextPlanStageKind::Semantic,
            if vector_source_count == 0 {
                ContextPlanStageStatus::Skipped
            } else {
                ContextPlanStageStatus::Selected
            },
            (vector_source_count > 0).then_some(ContextAccessPath::SnapshotVectorExact),
            vector_source_count > 0,
            if request.query.trim().is_empty() {
                "request supplied no semantic intent".into()
            } else if snapshot.vectors.is_empty() {
                "captured temporal snapshot contained no visible vectors".into()
            } else if vector_source_count == 0 {
                "no visible vector source matched an installed deterministic local text embedding backend"
                    .into()
            } else {
                format!(
                    "exact-scored {vector_source_count} compatible vector source(s) from the captured temporal snapshot"
                )
            },
        )?);

        // Seed lookup and graph expansion share the same temporal visibility.
        // An absent requested record must not regain influence through a live
        // relation. Preserve caller/retrieval order while deduplicating roots.
        let mut unique_roots = graph_roots.iter().cloned().collect::<BTreeSet<_>>();
        for (reference, _) in &retrieval_roots {
            if records.contains_key(reference) && unique_roots.insert(reference.clone()) {
                graph_roots.push(reference.clone());
            }
        }
        let graph_selected = request.max_graph_depth > 0
            && !snapshot.relations.is_empty()
            && !graph_roots.is_empty();
        if graph_selected {
            retrieval_truncated |= add_graph_context(
                request,
                &read,
                &snapshot,
                &records,
                &graph_roots,
                &mut candidates,
            )?;
        }
        stages.push(context_plan_stage(
            ContextPlanStageKind::Graph,
            if graph_selected {
                ContextPlanStageStatus::Selected
            } else {
                ContextPlanStageStatus::Skipped
            },
            graph_selected.then_some(ContextAccessPath::SnapshotGraphBidirectionalBfs),
            graph_selected,
            if request.max_graph_depth == 0 {
                "request disabled graph expansion".into()
            } else if snapshot.relations.is_empty() {
                "captured temporal snapshot contained no visible relations".into()
            } else if graph_roots.is_empty() {
                "no visible seed or retrieval result supplied a graph root".into()
            } else {
                format!(
                    "traversed visible relations in both directions to depth {} from {} root(s)",
                    request.max_graph_depth,
                    graph_roots.len()
                )
            },
        )?);

        stages.push(context_plan_stage(
            ContextPlanStageKind::Fusion,
            ContextPlanStageStatus::Selected,
            Some(ContextAccessPath::ReciprocalRankFusion),
            true,
            "deterministically fused selected retrieval stages with reciprocal-rank fusion".into(),
        )?);

        let mut items = candidates
            .into_iter()
            .map(|(identity, mut candidate)| {
                candidate.evidence.sort_by(|left, right| {
                    left.kind
                        .cmp(&right.kind)
                        .then_with(|| left.source_rank.cmp(&right.source_rank))
                        .then_with(|| left.source.cmp(&right.source))
                        .then_with(|| left.evidence_sha256.cmp(&right.evidence_sha256))
                });
                let score = candidate
                    .evidence
                    .iter()
                    .map(|evidence| evidence.contribution)
                    .sum();
                ContextItem {
                    identity,
                    score,
                    values: candidate.values,
                    evidence: candidate.evidence,
                }
            })
            .collect::<Vec<_>>();
        items.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.identity.cmp(&right.identity))
        });

        let item_limit = usize::try_from(request.max_items)
            .map_err(|_| ServiceError::Contract("context item bound exceeds usize".into()))?;
        let candidate_count = items.len();
        let mut bounded = Vec::with_capacity(items.len().min(item_limit));
        let mut output_bytes = 0_u64;
        let mut truncated = retrieval_truncated || candidate_count > item_limit;
        for item in items.into_iter().take(item_limit) {
            let bytes = serde_json::to_vec(&item)
                .map_err(|error| ServiceError::Contract(error.to_string()))?;
            let item_bytes = u64::try_from(bytes.len())
                .map_err(|_| ServiceError::Contract("context item bytes exceed u64".into()))?;
            if output_bytes.saturating_add(item_bytes) > request.max_output_bytes {
                truncated = true;
                break;
            }
            output_bytes += item_bytes;
            bounded.push(item);
        }

        let context_read = ContextReadStamp {
            runtime_manifest_sha256: read.manifest_id,
            runtime_cursor: read.commit_cursor,
            schema_revision: read.schema_revision,
            catalogue_revision: read.catalog_revision,
        };
        let mut plan = ContextPlanSnapshot {
            request_sha256: context_request_sha256(request)
                .map_err(|error| ServiceError::Contract(error.to_string()))?,
            security_policy_revision,
            authorization_sha256,
            read: context_read.clone(),
            stages,
            plan_sha256: String::new(),
        };
        plan.plan_sha256 = context_plan_sha256(&plan)
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        let mut packet = ContextPacket {
            scope: request.scope.clone(),
            valid_at: request.valid_at,
            result_identity_version: rrd_core::result_identity::RESULT_IDENTITY_VERSION,
            query_sha256: digest::sha256_hex(request.query.as_bytes()),
            read: context_read,
            plan,
            selected_versions,
            read_evidence: crate::runtime::public_read_evidence(read_evidence)?,
            items: bounded,
            output_bytes,
            truncated,
            packet_sha256: String::new(),
        };
        packet.packet_sha256 = context_packet_sha256(&packet)
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        packet
            .validate()
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        Ok(packet)
    }

    fn add_vector_context(
        &self,
        request: &AssembleContext,
        read: &ReadStamp,
        snapshot: &RuntimeDataSnapshot,
        records: &BTreeMap<RuntimeRef, RuntimeRecord>,
        candidates: &mut BTreeMap<String, ContextCandidate>,
    ) -> Result<VectorContextOutcome> {
        let mut eligible = self
            .embedding_backends
            .lock()
            .map_err(|_| ServiceError::Storage("embedding registry lock is poisoned".into()))?
            .descriptors()
            .into_iter()
            .filter(|descriptor| {
                descriptor.deterministic
                    && descriptor.model.modality == rrd_inference::EmbeddingModality::Text
                    && descriptor.network == rrd_inference::NetworkRequirement::None
                    && matches!(
                        descriptor.execution,
                        rrd_inference::ExecutionTarget::Cpu
                            | rrd_inference::ExecutionTarget::Gpu { .. }
                    )
            })
            .collect::<Vec<_>>();
        eligible.sort_by(|left, right| left.id.cmp(&right.id));
        if eligible.is_empty() {
            return Ok((Vec::new(), false, 0));
        }
        let scope = self.query_scope(&request.scope)?;
        let catalogue = rrd_vector::VectorCollectionRepository::new(&self.storage, scope.clone())
            .load()
            .map_err(super::super::vector::vector_collection_error)?;
        if catalogue.revision != read.catalog_revision {
            return Err(ServiceError::StorageConflict(format!(
                "context vector catalogue revision {} differs from captured revision {}",
                catalogue.revision, read.catalog_revision
            )));
        }

        let mut vectors = BTreeMap::<(String, String), Vec<&RuntimeVector>>::new();
        for entry in &snapshot.vectors {
            let address = &entry.value.collection;
            vectors
                .entry((address.collection_id.clone(), address.vector_name.clone()))
                .or_default()
                .push(&entry.value);
        }
        for entries in vectors.values_mut() {
            entries.sort_by(|left, right| {
                left.subject
                    .cmp(&right.subject)
                    .then_with(|| left.reference.cmp(&right.reference))
            });
        }

        let mut generated = BTreeMap::<String, Vec<f32>>::new();
        let mut roots = Vec::new();
        let source_limit = usize::try_from(request.max_items)
            .map_err(|_| ServiceError::Contract("context item bound exceeds usize".into()))?;
        let mut sources = 0_usize;
        let mut truncated = false;
        'collections: for collection in catalogue.collections.values() {
            for vector in collection.definition.vectors.values() {
                if vector.kind != rrd_vector::VectorValueKind::Dense {
                    continue;
                }
                let Some(binding) = &vector.embedding_model else {
                    continue;
                };
                let Some(descriptor) = eligible.iter().find(|descriptor| {
                    descriptor.model.dimensions == vector.dimensions
                        && descriptor.model.canonical_name() == binding.name
                        && descriptor.model.model_digest == binding.digest
                }) else {
                    continue;
                };
                let address = (
                    collection.definition.id.as_str().to_owned(),
                    vector.name.as_str().to_owned(),
                );
                let Some(source_vectors) = vectors.get(&address) else {
                    continue;
                };
                if sources >= source_limit {
                    truncated = true;
                    break 'collections;
                }
                sources += 1;
                let query_vector = if let Some(values) = generated.get(&descriptor.id) {
                    values.clone()
                } else {
                    let result = self.generate_embeddings_at(
                        &rrd_contract::GenerateEmbeddings {
                            scope: request.scope.clone(),
                            backend_id: descriptor.id.clone(),
                            network_policy: rrd_contract::EmbeddingNetworkPolicy::Deny,
                            inputs: vec![rrd_contract::EmbeddingInput {
                                id: CanonicalId::new("context-query")
                                    .map_err(|error| ServiceError::Contract(error.to_string()))?,
                                media_type: "text/plain; charset=utf-8".into(),
                                bytes: request.query.as_bytes().to_vec(),
                            }],
                        },
                        read.clone(),
                    )?;
                    if result.read_manifest_sha256 != read.manifest_id
                        || result.known_at_cursor != read.commit_cursor
                    {
                        return Err(ServiceError::StorageConflict(
                            "context embedding advanced beyond its captured read stamp".into(),
                        ));
                    }
                    let embedding = result.embeddings.into_iter().next().ok_or_else(|| {
                        ServiceError::Inference(
                            "context embedding backend returned no query vector".into(),
                        )
                    })?;
                    let DataVectorValue::Dense { values } = embedding.value else {
                        return Err(ServiceError::Inference(
                            "context vector source requires dense embeddings".into(),
                        ));
                    };
                    generated.insert(descriptor.id.clone(), values.clone());
                    values
                };
                let query = rrd_vector::VectorQuery::Dense {
                    values: query_vector,
                };
                let mut scored = Vec::<(&RuntimeVector, &RuntimeRecord, f64)>::new();
                for runtime_vector in source_vectors {
                    let Some(record) = records.get(&runtime_vector.subject) else {
                        continue;
                    };
                    let score =
                        rrd_vector::score_query(&query, &runtime_vector.value, vector.metric)
                            .map_err(|error| ServiceError::Vector(error.to_string()))?;
                    if !score.is_finite() {
                        return Err(ServiceError::Vector(
                            "context vector score is not finite".into(),
                        ));
                    }
                    scored.push((runtime_vector, record, score));
                }
                scored.sort_by(|left, right| {
                    right
                        .2
                        .total_cmp(&left.2)
                        .then_with(|| left.0.subject.cmp(&right.0.subject))
                        .then_with(|| left.0.reference.cmp(&right.0.reference))
                });
                if scored.len() > source_limit {
                    truncated = true;
                    scored.truncate(source_limit);
                }
                let plan_sha256 = context_digest(&(
                    "rrd-context-vector-exact-snapshot-v1",
                    read.manifest_id.as_str(),
                    catalogue.revision,
                    &collection.definition.id,
                    vector,
                    &descriptor.id,
                ))?;
                for (ordinal, (runtime_vector, record, score)) in scored.into_iter().enumerate() {
                    let rank = u64::try_from(ordinal + 1).map_err(|_| {
                        ServiceError::Contract("context vector rank exceeds u64".into())
                    })?;
                    let evidence = ContextEvidence {
                        kind: ContextEvidenceKind::Vector,
                        source: format!("vector:{}:{}", collection.definition.id, vector.name),
                        source_rank: rank,
                        source_score: score,
                        contribution: reciprocal_rank(VECTOR_WEIGHT, rank),
                        plan_sha256: plan_sha256.clone(),
                        evidence_sha256: context_digest(&(
                            "rrd-context-vector-snapshot-evidence-v1",
                            read.manifest_id.as_str(),
                            &runtime_vector.reference,
                            &runtime_vector.subject,
                            &runtime_vector.value,
                            score.to_bits(),
                        ))?,
                    };
                    add_record_candidate(candidates, &runtime_vector.subject, record, evidence)?;
                    roots.push((runtime_vector.subject.clone(), rank));
                }
            }
        }
        Ok((roots, truncated, sources))
    }
}
