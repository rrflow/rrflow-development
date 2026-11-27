use super::*;

impl RrdEngine {
    #[allow(clippy::too_many_arguments)]
    pub fn execute_query(
        &self,
        session_id: &CorrelationId,
        token: &CorrelationId,
        request: &ExecuteQuery,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<QueryResult> {
        self.execute_query_scoped(
            session_id,
            token,
            request,
            &self.instance_resource(),
            now,
            request_id,
            operation_id,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn execute_query_scoped(
        &self,
        session_id: &CorrelationId,
        token: &CorrelationId,
        request: &ExecuteQuery,
        resource: &ResourcePath,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<QueryResult> {
        request
            .validate()
            .map_err(|error| ServiceError::Query(error.to_string()))?;
        let (_, _, authorization) = self.authorize_resource(
            session_id,
            token,
            SecurityAction::QueryExecute,
            resource,
            now,
            request_id,
            operation_id,
        )?;
        let configuration = self.enforce_query_configuration("query.execute", &request.budget)?;
        let (security_policy_revision, authorization_sha256, data_policy) = authorization
            .map_or_else(
                || {
                    (
                        0,
                        digest::sha256_hex(b"rrd-security-disabled-loopback-development"),
                        None,
                    )
                },
                |authorization| {
                    (
                        authorization.policy_revision,
                        authorization.authorization_sha256,
                        authorization.data_policy,
                    )
                },
            );
        let expected_scope = format!("instance:{}", self.instance);
        if request.scope != expected_scope {
            return Err(ServiceError::WrongScope);
        }
        let scope = ScopeId::new(request.scope.clone())
            .map_err(|error| ServiceError::Query(error.to_string()))?;
        let mut query = rrflow_ql::parse(&request.query)
            .map_err(|error| ServiceError::Query(error.to_string()))?;
        if let Some(policy) = &data_policy {
            apply_query_policy(&mut query, policy)?;
        }
        let parameters = request
            .parameters
            .iter()
            .map(|(name, value)| Ok((name.clone(), runtime_value(value)?)))
            .collect::<Result<rrflow_ql::Parameters>>()?;
        let budget = query_execution_budget(&request.budget)?;
        let read = self
            .storage
            .runtime()
            .read_stamp(&scope)
            .map_err(super::error::query_storage_error)?;
        let pipeline = rrflow_ql::StampedQueryPipeline::new(&self.storage, read)
            .map_err(ServiceError::from)?;
        let stamped = pipeline
            .run(&query, &parameters, &budget)
            .map_err(ServiceError::from)?;
        let bound = stamped.bound;
        let plan = stamped.plan;
        let execution = stamped.execution;
        let rows = execution
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .map(|row| {
                Ok(QueryRowSnapshot {
                    identity: row.identity.clone(),
                    values: row
                        .values
                        .iter()
                        .map(|(name, value)| Ok((name.clone(), query_value(value)?)))
                        .collect::<Result<_>>()?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let selected_versions = u64::try_from(execution.selected_versions)
            .map_err(|_| ServiceError::Query("selected versions exceed u64".into()))?;
        let analysis = execution
            .analysis
            .as_ref()
            .map(public_query_analysis)
            .transpose()?;
        let read_evidence = crate::runtime::public_read_evidence(execution.read_evidence)?;
        Ok(QueryResult {
            configuration,
            canonical_query: query.canonical(),
            scope: request.scope.clone(),
            read_manifest_sha256: execution.read_manifest.clone(),
            known_at_cursor: execution.known_at_cursor,
            result_identity_version: rrflow_core::result_identity::RESULT_IDENTITY_VERSION,
            schema_revision: bound.schema_revision,
            plan: QueryPlanSnapshot {
                plan_sha256: plan.digest.clone(),
                security_policy_revision,
                authorization_sha256,
                exact: plan.explanation.contract.exact,
                deterministic_order: plan.explanation.contract.deterministic_order.clone(),
                authorization_boundary: plan.explanation.contract.authorization_boundary.clone(),
                candidates: plan
                    .explanation
                    .candidates
                    .iter()
                    .map(|candidate| QueryPlanCandidate {
                        name: candidate.name.clone(),
                        selected: candidate.selected,
                        exact: candidate.exact,
                        reason: candidate.reason.clone(),
                    })
                    .collect(),
            },
            execution: QueryExecutionSnapshot {
                selected_versions,
                read_evidence,
                returned_rows: u64::try_from(execution.returned_rows)
                    .map_err(|_| ServiceError::Query("returned rows exceed u64".into()))?,
                output_bytes: u64::try_from(execution.output_bytes)
                    .map_err(|_| ServiceError::Query("output bytes exceed u64".into()))?,
                truncated: execution.truncated,
                analysis,
            },
            rows,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn poll_live_query(
        &self,
        session_id: &CorrelationId,
        token: &CorrelationId,
        request: &PollLiveQuery,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<LiveQueryDeltaResult> {
        request
            .validate()
            .map_err(|error| ServiceError::Query(error.to_string()))?;
        self.authorize(
            session_id,
            token,
            SecurityAction::QueryLivePoll,
            now,
            request_id,
            operation_id,
        )?;
        self.poll_live_query_page(request)
    }

    pub(in crate::engine) fn poll_live_query_page(
        &self,
        request: &PollLiveQuery,
    ) -> Result<LiveQueryDeltaResult> {
        request
            .validate()
            .map_err(|error| ServiceError::Query(error.to_string()))?;
        self.enforce_query_configuration("query.live.page", &request.budget)?;
        let expected_scope = format!("instance:{}", self.instance);
        if request.scope != expected_scope {
            return Err(ServiceError::WrongScope);
        }
        let scope = ScopeId::new(request.scope.clone())
            .map_err(|error| ServiceError::Query(error.to_string()))?;
        let query = rrflow_ql::parse(&request.query)
            .map_err(|error| ServiceError::Query(error.to_string()))?;
        let parameters = request
            .parameters
            .iter()
            .map(|(name, value)| Ok((name.clone(), runtime_value(value)?)))
            .collect::<Result<rrflow_ql::Parameters>>()?;
        let live_budget = rrflow_ql::LiveQueryBudget {
            execution: query_execution_budget(&request.budget)?,
            max_delta_rows: usize::try_from(request.max_delta_rows)
                .map_err(|_| ServiceError::Query("live query delta budget exceeds usize".into()))?,
        };
        let started = Instant::now();
        let timeout = Duration::from_millis(request.wait_timeout_ms);
        let (delta, timed_out, waited_ms) = loop {
            let delta = rrflow_ql::poll_live_query(
                &self.storage,
                &scope,
                &query,
                &parameters,
                request.after_cursor,
                request.result_identity_version,
                &live_budget,
            )
            .map_err(ServiceError::from)?;
            let elapsed = started.elapsed();
            if delta.through_cursor > request.after_cursor
                || request.wait_timeout_ms == 0
                || elapsed >= timeout
            {
                let timed_out = delta.through_cursor == request.after_cursor
                    && delta.added.is_empty()
                    && delta.updated.is_empty()
                    && delta.removed.is_empty()
                    && request.wait_timeout_ms > 0
                    && elapsed >= timeout;
                break (
                    delta,
                    timed_out,
                    u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX),
                );
            }
            std::thread::sleep(
                timeout
                    .saturating_sub(elapsed)
                    .min(Duration::from_millis(25)),
            );
        };
        Ok(LiveQueryDeltaResult {
            timed_out,
            waited_ms,
            query_sha256: delta.query_digest,
            result_identity_version: delta.result_identity_version,
            from_cursor: delta.from_cursor,
            through_cursor: delta.through_cursor,
            head_cursor: delta.head_cursor,
            added: delta
                .added
                .iter()
                .map(public_query_row)
                .collect::<Result<_>>()?,
            updated: delta
                .updated
                .iter()
                .map(|change| {
                    Ok(LiveQueryRowChange {
                        before: public_query_row(&change.before)?,
                        after: public_query_row(&change.after)?,
                    })
                })
                .collect::<Result<_>>()?,
            removed: delta
                .removed
                .iter()
                .map(public_query_row)
                .collect::<Result<_>>()?,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ensure_query_index(
        &self,
        session_id: &CorrelationId,
        token: &CorrelationId,
        idempotency_key: &CorrelationId,
        request: &EnsureQueryIndex,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<EnsureQueryIndexResult> {
        request
            .validate()
            .map_err(|error| ServiceError::Query(error.to_string()))?;
        let (session_bytes, mut session) = self.load_authenticated(session_id, token)?;
        self.authorize_session_policy(&session, SecurityAction::QueryIndexEnsure, now)?;
        self.enforce_query_configuration("query.index.ensure", &request.budget)?;
        let scope = self.query_scope(&request.scope)?;
        let operation_digest = digest::sha256_hex(
            &serde_json::to_vec(request).map_err(|error| ServiceError::Query(error.to_string()))?,
        );
        let (definition, valid_at) = query_index_definition(request)?;
        let query_catalogue = rrflow_ql::Catalog::capture_for_sources(
            &self.storage,
            &scope,
            std::slice::from_ref(&definition.source),
            rrflow_db::RuntimeReadBudget::new(request.budget.max_storage_keys)
                .map_err(|error| ServiceError::Query(error.to_string()))?,
        )
        .map_err(|error| ServiceError::Query(error.to_string()))?;
        let selected_versions = u64::try_from(query_catalogue.selected_versions)
            .map_err(|_| ServiceError::Query("selected versions exceed u64".into()))?;
        let read_evidence =
            crate::runtime::public_read_evidence(query_catalogue.read_evidence.clone())?;
        let repository = rrflow_ql::IndexCatalogueRepository::new(&self.storage, scope.clone());
        if let Some(receipt) = repository
            .operation_receipt(idempotency_key.as_str(), &operation_digest)
            .map_err(query_index_error)?
        {
            let catalogue = repository
                .load()
                .map_err(|error| ServiceError::Query(error.to_string()))?;
            return Ok(EnsureQueryIndexResult {
                index: public_query_index(&receipt.entry)?,
                catalogue_revision: catalogue.revision,
                selected_versions,
                read_evidence,
                idempotent_replay: true,
            });
        }
        // An exact durable operation replay above remains recoverable after
        // the adapter session expires. Creating or rebuilding an index is a
        // new mutation and therefore still requires an active session.
        self.require_active_or_expire(
            session_id,
            session_bytes,
            &mut session,
            now,
            request_id,
            operation_id,
        )?;
        let preliminary = repository
            .load()
            .map_err(|error| ServiceError::Query(error.to_string()))?;
        let requires_unique_install = definition.unique
            && preliminary
                .entries
                .get(&definition.id)
                .is_none_or(|entry| !entry.unique_validated);
        // Ordinary rebuilds never enter the authoritative commit path. The
        // first installation of a unique constraint is the exception: it must
        // prove existing rows and publish its enforcement bit without a write
        // racing between those two events.
        let _transaction_guard = requires_unique_install
            .then(|| self.admit_transaction(&definition, request_id, operation_id))
            .transpose()?;
        let current = if requires_unique_install {
            repository
                .load()
                .map_err(|error| ServiceError::Query(error.to_string()))?
        } else {
            preliminary
        };
        let source_cursor = query_catalogue.source_cursor(&definition.source);
        let mutation = rrflow_ql::IndexMutationContext {
            at: now,
            actor: format!("session:{}", session_id.as_str()),
            request_id: request_id.into(),
            operation_id: operation_id.into(),
        };
        if let Some(existing) = current.entries.get(&definition.id) {
            if existing.definition != definition {
                return Err(ServiceError::Query(format!(
                    "index {} already exists with a different definition",
                    definition.id
                )));
            }
            match existing.stamp.state {
                ProjectionState::Building => {}
                ProjectionState::Ready if existing.is_usable_at(source_cursor, valid_at) => {}
                ProjectionState::Ready => {
                    repository
                        .begin_rebuild(&mutation, &definition.id)
                        .map_err(|error| ServiceError::Query(error.to_string()))?;
                }
                ProjectionState::Quarantined | ProjectionState::Retiring => {
                    return Err(ServiceError::Query(format!(
                        "index {} must be explicitly recovered before ensure",
                        definition.id
                    )));
                }
            }
        } else {
            repository
                .create(&mutation, &query_catalogue, definition.clone())
                .map_err(|error| ServiceError::Query(error.to_string()))?;
        }
        let catalogue = repository
            .load()
            .map_err(|error| ServiceError::Query(error.to_string()))?;
        let entry = catalogue
            .entries
            .get(&definition.id)
            .ok_or_else(|| ServiceError::Query("created index disappeared".into()))?;
        let ready = if entry.is_usable_at(source_cursor, valid_at) {
            catalogue
        } else {
            repository
                .build(
                    &mutation,
                    &definition.id,
                    valid_at,
                    &query_execution_budget(&request.budget)?,
                )
                .map_err(|error| ServiceError::Query(error.to_string()))?
        };
        let entry = ready
            .entries
            .get(&definition.id)
            .cloned()
            .ok_or_else(|| ServiceError::Query("ready index disappeared".into()))?;
        let recorded = repository
            .record_operation(
                &mutation,
                idempotency_key.as_str().into(),
                operation_digest,
                entry.clone(),
            )
            .map_err(query_index_error)?;
        Ok(EnsureQueryIndexResult {
            index: public_query_index(&entry)?,
            catalogue_revision: recorded.revision,
            selected_versions,
            read_evidence,
            idempotent_replay: false,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn list_query_indexes(
        &self,
        session_id: &CorrelationId,
        token: &CorrelationId,
        request: &ListQueryIndexes,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<QueryIndexCatalogueSnapshot> {
        request
            .validate()
            .map_err(|error| ServiceError::Query(error.to_string()))?;
        self.authorize(
            session_id,
            token,
            SecurityAction::QueryIndexList,
            now,
            request_id,
            operation_id,
        )?;
        let scope = self.query_scope(&request.scope)?;
        let catalogue = rrflow_ql::IndexCatalogueRepository::new(&self.storage, scope)
            .load()
            .map_err(|error| ServiceError::Query(error.to_string()))?;
        Ok(QueryIndexCatalogueSnapshot {
            scope: request.scope.clone(),
            revision: catalogue.revision,
            indexes: catalogue
                .entries
                .values()
                .map(public_query_index)
                .collect::<Result<_>>()?,
        })
    }
}

fn apply_query_policy(
    query: &mut rrflow_ql::Query,
    policy: &rrflow_security::DataPolicy,
) -> Result<()> {
    for predicate in policy.predicates() {
        query.filters.push(Filter {
            field: predicate.field.clone(),
            comparison: ComparisonOperator::Equal,
            value: ValueExpr::Literal(predicate.value.clone()),
        });
    }
    if let Some(allowed) = &policy.allowed_fields {
        match &query.projection {
            Projection::All => {
                query.projection = Projection::Fields(allowed.iter().cloned().collect())
            }
            Projection::Fields(requested)
                if requested.iter().all(|field| allowed.contains(field)) => {}
            Projection::Fields(_) => return Err(ServiceError::PermissionDenied),
        }
    }
    query
        .validate()
        .map_err(|error| ServiceError::Query(error.to_string()))
}

fn public_query_row(row: &rrflow_ql::QueryRow) -> Result<QueryRowSnapshot> {
    Ok(QueryRowSnapshot {
        identity: row.identity.clone(),
        values: row
            .values
            .iter()
            .map(|(name, value)| Ok((name.clone(), query_value(value)?)))
            .collect::<Result<_>>()?,
    })
}

fn query_execution_budget(
    budget: &rrd_contract::QueryBudget,
) -> Result<rrflow_ql::ExecutionBudget> {
    let max_storage_keys = usize::try_from(budget.max_storage_keys)
        .map_err(|_| ServiceError::Query("query key budget exceeds usize".into()))?;
    Ok(rrflow_ql::ExecutionBudget {
        max_storage_keys: budget.max_storage_keys,
        max_input_rows: max_storage_keys,
        max_rows: usize::try_from(budget.max_rows)
            .map_err(|_| ServiceError::Query("query row budget exceeds usize".into()))?,
        max_output_bytes: usize::try_from(budget.max_output_bytes)
            .map_err(|_| ServiceError::Query("query output budget exceeds usize".into()))?,
        max_batch_rows: usize::try_from(budget.max_batch_rows)
            .map_err(|_| ServiceError::Query("query batch budget exceeds usize".into()))?,
        max_memory_bytes: usize::try_from(budget.max_memory_bytes)
            .map_err(|_| ServiceError::Query("query memory budget exceeds usize".into()))?,
        max_spill_bytes: usize::try_from(budget.max_spill_bytes)
            .map_err(|_| ServiceError::Query("query spill budget exceeds usize".into()))?,
        max_elapsed_ms: budget.max_elapsed_ms,
    })
}

fn public_query_analysis(
    analysis: &rrflow_ql::FusionAnalysis,
) -> Result<QueryExecutionAnalysisSnapshot> {
    let number = |value: usize, label: &str| {
        u64::try_from(value).map_err(|_| ServiceError::Query(format!("query {label} exceeds u64")))
    };
    Ok(QueryExecutionAnalysisSnapshot {
        engine: analysis.engine.clone(),
        provider_scans: number(analysis.provider_scans, "provider scans")?,
        input_rows: number(analysis.input_rows, "analysis input rows")?,
        input_batches: number(analysis.input_batches, "analysis input batches")?,
        input_memory_bytes: number(analysis.input_memory_bytes, "analysis input memory")?,
        output_batches: number(analysis.output_batches, "analysis output batches")?,
        projection_pushdown: analysis.projection_pushdown.clone(),
        filter_pushdown: analysis.filter_pushdown.clone(),
        limit_pushdown: analysis.limit_pushdown.clone(),
        physical_operators: number(analysis.physical_operators, "physical operators")?,
        peak_memory_bytes: number(analysis.peak_memory_bytes, "peak memory bytes")?,
        spill_count: number(analysis.spill_count, "spill count")?,
        spilled_bytes: number(analysis.spilled_bytes, "spilled bytes")?,
        spilled_rows: number(analysis.spilled_rows, "spilled rows")?,
        elapsed_micros: analysis.elapsed_micros,
    })
}

fn query_index_definition(request: &EnsureQueryIndex) -> Result<(rrflow_ql::IndexDefinition, u64)> {
    let query = rrflow_ql::parse(&request.definition_query)
        .map_err(|error| ServiceError::Query(error.to_string()))?;
    if query.join.is_some()
        || query.limit.is_some()
        || query.explain_contract
        || query.explain_analyze
        || !matches!(query.temporal.known_at, CursorExpr::Head)
    {
        return Err(ServiceError::Query(
            "index definition query requires KNOWN HEAD and cannot contain joins, limits, or EXPLAIN"
                .into(),
        ));
    }
    let TimeExpr::Literal(valid_at) = query.temporal.valid_at else {
        return Err(ServiceError::Query(
            "index definition valid-time must be a literal".into(),
        ));
    };
    let fields = match query.projection {
        Projection::Fields(fields) => fields,
        Projection::All if request.kind == QueryIndexKind::Count => Vec::new(),
        Projection::All => return Err(ServiceError::Query(
            "only a count index may use PROJECT *; other index definitions require named fields"
                .into(),
        )),
    };
    let full_text = request.full_text.clone().unwrap_or_default();
    Ok((
        rrflow_ql::IndexDefinition {
            id: ProjectionId::new(request.index_id.as_str())
                .map_err(|error| ServiceError::Query(error.to_string()))?,
            source: query.source,
            fields,
            unique: request.unique,
            kind: match request.kind {
                QueryIndexKind::Scalar => rrflow_ql::IndexKind::Scalar,
                QueryIndexKind::Count => rrflow_ql::IndexKind::Count,
                QueryIndexKind::Geo => rrflow_ql::IndexKind::Geo,
                QueryIndexKind::MaterializedView => rrflow_ql::IndexKind::MaterializedView,
                QueryIndexKind::AggregateCount => rrflow_ql::IndexKind::AggregateCount,
                QueryIndexKind::Bm25 => rrflow_ql::IndexKind::Bm25 {
                    config: rrflow_ql::Bm25Config {
                        analyzer: match full_text.analyzer {
                            QueryTextAnalyzer::UnicodeLowercase => {
                                rrflow_ql::Bm25Analyzer::UnicodeLowercase
                            }
                            QueryTextAnalyzer::UnicodeCaseSensitive => {
                                rrflow_ql::Bm25Analyzer::UnicodeCaseSensitive
                            }
                        },
                        tokenizer: match full_text.tokenizer {
                            QueryTextTokenizer::UnicodeAlphanumeric => {
                                rrflow_ql::Bm25Tokenizer::UnicodeAlphanumeric
                            }
                            QueryTextTokenizer::Whitespace => rrflow_ql::Bm25Tokenizer::Whitespace,
                        },
                        ascii_folding: full_text.ascii_folding,
                        stop_words: full_text.stop_words,
                        min_token_chars: full_text.min_token_chars,
                        max_token_chars: full_text.max_token_chars,
                        stemmer: match full_text.stemmer {
                            QueryTextStemmer::None => rrflow_ql::Bm25Stemmer::None,
                            QueryTextStemmer::English => rrflow_ql::Bm25Stemmer::English,
                        },
                        identifiers: match full_text.identifiers {
                            QueryTextIdentifiers::Pieces => rrflow_ql::Bm25Identifiers::Pieces,
                            QueryTextIdentifiers::WholeAndParts => {
                                rrflow_ql::Bm25Identifiers::WholeAndParts
                            }
                        },
                        k1_micros: full_text.k1_micros,
                        b_micros: full_text.b_micros,
                    },
                },
            },
            filters: query.filters,
        },
        valid_at,
    ))
}

pub(in crate::engine) fn public_query_index(
    entry: &rrflow_ql::IndexEntry,
) -> Result<QueryIndexSnapshot> {
    let valid_at = entry.built_valid_at.unwrap_or(0);
    let mut definition = rrflow_ql::Query::new(
        entry.definition.source.clone(),
        rrflow_ql::TemporalSelector {
            valid_at: TimeExpr::Literal(valid_at),
            known_at: CursorExpr::Head,
        },
    );
    definition.projection = Projection::Fields(entry.definition.fields.clone());
    definition.filters = entry.definition.filters.clone();
    if entry.definition.fields.is_empty() {
        definition.projection = Projection::All;
    }
    let full_text = match &entry.definition.kind {
        rrflow_ql::IndexKind::Bm25 { config } => Some(QueryFullTextConfiguration {
            analyzer: match config.analyzer {
                rrflow_ql::Bm25Analyzer::UnicodeLowercase => QueryTextAnalyzer::UnicodeLowercase,
                rrflow_ql::Bm25Analyzer::UnicodeCaseSensitive => {
                    QueryTextAnalyzer::UnicodeCaseSensitive
                }
            },
            tokenizer: match config.tokenizer {
                rrflow_ql::Bm25Tokenizer::UnicodeAlphanumeric => {
                    QueryTextTokenizer::UnicodeAlphanumeric
                }
                rrflow_ql::Bm25Tokenizer::Whitespace => QueryTextTokenizer::Whitespace,
            },
            ascii_folding: config.ascii_folding,
            stop_words: config.stop_words.clone(),
            min_token_chars: config.min_token_chars,
            max_token_chars: config.max_token_chars,
            stemmer: match config.stemmer {
                rrflow_ql::Bm25Stemmer::None => QueryTextStemmer::None,
                rrflow_ql::Bm25Stemmer::English => QueryTextStemmer::English,
            },
            identifiers: match config.identifiers {
                rrflow_ql::Bm25Identifiers::Pieces => QueryTextIdentifiers::Pieces,
                rrflow_ql::Bm25Identifiers::WholeAndParts => QueryTextIdentifiers::WholeAndParts,
            },
            k1_micros: config.k1_micros,
            b_micros: config.b_micros,
        }),
        _ => None,
    };
    Ok(QueryIndexSnapshot {
        index_id: CanonicalId::new(entry.definition.id.to_string())
            .map_err(|error| ServiceError::Query(error.to_string()))?,
        definition_query: definition.canonical(),
        unique: entry.definition.unique,
        kind: match &entry.definition.kind {
            rrflow_ql::IndexKind::Scalar => QueryIndexKind::Scalar,
            rrflow_ql::IndexKind::Count => QueryIndexKind::Count,
            rrflow_ql::IndexKind::Geo => QueryIndexKind::Geo,
            rrflow_ql::IndexKind::MaterializedView => QueryIndexKind::MaterializedView,
            rrflow_ql::IndexKind::AggregateCount => QueryIndexKind::AggregateCount,
            rrflow_ql::IndexKind::Bm25 { .. } => QueryIndexKind::Bm25,
        },
        full_text,
        generation: entry.stamp.generation,
        source_cursor: entry.stamp.source_cursor,
        built_valid_at: entry.built_valid_at,
        artifact_rows: entry.artifact_rows,
        result_identity_version: entry.result_identity_version,
        identity_rebuild_required: entry.stamp.state == rrflow_core::ProjectionState::Ready
            && entry.result_identity_version
                != Some(rrflow_core::result_identity::RESULT_IDENTITY_VERSION),
        analytics_total_count: entry.analytics_total_count,
        analytics_group_count: entry.analytics_group_count,
        maintenance: entry
            .maintenance
            .as_ref()
            .map(|maintenance| QueryIndexMaintenanceSnapshot {
                mode: maintenance.mode.clone(),
                prior_source_cursor: maintenance.prior_source_cursor,
                source_cursor: maintenance.source_cursor,
                inserted_rows: maintenance.inserted_rows,
                updated_rows: maintenance.updated_rows,
                removed_rows: maintenance.removed_rows,
            }),
        configuration_sha256: entry.stamp.config_digest.clone(),
        artifact_sha256: entry.stamp.artifact_digest.clone(),
        state: match entry.stamp.state {
            ProjectionState::Building => QueryIndexState::Building,
            ProjectionState::Ready => QueryIndexState::Ready,
            ProjectionState::Quarantined => QueryIndexState::Quarantined,
            ProjectionState::Retiring => QueryIndexState::Retiring,
        },
    })
}

fn query_index_error(error: rrflow_ql::Error) -> ServiceError {
    if matches!(&error, rrflow_ql::Error::Catalog(message) if message.contains("idempotency key")) {
        ServiceError::IdempotencyConflict
    } else {
        ServiceError::Query(error.to_string())
    }
}
