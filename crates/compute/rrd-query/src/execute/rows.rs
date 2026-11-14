//! Rows for each source: records, graph values, geo, series and claims.

use super::*;

pub(super) fn rows_for_source(
    changes: &[RuntimeChange],
    scope: &rrd_core::ScopeId,
    source: &Source,
    valid_at: u64,
    known_at_cursor: u64,
) -> Vec<QueryRow> {
    match source {
        Source::Record { kind } => active_graph_values(changes, scope, valid_at, known_at_cursor)
            .0
            .into_values()
            .filter(|record| &record.reference.kind == kind)
            .map(|record| {
                let identity =
                    result_identity::typed(ResultIdentityFamily::Record, &record.reference);
                let mut values = record.properties;
                values.insert("id".into(), string(record.reference.id.to_string()));
                values.insert("kind".into(), string(record.reference.kind.to_string()));
                values.insert(
                    "valid_from".into(),
                    RuntimeValue::Unsigned(record.valid_from),
                );
                values.insert("valid_to".into(), optional_u64(record.valid_to));
                QueryRow { identity, values }
            })
            .collect(),
        Source::Relation { kind } => active_graph_values(changes, scope, valid_at, known_at_cursor)
            .1
            .into_values()
            .filter(|relation| &relation.reference.kind == kind)
            .map(|relation| {
                let identity =
                    result_identity::typed(ResultIdentityFamily::Relation, &relation.reference);
                let mut values = relation.properties;
                values.insert("id".into(), string(relation.reference.id.to_string()));
                values.insert("kind".into(), string(relation.reference.kind.to_string()));
                values.insert("from_kind".into(), string(relation.from.kind.to_string()));
                values.insert("from_id".into(), string(relation.from.id.to_string()));
                values.insert("to_kind".into(), string(relation.to.kind.to_string()));
                values.insert("to_id".into(), string(relation.to.id.to_string()));
                values.insert(
                    "valid_from".into(),
                    RuntimeValue::Unsigned(relation.valid_from),
                );
                values.insert("valid_to".into(), optional_u64(relation.valid_to));
                QueryRow { identity, values }
            })
            .collect(),
        Source::Event { kind } => {
            let mut active = BTreeMap::new();
            for change in changes
                .iter()
                .filter(|change| change.cursor <= known_at_cursor && &change.scope == scope)
            {
                match &change.mutation {
                    RuntimeMutation::Event { event }
                        if &event.kind == kind && change.at <= valid_at =>
                    {
                        let reference =
                            rrd_core::RuntimeRef::event(event.kind.clone(), change.cursor);
                        active.insert(reference, (change, event));
                    }
                    RuntimeMutation::Retire { retirement }
                        if retirement.model.is_event_like()
                            && &retirement.reference.kind == kind
                            && retirement.effective_at <= valid_at =>
                    {
                        active.remove(&retirement.reference);
                    }
                    _ => {}
                }
            }
            active
                .into_values()
                .map(|(change, event)| {
                    let mut values = event.properties.clone();
                    values.insert("cursor".into(), RuntimeValue::Unsigned(change.cursor));
                    values.insert("kind".into(), string(event.kind.to_string()));
                    values.insert("at".into(), RuntimeValue::Unsigned(change.at));
                    values.insert("actor".into(), string(change.actor.clone()));
                    values.insert(
                        "subject_kind".into(),
                        event
                            .subject
                            .as_ref()
                            .map_or(RuntimeValue::Null, |subject| {
                                string(subject.kind.to_string())
                            }),
                    );
                    values.insert(
                        "subject_id".into(),
                        event
                            .subject
                            .as_ref()
                            .map_or(RuntimeValue::Null, |subject| string(subject.id.to_string())),
                    );
                    QueryRow {
                        identity: result_identity::encode(
                            ResultIdentityFamily::Event,
                            &[event.kind.as_str(), &change.cursor.to_string()],
                        ),
                        values,
                    }
                })
                .collect()
        }
        Source::Series { kind } => {
            let mut active = BTreeMap::new();
            for change in changes
                .iter()
                .filter(|change| change.cursor <= known_at_cursor && &change.scope == scope)
            {
                match &change.mutation {
                    RuntimeMutation::SeriesSample { sample }
                        if &sample.series.kind == kind && sample.observed_at <= valid_at =>
                    {
                        active.insert(sample.reference.clone(), sample);
                    }
                    RuntimeMutation::Retire { retirement }
                        if retirement.model == rrd_core::RuntimeLogicalModel::TimeSeries
                            && retirement.effective_at <= valid_at =>
                    {
                        active.remove(&retirement.reference);
                    }
                    _ => {}
                }
            }
            active
                .into_values()
                .map(|sample| {
                    let mut values = sample.properties.clone();
                    values.insert("id".into(), string(sample.reference.id.to_string()));
                    values.insert("kind".into(), string(sample.reference.kind.to_string()));
                    values.insert("series_kind".into(), string(sample.series.kind.to_string()));
                    values.insert("series_id".into(), string(sample.series.id.to_string()));
                    values.insert(
                        "observed_at".into(),
                        RuntimeValue::Unsigned(sample.observed_at),
                    );
                    values.insert("value".into(), series_value(&sample.value));
                    QueryRow {
                        identity: result_identity::typed(
                            ResultIdentityFamily::Series,
                            &sample.reference,
                        ),
                        values,
                    }
                })
                .collect()
        }
        Source::Geo { kind } => geo_rows(changes, scope, kind, valid_at, known_at_cursor),
        Source::Traversal {
            relation,
            start,
            direction,
            max_depth,
        } => crate::physical::graph::traversal_rows(
            changes,
            scope,
            relation,
            start,
            *direction,
            *max_depth,
            valid_at,
            known_at_cursor,
        ),
        Source::Claim { predicate } => claim_rows(
            changes,
            scope,
            predicate.as_ref(),
            valid_at,
            known_at_cursor,
        ),
    }
}

pub(crate) fn active_graph_values<'a>(
    changes: impl IntoIterator<Item = &'a RuntimeChange>,
    scope: &rrd_core::ScopeId,
    valid_at: u64,
    known_at_cursor: u64,
) -> (
    BTreeMap<rrd_core::RuntimeRef, RuntimeRecord>,
    BTreeMap<rrd_core::RuntimeRef, RuntimeRelation>,
) {
    let mut state =
        crate::physical::graph::state::GraphState::new(scope, valid_at, known_at_cursor);
    for change in changes {
        state.apply(change);
    }
    state.into_values()
}

pub(super) fn geo_rows(
    changes: &[RuntimeChange],
    scope: &rrd_core::ScopeId,
    kind: &rrd_core::RuntimeType,
    valid_at: u64,
    known_at_cursor: u64,
) -> Vec<QueryRow> {
    let mut active = BTreeMap::<rrd_core::RuntimeRef, (u64, RuntimeGeo)>::new();
    for change in changes
        .iter()
        .filter(|change| change.cursor <= known_at_cursor && &change.scope == scope)
    {
        match &change.mutation {
            RuntimeMutation::Geo { geo }
                if &geo.reference.kind == kind
                    && geo.valid_from <= valid_at
                    && geo.valid_to.is_none_or(|end| valid_at < end) =>
            {
                let replace = active.get(&geo.reference).is_none_or(|(cursor, current)| {
                    geo.valid_from > current.valid_from
                        || (geo.valid_from == current.valid_from && change.cursor > *cursor)
                });
                if replace {
                    active.insert(geo.reference.clone(), (change.cursor, geo.clone()));
                }
            }
            RuntimeMutation::Retire { retirement }
                if retirement.model == rrd_core::RuntimeLogicalModel::Geo
                    && &retirement.reference.kind == kind
                    && retirement.effective_at <= valid_at =>
            {
                active.remove(&retirement.reference);
            }
            _ => {}
        }
    }
    active
        .into_values()
        .map(|(_, geo)| {
            let mut values = geo.properties;
            values.insert("id".into(), string(geo.reference.id.to_string()));
            values.insert("kind".into(), string(geo.reference.kind.to_string()));
            values.insert("subject_kind".into(), string(geo.subject.kind.to_string()));
            values.insert("subject_id".into(), string(geo.subject.id.to_string()));
            values.insert("field".into(), string(geo.field));
            values.insert("valid_from".into(), RuntimeValue::Unsigned(geo.valid_from));
            values.insert("valid_to".into(), optional_u64(geo.valid_to));
            insert_geo_value(&mut values, &geo.value);
            QueryRow {
                identity: result_identity::typed(ResultIdentityFamily::Geo, &geo.reference),
                values,
            }
        })
        .collect()
}

pub(super) fn series_value(value: &SeriesValue) -> RuntimeValue {
    match value {
        SeriesValue::Integer(value) => RuntimeValue::Integer(*value),
        SeriesValue::Unsigned(value) => RuntimeValue::Unsigned(*value),
        SeriesValue::Decimal(value) => RuntimeValue::Decimal(value.clone()),
        SeriesValue::Bool(value) => RuntimeValue::Bool(*value),
        SeriesValue::String(value) => RuntimeValue::String(value.clone()),
    }
}

pub(super) fn insert_geo_value(values: &mut BTreeMap<String, RuntimeValue>, geo: &GeoValue) {
    for field in [
        "longitude",
        "latitude",
        "southwest_longitude",
        "southwest_latitude",
        "northeast_longitude",
        "northeast_latitude",
    ] {
        values.insert(field.into(), RuntimeValue::Null);
    }
    match geo {
        GeoValue::Point { point } => {
            values.insert("geometry_kind".into(), string("point".into()));
            values.insert(
                "longitude".into(),
                RuntimeValue::Decimal(point.longitude.to_string()),
            );
            values.insert(
                "latitude".into(),
                RuntimeValue::Decimal(point.latitude.to_string()),
            );
        }
        GeoValue::BoundingBox {
            southwest,
            northeast,
        } => {
            values.insert("geometry_kind".into(), string("bounding_box".into()));
            values.insert(
                "southwest_longitude".into(),
                RuntimeValue::Decimal(southwest.longitude.to_string()),
            );
            values.insert(
                "southwest_latitude".into(),
                RuntimeValue::Decimal(southwest.latitude.to_string()),
            );
            values.insert(
                "northeast_longitude".into(),
                RuntimeValue::Decimal(northeast.longitude.to_string()),
            );
            values.insert(
                "northeast_latitude".into(),
                RuntimeValue::Decimal(northeast.latitude.to_string()),
            );
        }
    }
}

pub(super) fn claim_rows(
    changes: &[RuntimeChange],
    scope: &rrd_core::ScopeId,
    predicate: Option<&rrd_core::Predicate>,
    valid_at: u64,
    known_at_cursor: u64,
) -> Vec<QueryRow> {
    let mut groups = BTreeMap::<(String, String), Vec<(u64, Claim)>>::new();
    for change in changes
        .iter()
        .filter(|change| change.cursor <= known_at_cursor && &change.scope == scope)
    {
        let RuntimeMutation::Claim { claim } = &change.mutation else {
            continue;
        };
        if predicate.is_some_and(|expected| &claim.predicate != expected) {
            continue;
        }
        groups
            .entry((claim.subject.to_string(), claim.predicate.to_string()))
            .or_default()
            .push((change.cursor, claim.clone()));
    }
    groups
        .into_iter()
        .filter_map(|((subject, predicate), mut versions)| {
            versions.sort_by_key(|version| std::cmp::Reverse(version.0));
            let candidates = versions
                .into_iter()
                .map(|(_, claim)| claim)
                .collect::<Vec<_>>();
            let claim = resolve_as_of(&candidates, valid_at)?;
            let mut values = BTreeMap::new();
            values.insert("subject".into(), string(subject.clone()));
            values.insert("predicate".into(), string(predicate.clone()));
            values.insert("object".into(), string(claim.object.clone()));
            values.insert(
                "valid_from".into(),
                RuntimeValue::Unsigned(claim.valid_from),
            );
            values.insert("valid_to".into(), optional_u64(claim.valid_to));
            values.insert("tx_time".into(), RuntimeValue::Unsigned(claim.tx_time));
            values.insert("actor".into(), string(claim.producer.actor.clone()));
            Some(QueryRow {
                identity: result_identity::claim(&claim.subject, &claim.predicate),
                values,
            })
        })
        .collect()
}
