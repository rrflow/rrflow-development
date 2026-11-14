//! The plan's shape and exact join rows.

use super::*;

pub(super) struct PlanShape {
    pub(super) source: Source,
    pub(super) join: Option<Join>,
    pub(super) valid_at: u64,
    pub(super) known_at_cursor: u64,
    pub(super) filters: Vec<BoundFilter>,
    pub(super) projection: Projection,
    pub(super) limit: Option<usize>,
    pub(super) schema_revision: u64,
    pub(super) source_cursor: u64,
}

impl PlanShape {
    pub(super) fn read(plan: &PhysicalPlan) -> Result<Self> {
        let mut source = None;
        let mut temporal = None;
        let mut join = None;
        let mut filters = Vec::new();
        let mut projection = None;
        let mut limit = None;
        for operator in &plan.logical.operators {
            match operator {
                LogicalOperator::Scan { source: value } => {
                    if source.replace(value.clone()).is_some() {
                        return Err(Error::Integrity(
                            "logical plan contains more than one scan".into(),
                        ));
                    }
                }
                LogicalOperator::Join {
                    source,
                    left_field,
                    right_field,
                } => {
                    if join
                        .replace(Join {
                            source: source.clone(),
                            left_field: left_field.clone(),
                            right_field: right_field.clone(),
                        })
                        .is_some()
                    {
                        return Err(Error::Integrity(
                            "logical plan contains more than one join".into(),
                        ));
                    }
                }
                LogicalOperator::Filter { predicates } => filters.extend(predicates.clone()),
                LogicalOperator::Project { projection: value } => {
                    if projection.replace(value.clone()).is_some() {
                        return Err(Error::Integrity(
                            "logical plan contains more than one projection".into(),
                        ));
                    }
                }
                LogicalOperator::Limit { rows } => {
                    if limit.replace(*rows).is_some() {
                        return Err(Error::Integrity(
                            "logical plan contains more than one limit".into(),
                        ));
                    }
                }
                LogicalOperator::Temporal {
                    valid_at,
                    known_at_cursor,
                } => {
                    if temporal.replace((*valid_at, *known_at_cursor)).is_some() {
                        return Err(Error::Integrity(
                            "logical plan contains more than one temporal selector".into(),
                        ));
                    }
                }
            }
        }
        let (valid_at, known_at_cursor) = temporal
            .ok_or_else(|| Error::Integrity("logical plan has no temporal selector".into()))?;
        Ok(Self {
            source: source.ok_or_else(|| Error::Integrity("logical plan has no scan".into()))?,
            join,
            valid_at,
            known_at_cursor,
            filters,
            projection: projection
                .ok_or_else(|| Error::Integrity("logical plan has no projection".into()))?,
            limit,
            schema_revision: plan.logical.schema_revision,
            source_cursor: plan.logical.source_cursor,
        })
    }
}

pub(super) fn exact_join_rows(
    left: &[QueryRow],
    right: &[QueryRow],
    join: &Join,
    max_rows: usize,
) -> Result<Vec<QueryRow>> {
    let mut right_by_key = BTreeMap::<String, Vec<&QueryRow>>::new();
    for row in right {
        let Some(value) = row.values.get(&join.right_field) else {
            return Err(Error::Integrity(format!(
                "right join row {} lacks bound field {:?}",
                row.identity, join.right_field
            )));
        };
        if !matches!(value, RuntimeValue::Null) {
            right_by_key
                .entry(serde_json::to_string(value)?)
                .or_default()
                .push(row);
        }
    }

    let mut joined = Vec::new();
    for left_row in left {
        let Some(value) = left_row.values.get(&join.left_field) else {
            return Err(Error::Integrity(format!(
                "left join row {} lacks bound field {:?}",
                left_row.identity, join.left_field
            )));
        };
        if matches!(value, RuntimeValue::Null) {
            continue;
        }
        let key = serde_json::to_string(value)?;
        for right_row in right_by_key.get(&key).into_iter().flatten() {
            if joined.len() == max_rows {
                return Err(Error::Budget(format!(
                    "join produces more than the {max_rows}-row execution bound"
                )));
            }
            let values = left_row
                .values
                .iter()
                .map(|(field, value)| (format!("left.{field}"), value.clone()))
                .chain(
                    right_row
                        .values
                        .iter()
                        .map(|(field, value)| (format!("right.{field}"), value.clone())),
                )
                .collect();
            joined.push(QueryRow {
                identity: format!(
                    "join:{}:{}{}",
                    left_row.identity.len(),
                    left_row.identity,
                    right_row.identity
                ),
                values,
            });
        }
    }
    Ok(joined)
}
