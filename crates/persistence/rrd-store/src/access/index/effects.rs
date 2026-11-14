//! Synchronizing bindings and encoding each record's index effects at commit.

use super::*;

pub(crate) fn synchronize_index_commit_bindings(
    transaction: &mut dyn StorageTransaction,
    scope: &ScopeId,
    expected_schema_revision: u64,
    catalogue_revision: u64,
    source_control_key: &str,
    source_control_sha256: &str,
    definitions: &[IndexCommitBindingDefinition],
) -> Result<()> {
    validate_sha256("index source catalogue", source_control_sha256)?;
    let schema: RuntimeSchemaRegistry = get_json(
        &*transaction,
        keyspaces::RUNTIME_SCHEMAS,
        &keyspaces::runtime_schema_key(scope),
    )?
    .ok_or_else(|| Error::RuntimeSchemaMissing(scope.to_string()))?;
    if schema.revision != expected_schema_revision {
        return Err(Error::RuntimeSchemaConflict {
            expected: expected_schema_revision,
            actual: schema.revision,
        });
    }

    let mut requested = BTreeMap::new();
    for definition in definitions {
        definition.validate()?;
        validate_binding_schema(definition, &schema)?;
        if requested
            .insert(definition.id.clone(), definition.clone())
            .is_some()
        {
            return Err(Error::IndexConstraint(format!(
                "duplicate commit binding for index {}",
                definition.id
            )));
        }
    }
    let existing = raw_bindings_for_scope(&*transaction, scope)?
        .into_iter()
        .map(|binding| (binding.definition.id.clone(), binding))
        .collect::<BTreeMap<_, _>>();
    if let Some(foreign) = existing
        .values()
        .find(|binding| binding.source_control_key != source_control_key)
    {
        return Err(Error::IndexConstraint(format!(
            "index {} is bound to a different catalogue authority",
            foreign.definition.id
        )));
    }

    for (id, prior) in &existing {
        if !requested.contains_key(id) {
            clear_index_entries(transaction, scope, prior)?;
            transaction.delete(checked_key(
                keyspaces::RUNTIME_INDEX_BINDINGS,
                &keyspaces::runtime_index_binding_key(scope, id),
            )?)?;
        }
    }

    let canonical_definitions = requested.values().cloned().collect::<Vec<_>>();
    for definition in requested.into_values() {
        let binding = IndexCommitBinding {
            contract_version: INDEX_COMMIT_BINDING_CONTRACT_VERSION,
            scope: scope.clone(),
            definition,
            schema_revision: expected_schema_revision,
            catalogue_revision,
            source_control_key: source_control_key.to_owned(),
            source_control_sha256: source_control_sha256.to_owned(),
        };
        binding.validate()?;
        let requires_backfill = existing
            .get(&binding.definition.id)
            .is_none_or(|prior| prior.definition != binding.definition);
        if let Some(prior) = existing.get(&binding.definition.id) {
            if requires_backfill {
                clear_index_entries(transaction, scope, prior)?;
            }
        }
        if requires_backfill {
            backfill_index_entries(transaction, &binding)?;
        }
        put_json(
            transaction,
            keyspaces::RUNTIME_INDEX_BINDINGS,
            &keyspaces::runtime_index_binding_key(scope, &binding.definition.id),
            &binding,
        )?;
        put_json(
            transaction,
            keyspaces::RUNTIME_INDEX_BINDING_VERSIONS,
            &keyspaces::runtime_index_binding_version_key(
                scope,
                &binding.definition.id,
                catalogue_revision,
                expected_schema_revision,
            ),
            &binding,
        )?;
    }
    let binding_set = IndexCommitBindingSet {
        contract_version: INDEX_COMMIT_BINDING_CONTRACT_VERSION,
        scope: scope.clone(),
        schema_revision: expected_schema_revision,
        catalogue_revision,
        source_control_key: source_control_key.to_owned(),
        source_control_sha256: source_control_sha256.to_owned(),
        definition_count: u64::try_from(canonical_definitions.len())
            .map_err(|_| Error::IndexConstraint("index binding count exceeds u64".into()))?,
        definitions_sha256: definitions_sha256(&canonical_definitions)?,
    };
    binding_set.validate()?;
    put_json(
        transaction,
        keyspaces::RUNTIME_INDEX_BINDING_SETS,
        &keyspaces::runtime_index_binding_set_key(scope),
        &binding_set,
    )?;
    Ok(())
}

pub(crate) fn encode_record_index_effects(
    reader: &(impl super::super::runtime_state::AccessRead + ?Sized),
    plan: &mut SemanticCommitPlan,
    commit: &RuntimeCommit,
    schema: &RuntimeSchemaRegistry,
    catalogue_revision: u64,
    starting_cursor: u64,
) -> Result<()> {
    let source_control_key = format!("server/state/index-catalogue/{}", commit.scope);
    let source_control = get(
        reader,
        keyspaces::META,
        &keyspaces::control_record_key(&source_control_key),
    )?;
    let mut binding_set: Option<IndexCommitBindingSet> = get_json(
        reader,
        keyspaces::RUNTIME_INDEX_BINDING_SETS,
        &keyspaces::runtime_index_binding_set_key(&commit.scope),
    )?;
    let mut bindings = raw_bindings_for_scope(reader, &commit.scope)?;
    match (&source_control, &binding_set) {
        (None, None) if bindings.is_empty() => return Ok(()),
        (None, None) => {
            return Err(Error::IndexConstraint(format!(
                "scope {} has orphaned index commit bindings",
                commit.scope
            )));
        }
        (Some(_), None) => {
            return Err(Error::IndexConstraint(format!(
                "scope {} index catalogue has no atomic commit-binding set",
                commit.scope
            )));
        }
        (None, Some(_)) => {
            return Err(Error::IndexConstraint(format!(
                "scope {} index commit-binding set has no source catalogue",
                commit.scope
            )));
        }
        (Some(_), Some(_)) => {}
    }
    let source_control = source_control.expect("matched source catalogue bytes");
    let binding_set = binding_set
        .as_mut()
        .expect("matched index commit-binding set");
    binding_set.validate()?;
    if binding_set.scope != commit.scope
        || binding_set.source_control_key != source_control_key
        || binding_set.catalogue_revision > catalogue_revision
        || binding_set.schema_revision > schema.revision
        || binding_set.source_control_sha256 != digest::sha256_hex(&source_control)
    {
        return Err(Error::IndexConstraint(format!(
            "scope {} index commit-binding set does not match its accepted catalogue and schema",
            commit.scope
        )));
    }
    let definitions = bindings
        .iter()
        .map(|binding| binding.definition.clone())
        .collect::<Vec<_>>();
    let binding_count = u64::try_from(bindings.len())
        .map_err(|_| Error::IndexConstraint("index binding count exceeds u64".into()))?;
    if binding_set.definition_count != binding_count
        || binding_set.definitions_sha256 != definitions_sha256(&definitions)?
    {
        return Err(Error::IndexConstraint(format!(
            "scope {} index commit-binding projection is incomplete",
            commit.scope
        )));
    }
    for binding in &mut bindings {
        binding.validate()?;
        if binding.scope != commit.scope
            || binding.schema_revision != binding_set.schema_revision
            || binding.catalogue_revision != binding_set.catalogue_revision
            || binding.source_control_key != binding_set.source_control_key
            || binding.source_control_sha256 != binding_set.source_control_sha256
        {
            return Err(Error::IndexConstraint(format!(
                "index {} escaped its accepted commit-binding set",
                binding.definition.id
            )));
        }
        if binding.catalogue_revision > catalogue_revision {
            return Err(Error::IndexConstraint(format!(
                "index {} commit binding names a future catalogue revision",
                binding.definition.id
            )));
        }
        validate_binding_schema(&binding.definition, schema)?;
        if binding.schema_revision != schema.revision {
            binding.schema_revision = schema.revision;
            binding.catalogue_revision = catalogue_revision;
            plan.put_json(
                keyspaces::RUNTIME_INDEX_BINDINGS,
                &keyspaces::runtime_index_binding_key(&commit.scope, &binding.definition.id),
                binding,
            )?;
            plan.put_json(
                keyspaces::RUNTIME_INDEX_BINDING_VERSIONS,
                &keyspaces::runtime_index_binding_version_key(
                    &commit.scope,
                    &binding.definition.id,
                    catalogue_revision,
                    schema.revision,
                ),
                binding,
            )?;
        }
    }
    if binding_set.schema_revision != schema.revision {
        binding_set.schema_revision = schema.revision;
        binding_set.catalogue_revision = catalogue_revision;
        plan.put_json(
            keyspaces::RUNTIME_INDEX_BINDING_SETS,
            &keyspaces::runtime_index_binding_set_key(&commit.scope),
            binding_set,
        )?;
    }

    let transitions = record_transitions(reader, commit, starting_cursor)?;
    let removals = commit
        .mutations
        .iter()
        .filter_map(|mutation| match mutation {
            RuntimeMutation::Retire { retirement } if retirement.model.is_record_like() => {
                Some((&retirement.reference, retirement.effective_at))
            }
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    for binding in bindings {
        let relevant = transitions
            .iter()
            .filter(|(reference, _)| reference.kind == binding.definition.record_kind)
            .collect::<Vec<_>>();
        if relevant.is_empty() {
            continue;
        }
        if matches!(
            binding.definition.kind,
            IndexCommitBindingKind::Scalar { unique: true }
        ) {
            validate_unique_transition(reader, &commit.scope, &binding, &relevant)?;
        }
        let mut lexical = binding.definition.lexical.as_ref().map(|config| {
            super::super::lexical_index::LexicalChanges::new(
                &commit.scope,
                &binding.definition.id,
                config,
                binding.definition.fields[0].as_str(),
            )
        });
        for (reference, transition) in &relevant {
            if let Some(changes) = lexical.as_mut() {
                let before_text = transition
                    .before
                    .as_ref()
                    .map(|record| source_row(record, &binding.definition.fields));
                let after_text = transition
                    .after
                    .as_ref()
                    .map(|record| source_row(record, &binding.definition.fields));
                if before_text != after_text {
                    if let Some(before) = &transition.before {
                        changes.remove(before, removed_at(&removals, reference, transition))?;
                    }
                    if let Some(after) = &transition.after {
                        changes.add(after)?;
                    }
                }
            }
        }
        let mut put = |space, key: Vec<u8>, value: Option<Vec<u8>>| match value {
            Some(value) => plan.put_bytes(space, &key, value),
            None => plan.delete(space, &key),
        };
        if let Some(changes) = lexical {
            changes.write(reader, &mut put)?;
        } else {
            // Scalar entries describe current versions too, so the same
            // watermark bounds the valid times they answer for.
            let watermark = relevant
                .iter()
                .filter(|(_, transition)| {
                    transition
                        .before
                        .as_ref()
                        .map(|record| source_row(record, &binding.definition.fields))
                        != transition
                            .after
                            .as_ref()
                            .map(|record| source_row(record, &binding.definition.fields))
                })
                .flat_map(|(reference, transition)| {
                    [
                        removed_at(&removals, reference, transition),
                        transition
                            .after
                            .as_ref()
                            .map(super::super::lexical_index::addition_watermark),
                    ]
                })
                .flatten()
                .max();
            if let Some(at) = watermark {
                super::super::lexical_index::raise_watermark(
                    reader,
                    &mut put,
                    &commit.scope,
                    &binding.definition.id,
                    at,
                )?;
            }
        }
        for (reference, transition) in relevant {
            let before = transition
                .before
                .as_ref()
                .map(|record| source_row(record, &binding.definition.fields));
            let after = transition
                .after
                .as_ref()
                .map(|record| source_row(record, &binding.definition.fields));
            if before == after {
                continue;
            }
            if let IndexCommitBindingKind::Scalar { unique } = binding.definition.kind {
                let space = if unique {
                    keyspaces::RUNTIME_UNIQUE_ENTRIES
                } else {
                    keyspaces::RUNTIME_SCALAR_ENTRIES
                };
                if let Some(row) = &before {
                    if !unique || !row.values.contains(&RuntimeValue::Null) {
                        plan.delete(
                            space,
                            &keyspaces::runtime_index_entry_key(
                                space,
                                &commit.scope,
                                &binding.definition.id,
                                &row.values,
                                reference,
                                row.valid_from,
                            )?,
                        )?;
                    }
                }
                if let Some(row) = &after {
                    if !unique || !row.values.contains(&RuntimeValue::Null) {
                        let entry = ScalarIndexEntry {
                            index_id: binding.definition.id.clone(),
                            record: (*reference).clone(),
                            values: row.values.clone(),
                            valid_from: row.valid_from,
                            valid_to: row.valid_to,
                            source_cursor: transition.source_cursor,
                        };
                        plan.put_json(
                            space,
                            &keyspaces::runtime_index_entry_key(
                                space,
                                &commit.scope,
                                &binding.definition.id,
                                &row.values,
                                reference,
                                row.valid_from,
                            )?,
                            &entry,
                        )?;
                    }
                }
            }
            let delta = IndexSourceDelta {
                contract_version: INDEX_SOURCE_DELTA_CONTRACT_VERSION,
                scope: commit.scope.clone(),
                index_id: binding.definition.id.clone(),
                binding_kind: binding.definition.kind,
                configuration_sha256: binding.definition.configuration_sha256.clone(),
                source_control_sha256: binding.source_control_sha256.clone(),
                schema_revision: schema.revision,
                catalogue_revision,
                source_cursor: transition.source_cursor,
                commit_id: commit.digest(),
                commit_ordinal: transition.commit_ordinal,
                record: (*reference).clone(),
                before,
                after,
            };
            delta.validate()?;
            plan.put_json(
                keyspaces::RUNTIME_INDEX_SOURCE_DELTAS,
                &keyspaces::runtime_index_source_delta_key(
                    &commit.scope,
                    &binding.definition.id,
                    transition.source_cursor,
                    transition.commit_ordinal,
                ),
                &delta,
            )?;
        }
    }
    Ok(())
}

pub(crate) fn index_source_deltas(
    reader: &(impl super::super::runtime_state::AccessRead + ?Sized),
    scope: &ScopeId,
    index: &ProjectionId,
    after: u64,
    limit: usize,
) -> Result<Vec<IndexSourceDelta>> {
    if limit == 0 {
        return Err(Error::Substrate(
            "index source-delta page limit must be greater than zero".into(),
        ));
    }
    let start = keyspaces::runtime_index_source_delta_start(
        scope,
        index,
        after.checked_add(1).ok_or(Error::SequenceOverflow)?,
    );
    let prefix = keyspaces::runtime_index_source_delta_prefix(scope, index);
    let mut deltas = scan_space_bounded_from(
        reader,
        keyspaces::RUNTIME_INDEX_SOURCE_DELTAS,
        &prefix,
        &start,
        limit,
    )?
    .into_iter()
    .map(|(_, bytes)| serde_json::from_slice::<IndexSourceDelta>(&bytes).map_err(Error::from))
    .collect::<Result<Vec<_>>>()?;
    for delta in &deltas {
        delta.validate()?;
        if delta.scope != *scope || delta.index_id != *index {
            return Err(Error::IndexConstraint(
                "index source-delta key escaped its requested identity".into(),
            ));
        }
    }
    deltas.sort_by_key(|delta| (delta.source_cursor, delta.commit_ordinal));
    Ok(deltas)
}

pub(super) fn raw_bindings_for_scope(
    reader: &(impl super::super::runtime_state::AccessRead + ?Sized),
    scope: &ScopeId,
) -> Result<Vec<IndexCommitBinding>> {
    let prefix = keyspaces::runtime_index_binding_scope_prefix(scope);
    let encoded = scan_space_bounded(
        reader,
        keyspaces::RUNTIME_INDEX_BINDINGS,
        &prefix,
        MAX_INDEX_COMMIT_BINDINGS + 1,
    )?;
    if encoded.len() > MAX_INDEX_COMMIT_BINDINGS {
        return Err(Error::IndexConstraint(format!(
            "scope {scope} exceeds {MAX_INDEX_COMMIT_BINDINGS} commit-bound indexes"
        )));
    }
    encoded
        .into_iter()
        .map(|(_, bytes)| {
            let binding: IndexCommitBinding = serde_json::from_slice(&bytes)?;
            binding.validate()?;
            if binding.scope != *scope {
                return Err(Error::IndexConstraint(
                    "index commit binding escaped its scope key".into(),
                ));
            }
            Ok(binding)
        })
        .collect()
}
