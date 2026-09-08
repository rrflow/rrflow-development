//! Schema-captured GraphQL ingress for the canonical rrflowQL query model.
//!
//! This module is deliberately an adapter, not an executor. It parses one
//! bounded GraphQL query operation, validates the lowerable surface against a
//! [`Catalog`] captured at one [`ReadStamp`], and produces the same [`Query`]
//! and [`BoundQuery`] used by rrflowQL. It performs no storage I/O and owns no
//! resolver, authorization decision, read stamp, or response lifecycle.

use crate::plan::fields_for_source;
use crate::{
    bind, BoundQuery, Catalog, ComparisonOperator, CursorExpr, Error, Filter, Join, Parameters,
    Projection, Query, Result, Source, TemporalSelector, TimeExpr, TraversalDirection, ValueExpr,
    QUERY_CONTRACT_VERSION,
};
use async_graphql_parser::types::{
    BaseType, DocumentOperations, OperationDefinition, OperationType, Selection, Type,
};
use async_graphql_parser::{parse_query, Positioned};
use async_graphql_value::{ConstValue, Value};
use rrd_core::{
    digest, Predicate, ReadStamp, RuntimeId, RuntimeRef, RuntimeType, RuntimeValue,
    RuntimeValueType,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const GRAPHQL_ADAPTER_CONTRACT_VERSION: u16 = 1;
pub const MAX_GRAPHQL_DOCUMENT_BYTES: usize = 64 * 1024;
pub const MAX_GRAPHQL_VARIABLES: usize = 256;
const MAX_GRAPHQL_ARGUMENTS: usize = 32;
const MAX_GRAPHQL_PROJECTION_FIELDS: usize = 512;

/// One transport-neutral GraphQL request. HTTP carriage is intentionally a
/// later adapter gate; values already use RRFlow's canonical scalar model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphqlRequest {
    pub contract_version: u16,
    pub document: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub variables: Parameters,
}

impl GraphqlRequest {
    fn validate_envelope(&self) -> Result<()> {
        if self.contract_version != GRAPHQL_ADAPTER_CONTRACT_VERSION {
            return graphql_error(format!(
                "unsupported GraphQL adapter contract version {}",
                self.contract_version
            ));
        }
        if self.document.is_empty() || self.document.len() > MAX_GRAPHQL_DOCUMENT_BYTES {
            return graphql_error(format!(
                "GraphQL document length must be in 1..={MAX_GRAPHQL_DOCUMENT_BYTES} bytes"
            ));
        }
        if self.variables.len() > MAX_GRAPHQL_VARIABLES {
            return graphql_error(format!(
                "GraphQL request exceeds {MAX_GRAPHQL_VARIABLES} variables"
            ));
        }
        if let Some(name) = &self.operation_name {
            validate_graphql_name(name, "operation name")?;
        }
        for name in self.variables.keys() {
            validate_graphql_name(name, "variable name")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphqlSourceFamily {
    Claim,
    Event,
    Geo,
    Record,
    Relation,
    Series,
    Traversal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphqlFieldDefinition {
    pub name: String,
    pub accepted_types: Vec<RuntimeValueType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphqlSourceDefinition {
    pub family: GraphqlSourceFamily,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub fields: Vec<GraphqlFieldDefinition>,
}

/// A deterministic GraphQL-visible schema projection at one historical
/// schema revision inside a captured read stamp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphqlSchema {
    pub contract_version: u16,
    pub read: ReadStamp,
    pub known_at_cursor: u64,
    pub schema_revision: u64,
    pub sources: Vec<GraphqlSourceDefinition>,
    pub digest: String,
}

impl GraphqlSchema {
    pub fn validate(&self) -> Result<()> {
        if self.contract_version != GRAPHQL_ADAPTER_CONTRACT_VERSION {
            return graphql_error("GraphQL schema contract version is unsupported");
        }
        self.read
            .validate()
            .map_err(|error| Error::Graphql(error.to_string()))?;
        if self.known_at_cursor > self.read.commit_cursor || self.schema_revision == 0 {
            return graphql_error("GraphQL schema carries an invalid read coordinate");
        }
        if self.sources.is_empty()
            || self
                .sources
                .windows(2)
                .any(|pair| source_sort_key(&pair[0]) >= source_sort_key(&pair[1]))
        {
            return graphql_error("GraphQL schema sources must be non-empty, sorted, and unique");
        }
        for source in &self.sources {
            if source.fields.is_empty()
                || source
                    .fields
                    .windows(2)
                    .any(|pair| pair[0].name >= pair[1].name)
            {
                return graphql_error(
                    "GraphQL source fields must be non-empty, sorted, and unique",
                );
            }
            if let Some(kind) = &source.kind {
                if kind.is_empty() {
                    return graphql_error("GraphQL source kind must not be empty");
                }
            }
            for field in &source.fields {
                validate_graphql_name(&field.name, "schema field")?;
                if field.accepted_types.is_empty() {
                    return graphql_error("GraphQL schema field requires an accepted type");
                }
            }
        }
        if self.digest != schema_digest(self)? {
            return graphql_error("GraphQL schema digest does not match its captured contents");
        }
        Ok(())
    }

    fn admits(&self, source: &Source) -> bool {
        let (family, kind) = source_coordinate(source);
        self.sources.iter().any(|candidate| {
            candidate.family == family
                && (candidate.kind.is_none() || candidate.kind.as_deref() == kind)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoweredGraphqlQuery {
    pub schema_digest: String,
    pub response_key: String,
    pub query: Query,
    pub parameters: Parameters,
}

/// Derive the exact GraphQL-visible source/field catalogue for a historical
/// schema revision within `catalog`'s already captured read stamp.
pub fn derive_graphql_schema(catalog: &Catalog, known_at_cursor: u64) -> Result<GraphqlSchema> {
    if known_at_cursor > catalog.read.commit_cursor {
        return graphql_error(format!(
            "known cursor {known_at_cursor} exceeds captured head {}",
            catalog.read.commit_cursor
        ));
    }
    let registry = catalog.schema_at(known_at_cursor).ok_or_else(|| {
        Error::Graphql(format!(
            "no schema was visible at known cursor {known_at_cursor}"
        ))
    })?;
    let mut definitions = Vec::new();
    for kind in registry.records.keys() {
        definitions.push(source_definition(
            GraphqlSourceFamily::Record,
            Some(kind.as_str()),
            Source::Record { kind: kind.clone() },
            registry,
        )?);
    }
    for kind in registry.relations.keys() {
        definitions.push(source_definition(
            GraphqlSourceFamily::Relation,
            Some(kind.as_str()),
            Source::Relation { kind: kind.clone() },
            registry,
        )?);
    }
    for kind in registry.events.keys() {
        definitions.push(source_definition(
            GraphqlSourceFamily::Event,
            Some(kind.as_str()),
            Source::Event { kind: kind.clone() },
            registry,
        )?);
    }

    definitions.push(source_definition(
        GraphqlSourceFamily::Series,
        None,
        Source::Series {
            kind: RuntimeType::new("_graphql_schema_series")
                .map_err(|error| Error::Graphql(error.to_string()))?,
        },
        registry,
    )?);
    definitions.push(source_definition(
        GraphqlSourceFamily::Geo,
        None,
        Source::Geo {
            kind: RuntimeType::new("_graphql_schema_geo")
                .map_err(|error| Error::Graphql(error.to_string()))?,
        },
        registry,
    )?);
    definitions.push(source_definition(
        GraphqlSourceFamily::Claim,
        None,
        Source::Claim { predicate: None },
        registry,
    )?);

    for (relation, definition) in &registry.relations {
        let Some(start_kind) = definition.from.iter().chain(&definition.to).next() else {
            continue;
        };
        definitions.push(source_definition(
            GraphqlSourceFamily::Traversal,
            Some(relation.as_str()),
            Source::Traversal {
                relation: relation.clone(),
                start: RuntimeRef {
                    kind: start_kind.clone(),
                    id: RuntimeId::new("_graphql_schema_start")
                        .map_err(|error| Error::Graphql(error.to_string()))?,
                },
                direction: TraversalDirection::Both,
                max_depth: 1,
            },
            registry,
        )?);
    }

    definitions.sort_by(|left, right| source_sort_key(left).cmp(&source_sort_key(right)));
    if definitions
        .windows(2)
        .any(|pair| source_sort_key(&pair[0]) == source_sort_key(&pair[1]))
    {
        return graphql_error("GraphQL schema contains a duplicate source coordinate");
    }
    let mut schema = GraphqlSchema {
        contract_version: GRAPHQL_ADAPTER_CONTRACT_VERSION,
        read: catalog.read.clone(),
        known_at_cursor,
        schema_revision: registry.revision,
        sources: definitions,
        digest: String::new(),
    };
    schema.digest = schema_digest(&schema)?;
    schema.validate()?;
    Ok(schema)
}

/// Parse and lower GraphQL without touching storage. The returned parameters
/// include validated request values and operation defaults.
pub fn lower_graphql(request: &GraphqlRequest, catalog: &Catalog) -> Result<LoweredGraphqlQuery> {
    request.validate_envelope()?;
    let document = parse_query(&request.document)
        .map_err(|error| Error::Graphql(format!("GraphQL syntax error: {error}")))?;
    if !document.fragments.is_empty() {
        return graphql_error(
            "GraphQL fragments are outside the 1.0 lowering contract; inline the selection",
        );
    }
    let operation = select_operation(&document.operations, request.operation_name.as_deref())?;
    if operation.node.ty != OperationType::Query {
        return graphql_error("GraphQL mutation and subscription operations are not query ingress");
    }
    if !operation.node.directives.is_empty() {
        return graphql_error("GraphQL operation directives are not lowerable");
    }
    let parameters = validate_variables(&operation.node, &request.variables)?;
    let root = single_root_field(&operation.node)?;
    if !root.node.directives.is_empty() {
        return graphql_error("GraphQL root-field directives are not lowerable");
    }
    let response_key = root.node.response_key().node.to_string();
    let arguments = argument_map(&root.node.arguments)?;
    let source = lower_source(root.node.name.node.as_str(), &arguments, &parameters)?;
    let temporal = TemporalSelector {
        valid_at: lower_valid_at(required_argument(&arguments, "validAt")?, &parameters)?,
        known_at: lower_known_at(required_argument(&arguments, "knownAt")?, &parameters)?,
    };
    let known_at_cursor = resolve_known_cursor(&temporal.known_at, &parameters, &catalog.read)?;
    let schema = derive_graphql_schema(catalog, known_at_cursor)?;
    if !schema.admits(&source) {
        return graphql_error("GraphQL source is not present in the captured schema");
    }
    let join = arguments
        .get("join")
        .map(|value| lower_join(value, &parameters))
        .transpose()?;
    if matches!(source, Source::Traversal { .. }) && join.is_some() {
        return graphql_error("GraphQL traversal queries cannot also declare a join");
    }
    let projection = lower_projection(&root.node.selection_set.node.items, join.is_some())?;
    let filters = arguments
        .get("where")
        .map(|value| lower_filters(value, &parameters))
        .transpose()?
        .unwrap_or_default();
    let limit = arguments
        .get("limit")
        .map(|value| resolve_usize(value, &parameters, "limit"))
        .transpose()?;
    let (explain_contract, explain_analyze) = arguments
        .get("explain")
        .map(|value| lower_explain(value, &parameters))
        .transpose()?
        .unwrap_or((false, false));
    validate_allowed_arguments(root.node.name.node.as_str(), arguments.keys().copied())?;

    let query = Query {
        contract_version: QUERY_CONTRACT_VERSION,
        source,
        join,
        temporal,
        filters,
        projection,
        limit,
        explain_contract,
        explain_analyze,
    };
    query
        .validate()
        .map_err(|error| Error::Graphql(error.to_string()))?;
    Ok(LoweredGraphqlQuery {
        schema_digest: schema.digest,
        response_key,
        query,
        parameters,
    })
}

/// Lower GraphQL and invoke the existing rrflowQL binder. This is the only
/// B-05 entry point that produces a catalogue-bound query representation.
pub fn bind_graphql(request: &GraphqlRequest, catalog: &Catalog) -> Result<BoundQuery> {
    let lowered = lower_graphql(request, catalog)?;
    bind(&lowered.query, &lowered.parameters, catalog)
}

fn source_definition(
    family: GraphqlSourceFamily,
    kind: Option<&str>,
    source: Source,
    registry: &rrd_core::RuntimeSchemaRegistry,
) -> Result<GraphqlSourceDefinition> {
    let fields = fields_for_source(&source, registry)?
        .into_iter()
        .map(|(name, accepted_types)| {
            validate_graphql_name(&name, "schema field")?;
            Ok(GraphqlFieldDefinition {
                name,
                accepted_types,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(GraphqlSourceDefinition {
        family,
        kind: kind.map(str::to_owned),
        fields,
    })
}

fn schema_digest(schema: &GraphqlSchema) -> Result<String> {
    let bytes = serde_json::to_vec(&(
        schema.contract_version,
        &schema.read,
        schema.known_at_cursor,
        schema.schema_revision,
        &schema.sources,
    ))?;
    Ok(digest::sha256_hex(&bytes))
}

fn source_sort_key(source: &GraphqlSourceDefinition) -> (GraphqlSourceFamily, &str) {
    (source.family, source.kind.as_deref().unwrap_or(""))
}

fn source_coordinate(source: &Source) -> (GraphqlSourceFamily, Option<&str>) {
    match source {
        Source::Record { kind } => (GraphqlSourceFamily::Record, Some(kind.as_str())),
        Source::Relation { kind } => (GraphqlSourceFamily::Relation, Some(kind.as_str())),
        Source::Event { kind } => (GraphqlSourceFamily::Event, Some(kind.as_str())),
        Source::Series { .. } => (GraphqlSourceFamily::Series, None),
        Source::Geo { .. } => (GraphqlSourceFamily::Geo, None),
        Source::Traversal { relation, .. } => {
            (GraphqlSourceFamily::Traversal, Some(relation.as_str()))
        }
        Source::Claim { .. } => (GraphqlSourceFamily::Claim, None),
    }
}

fn select_operation<'a>(
    operations: &'a DocumentOperations,
    requested: Option<&str>,
) -> Result<&'a Positioned<OperationDefinition>> {
    match operations {
        DocumentOperations::Single(operation) => {
            if requested.is_some() {
                graphql_error("operationName cannot select an anonymous GraphQL operation")
            } else {
                Ok(operation)
            }
        }
        DocumentOperations::Multiple(operations) => match requested {
            Some(name) => operations
                .get(name)
                .ok_or_else(|| Error::Graphql(format!("GraphQL operation {name:?} was not found"))),
            None if operations.len() == 1 => Ok(operations
                .values()
                .next()
                .expect("one named operation was checked")),
            None => graphql_error(
                "operationName is required when a GraphQL document has multiple operations",
            ),
        },
    }
}

fn single_root_field(
    operation: &OperationDefinition,
) -> Result<&Positioned<async_graphql_parser::types::Field>> {
    if operation.selection_set.node.items.len() != 1 {
        return graphql_error("GraphQL query must select exactly one RRFlow root field");
    }
    match &operation.selection_set.node.items[0].node {
        Selection::Field(field) => Ok(field),
        Selection::FragmentSpread(_) | Selection::InlineFragment(_) => {
            graphql_error("GraphQL root selection must be a field")
        }
    }
}

fn argument_map(
    arguments: &[(Positioned<async_graphql_value::Name>, Positioned<Value>)],
) -> Result<BTreeMap<&str, &Value>> {
    if arguments.len() > MAX_GRAPHQL_ARGUMENTS {
        return graphql_error(format!(
            "GraphQL root exceeds {MAX_GRAPHQL_ARGUMENTS} arguments"
        ));
    }
    let mut out = BTreeMap::new();
    for (name, value) in arguments {
        if out.insert(name.node.as_str(), &value.node).is_some() {
            return graphql_error(format!("duplicate GraphQL argument {:?}", name.node));
        }
    }
    Ok(out)
}

fn required_argument<'a>(arguments: &BTreeMap<&str, &'a Value>, name: &str) -> Result<&'a Value> {
    arguments
        .get(name)
        .copied()
        .ok_or_else(|| Error::Graphql(format!("GraphQL argument {name:?} is required")))
}

fn lower_source(
    root: &str,
    arguments: &BTreeMap<&str, &Value>,
    parameters: &Parameters,
) -> Result<Source> {
    match root {
        "record" => Ok(Source::Record {
            kind: runtime_type(required_argument(arguments, "kind")?, parameters, "kind")?,
        }),
        "relation" => Ok(Source::Relation {
            kind: runtime_type(required_argument(arguments, "kind")?, parameters, "kind")?,
        }),
        "event" => Ok(Source::Event {
            kind: runtime_type(required_argument(arguments, "kind")?, parameters, "kind")?,
        }),
        "series" => Ok(Source::Series {
            kind: runtime_type(required_argument(arguments, "kind")?, parameters, "kind")?,
        }),
        "geo" => Ok(Source::Geo {
            kind: runtime_type(required_argument(arguments, "kind")?, parameters, "kind")?,
        }),
        "claim" => Ok(Source::Claim {
            predicate: arguments
                .get("predicate")
                .map(|value| resolve_string(value, parameters, "predicate"))
                .transpose()?
                .map(Predicate::new)
                .transpose()
                .map_err(|error| Error::Graphql(error.to_string()))?,
        }),
        "traverse" => {
            let relation = runtime_type(
                required_argument(arguments, "relation")?,
                parameters,
                "relation",
            )?;
            let start_kind = resolve_string(
                required_argument(arguments, "startKind")?,
                parameters,
                "startKind",
            )?;
            let start_id = resolve_string(
                required_argument(arguments, "startId")?,
                parameters,
                "startId",
            )?;
            let direction = match resolve_token(
                required_argument(arguments, "direction")?,
                parameters,
                "direction",
            )?
            .as_str()
            {
                "OUTGOING" => TraversalDirection::Outgoing,
                "INCOMING" => TraversalDirection::Incoming,
                "BOTH" => TraversalDirection::Both,
                value => {
                    return graphql_error(format!(
                        "GraphQL direction {value:?} must be OUTGOING, INCOMING, or BOTH"
                    ))
                }
            };
            let depth = resolve_u64(required_argument(arguments, "depth")?, parameters, "depth")?;
            Ok(Source::Traversal {
                relation,
                start: RuntimeRef::new(start_kind, start_id)
                    .map_err(|error| Error::Graphql(error.to_string()))?,
                direction,
                max_depth: u16::try_from(depth)
                    .map_err(|_| Error::Graphql("GraphQL depth exceeds u16".into()))?,
            })
        }
        _ => graphql_error(format!("unknown RRFlow GraphQL root field {root:?}")),
    }
}

fn lower_join(value: &Value, parameters: &Parameters) -> Result<Join> {
    let object = resolve_object(value, "join")?;
    let family = resolve_token(object_value(object, "family")?, parameters, "join.family")?;
    let kind = runtime_type(object_value(object, "kind")?, parameters, "join.kind")?;
    let source = match family.as_str() {
        "RECORD" => Source::Record { kind },
        "RELATION" => Source::Relation { kind },
        "EVENT" => Source::Event { kind },
        "SERIES" => Source::Series { kind },
        "GEO" => Source::Geo { kind },
        _ => {
            return graphql_error(format!(
                "GraphQL join family {family:?} is not a lowerable source"
            ))
        }
    };
    let allowed = BTreeSet::from(["family", "kind", "leftField", "rightField"]);
    reject_unknown_object_fields(object, &allowed, "join")?;
    Ok(Join {
        source,
        left_field: resolve_string(
            object_value(object, "leftField")?,
            parameters,
            "join.leftField",
        )?,
        right_field: resolve_string(
            object_value(object, "rightField")?,
            parameters,
            "join.rightField",
        )?,
    })
}

fn lower_valid_at(value: &Value, parameters: &Parameters) -> Result<TimeExpr> {
    match value {
        Value::Variable(name) => {
            require_unsigned_parameter(parameters, name.as_str(), "validAt")?;
            Ok(TimeExpr::Parameter(name.to_string()))
        }
        _ => Ok(TimeExpr::Literal(resolve_u64(
            value, parameters, "validAt",
        )?)),
    }
}

fn lower_known_at(value: &Value, parameters: &Parameters) -> Result<CursorExpr> {
    match value {
        Value::Variable(name) => {
            require_unsigned_parameter(parameters, name.as_str(), "knownAt")?;
            Ok(CursorExpr::Parameter(name.to_string()))
        }
        Value::Enum(name) if name == "HEAD" => Ok(CursorExpr::Head),
        _ => Ok(CursorExpr::Literal(resolve_u64(
            value, parameters, "knownAt",
        )?)),
    }
}

fn resolve_known_cursor(
    value: &CursorExpr,
    parameters: &Parameters,
    read: &ReadStamp,
) -> Result<u64> {
    let cursor = match value {
        CursorExpr::Head => read.commit_cursor,
        CursorExpr::Literal(value) => *value,
        CursorExpr::Parameter(name) => require_unsigned_parameter(parameters, name, "knownAt")?,
    };
    if cursor > read.commit_cursor {
        return graphql_error(format!(
            "known cursor {cursor} exceeds captured head {}",
            read.commit_cursor
        ));
    }
    Ok(cursor)
}

fn lower_filters(value: &Value, parameters: &Parameters) -> Result<Vec<Filter>> {
    let Value::List(values) = value else {
        return graphql_error("GraphQL where argument must be a list");
    };
    if values.len() > MAX_GRAPHQL_PROJECTION_FIELDS {
        return graphql_error("GraphQL where argument is too large");
    }
    values
        .iter()
        .map(|value| {
            let object = resolve_object(value, "where item")?;
            let allowed = BTreeSet::from(["field", "comparison", "value"]);
            reject_unknown_object_fields(object, &allowed, "where item")?;
            let field = resolve_string(object_value(object, "field")?, parameters, "where.field")?;
            let comparison = match resolve_token(
                object_value(object, "comparison")?,
                parameters,
                "where.comparison",
            )?
            .as_str()
            {
                "EQ" => ComparisonOperator::Equal,
                "NE" => ComparisonOperator::NotEqual,
                "LT" => ComparisonOperator::LessThan,
                "LTE" => ComparisonOperator::LessThanOrEqual,
                "GT" => ComparisonOperator::GreaterThan,
                "GTE" => ComparisonOperator::GreaterThanOrEqual,
                "MATCH" => ComparisonOperator::Match,
                token => {
                    return graphql_error(format!("GraphQL comparison {token:?} is not supported"))
                }
            };
            let value = object_value(object, "value")?;
            let value = match value {
                Value::Variable(name) => {
                    if !parameters.contains_key(name.as_str()) {
                        return graphql_error(format!(
                            "missing GraphQL variable ${}",
                            name.as_str()
                        ));
                    }
                    ValueExpr::Parameter(name.to_string())
                }
                _ => ValueExpr::Literal(resolve_scalar(value, parameters, "where.value")?),
            };
            Ok(Filter {
                field,
                comparison,
                value,
            })
        })
        .collect()
}

fn lower_projection(selections: &[Positioned<Selection>], joined: bool) -> Result<Projection> {
    if selections.is_empty() || selections.len() > MAX_GRAPHQL_PROJECTION_FIELDS {
        return graphql_error(format!(
            "GraphQL projection requires 1..={MAX_GRAPHQL_PROJECTION_FIELDS} fields"
        ));
    }
    let mut fields = Vec::new();
    for selection in selections {
        let Selection::Field(field) = &selection.node else {
            return graphql_error("GraphQL projection fragments are not lowerable");
        };
        if field.node.alias.is_some()
            || !field.node.arguments.is_empty()
            || !field.node.directives.is_empty()
        {
            return graphql_error(
                "GraphQL projection aliases, arguments, and directives are not lowerable",
            );
        }
        let name = field.node.name.node.as_str();
        if joined {
            if !matches!(name, "left" | "right") || field.node.selection_set.node.items.is_empty() {
                return graphql_error(
                    "joined GraphQL projections must use non-empty left and right selections",
                );
            }
            for nested in &field.node.selection_set.node.items {
                let Selection::Field(nested) = &nested.node else {
                    return graphql_error("joined GraphQL projection cannot contain fragments");
                };
                if nested.node.alias.is_some()
                    || !nested.node.arguments.is_empty()
                    || !nested.node.directives.is_empty()
                    || !nested.node.selection_set.node.items.is_empty()
                {
                    return graphql_error("joined GraphQL projection leaves must be plain fields");
                }
                fields.push(format!("{name}.{}", nested.node.name.node));
            }
        } else {
            if !field.node.selection_set.node.items.is_empty() {
                return graphql_error("GraphQL projection fields must be scalar leaves");
            }
            fields.push(name.to_owned());
        }
    }
    if fields.len() > MAX_GRAPHQL_PROJECTION_FIELDS {
        return graphql_error("GraphQL projection is too large");
    }
    let unique = fields.iter().collect::<BTreeSet<_>>();
    if unique.len() != fields.len() {
        return graphql_error("GraphQL projection fields must be unique");
    }
    Ok(Projection::Fields(fields))
}

fn lower_explain(value: &Value, parameters: &Parameters) -> Result<(bool, bool)> {
    match resolve_token(value, parameters, "explain")?.as_str() {
        "CONTRACT" => Ok((true, false)),
        "ANALYZE" => Ok((false, true)),
        token => graphql_error(format!(
            "GraphQL explain {token:?} must be CONTRACT or ANALYZE"
        )),
    }
}

fn validate_allowed_arguments<'a>(root: &str, names: impl Iterator<Item = &'a str>) -> Result<()> {
    let mut allowed = BTreeSet::from(["validAt", "knownAt", "where", "limit", "explain"]);
    match root {
        "record" | "relation" | "event" | "series" | "geo" => {
            allowed.extend(["kind", "join"]);
        }
        "claim" => {
            allowed.extend(["predicate", "join"]);
        }
        "traverse" => {
            allowed.extend(["relation", "startKind", "startId", "direction", "depth"]);
        }
        _ => return graphql_error(format!("unknown RRFlow GraphQL root field {root:?}")),
    }
    let unknown = names
        .filter(|name| !allowed.contains(name))
        .collect::<Vec<_>>();
    if unknown.is_empty() {
        Ok(())
    } else {
        graphql_error(format!("unknown GraphQL arguments: {}", unknown.join(", ")))
    }
}

fn validate_variables(
    operation: &OperationDefinition,
    supplied: &Parameters,
) -> Result<Parameters> {
    if operation.variable_definitions.len() > MAX_GRAPHQL_VARIABLES {
        return graphql_error(format!(
            "GraphQL operation exceeds {MAX_GRAPHQL_VARIABLES} variable definitions"
        ));
    }
    let mut definitions = BTreeMap::new();
    for definition in &operation.variable_definitions {
        let name = definition.node.name.node.to_string();
        validate_graphql_name(&name, "variable definition")?;
        validate_variable_type(&definition.node.var_type.node)?;
        if definitions.insert(name.clone(), definition).is_some() {
            return graphql_error(format!("duplicate GraphQL variable definition ${name}"));
        }
    }
    let mut referenced = BTreeSet::new();
    for selection in &operation.selection_set.node.items {
        collect_selection_variables(&selection.node, &mut referenced);
    }
    for name in &referenced {
        if !definitions.contains_key(name) {
            return graphql_error(format!("GraphQL variable ${name} is not defined"));
        }
    }
    for name in definitions.keys() {
        if !referenced.contains(name) {
            return graphql_error(format!("GraphQL variable ${name} is never used"));
        }
    }
    for name in supplied.keys() {
        if !definitions.contains_key(name) {
            return graphql_error(format!("GraphQL variable ${name} was not declared"));
        }
    }

    let mut effective = supplied.clone();
    for (name, definition) in definitions {
        if !effective.contains_key(&name) {
            if let Some(default) = &definition.node.default_value {
                effective.insert(
                    name.clone(),
                    const_scalar(&default.node, "variable default")?,
                );
            } else if !definition.node.var_type.node.nullable {
                return graphql_error(format!("required GraphQL variable ${name} is missing"));
            } else {
                effective.insert(name.clone(), RuntimeValue::Null);
            }
        }
        validate_variable_value(
            &name,
            &definition.node.var_type.node,
            effective
                .get(&name)
                .expect("effective variable was inserted or supplied"),
        )?;
    }
    Ok(effective)
}

fn validate_variable_type(value: &Type) -> Result<()> {
    let BaseType::Named(name) = &value.base else {
        return graphql_error("list-valued GraphQL variables are not lowerable");
    };
    if matches!(
        name.as_str(),
        "Boolean" | "ID" | "Int" | "String" | "Unsigned"
    ) {
        Ok(())
    } else {
        graphql_error(format!(
            "GraphQL variable type {:?} is not in the lowering contract",
            name.as_str()
        ))
    }
}

fn validate_variable_value(name: &str, value_type: &Type, value: &RuntimeValue) -> Result<()> {
    if matches!(value, RuntimeValue::Null) {
        return if value_type.nullable {
            Ok(())
        } else {
            graphql_error(format!("non-null GraphQL variable ${name} cannot be null"))
        };
    }
    let BaseType::Named(expected) = &value_type.base else {
        return graphql_error("list-valued GraphQL variables are not lowerable");
    };
    let valid = match expected.as_str() {
        "Boolean" => matches!(value, RuntimeValue::Bool(_)),
        "ID" | "String" => matches!(value, RuntimeValue::String(_)),
        "Int" => match value {
            RuntimeValue::Integer(value) => i32::try_from(*value).is_ok(),
            RuntimeValue::Unsigned(value) => i32::try_from(*value).is_ok(),
            _ => false,
        },
        "Unsigned" => matches!(value, RuntimeValue::Unsigned(_)),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        graphql_error(format!(
            "GraphQL variable ${name} does not match type {expected}"
        ))
    }
}

fn collect_selection_variables(selection: &Selection, output: &mut BTreeSet<String>) {
    match selection {
        Selection::Field(field) => {
            for (_, value) in &field.node.arguments {
                collect_value_variables(&value.node, output);
            }
            for nested in &field.node.selection_set.node.items {
                collect_selection_variables(&nested.node, output);
            }
        }
        Selection::FragmentSpread(_) => {}
        Selection::InlineFragment(fragment) => {
            for nested in &fragment.node.selection_set.node.items {
                collect_selection_variables(&nested.node, output);
            }
        }
    }
}

fn collect_value_variables(value: &Value, output: &mut BTreeSet<String>) {
    match value {
        Value::Variable(name) => {
            output.insert(name.to_string());
        }
        Value::List(values) => {
            for value in values {
                collect_value_variables(value, output);
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                collect_value_variables(value, output);
            }
        }
        Value::Null
        | Value::Number(_)
        | Value::String(_)
        | Value::Boolean(_)
        | Value::Binary(_)
        | Value::Enum(_) => {}
    }
}

fn resolve_scalar(value: &Value, parameters: &Parameters, label: &str) -> Result<RuntimeValue> {
    match value {
        Value::Variable(name) => parameters
            .get(name.as_str())
            .cloned()
            .ok_or_else(|| Error::Graphql(format!("missing GraphQL variable ${}", name.as_str()))),
        Value::Null => Ok(RuntimeValue::Null),
        Value::Boolean(value) => Ok(RuntimeValue::Bool(*value)),
        Value::String(value) => Ok(RuntimeValue::String(value.clone())),
        Value::Enum(value) => Ok(RuntimeValue::String(value.to_string())),
        Value::Number(value) => number_scalar(value, label),
        Value::Binary(_) | Value::List(_) | Value::Object(_) => {
            graphql_error(format!("GraphQL {label} must be a scalar"))
        }
    }
}

fn const_scalar(value: &ConstValue, label: &str) -> Result<RuntimeValue> {
    match value {
        ConstValue::Null => Ok(RuntimeValue::Null),
        ConstValue::Boolean(value) => Ok(RuntimeValue::Bool(*value)),
        ConstValue::String(value) => Ok(RuntimeValue::String(value.clone())),
        ConstValue::Enum(value) => Ok(RuntimeValue::String(value.to_string())),
        ConstValue::Number(value) => number_scalar(value, label),
        ConstValue::Binary(_) | ConstValue::List(_) | ConstValue::Object(_) => {
            graphql_error(format!("GraphQL {label} must be a scalar"))
        }
    }
}

fn number_scalar(value: &serde_json::Number, label: &str) -> Result<RuntimeValue> {
    if let Some(value) = value.as_u64() {
        Ok(RuntimeValue::Unsigned(value))
    } else if let Some(value) = value.as_i64() {
        Ok(RuntimeValue::Integer(value))
    } else {
        graphql_error(format!("GraphQL {label} must be an integer"))
    }
}

fn resolve_string(value: &Value, parameters: &Parameters, label: &str) -> Result<String> {
    match resolve_scalar(value, parameters, label)? {
        RuntimeValue::String(value) => Ok(value),
        _ => graphql_error(format!("GraphQL {label} must be a string")),
    }
}

fn resolve_token(value: &Value, parameters: &Parameters, label: &str) -> Result<String> {
    match value {
        Value::Enum(value) => Ok(value.to_string()),
        _ => resolve_string(value, parameters, label),
    }
}

fn resolve_u64(value: &Value, parameters: &Parameters, label: &str) -> Result<u64> {
    match resolve_scalar(value, parameters, label)? {
        RuntimeValue::Unsigned(value) => Ok(value),
        _ => graphql_error(format!("GraphQL {label} must be unsigned")),
    }
}

fn resolve_usize(value: &Value, parameters: &Parameters, label: &str) -> Result<usize> {
    usize::try_from(resolve_u64(value, parameters, label)?)
        .map_err(|_| Error::Graphql(format!("GraphQL {label} exceeds platform capacity")))
}

fn require_unsigned_parameter(parameters: &Parameters, name: &str, label: &str) -> Result<u64> {
    match parameters.get(name) {
        Some(RuntimeValue::Unsigned(value)) => Ok(*value),
        Some(_) => graphql_error(format!("GraphQL {label} variable ${name} must be unsigned")),
        None => graphql_error(format!("missing GraphQL variable ${name}")),
    }
}

fn runtime_type(value: &Value, parameters: &Parameters, label: &str) -> Result<RuntimeType> {
    RuntimeType::new(resolve_string(value, parameters, label)?)
        .map_err(|error| Error::Graphql(error.to_string()))
}

fn resolve_object<'a>(
    value: &'a Value,
    label: &str,
) -> Result<&'a async_graphql_value::indexmap::IndexMap<async_graphql_value::Name, Value>> {
    let Value::Object(value) = value else {
        return graphql_error(format!("GraphQL {label} must be an input object"));
    };
    Ok(value)
}

fn object_value<'a>(
    object: &'a async_graphql_value::indexmap::IndexMap<async_graphql_value::Name, Value>,
    name: &str,
) -> Result<&'a Value> {
    object
        .get(name)
        .ok_or_else(|| Error::Graphql(format!("GraphQL input field {name:?} is required")))
}

fn reject_unknown_object_fields(
    object: &async_graphql_value::indexmap::IndexMap<async_graphql_value::Name, Value>,
    allowed: &BTreeSet<&str>,
    label: &str,
) -> Result<()> {
    let unknown = object
        .keys()
        .filter(|name| !allowed.contains(name.as_str()))
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    if unknown.is_empty() {
        Ok(())
    } else {
        graphql_error(format!(
            "GraphQL {label} contains unknown fields: {}",
            unknown.join(", ")
        ))
    }
}

fn validate_graphql_name(value: &str, label: &str) -> Result<()> {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return graphql_error(format!("GraphQL {label} must not be empty"));
    };
    if !(first == '_' || first.is_ascii_alphabetic())
        || !characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
        || value.starts_with("__")
    {
        return graphql_error(format!(
            "GraphQL {label} {value:?} is not a permitted GraphQL name"
        ));
    }
    Ok(())
}

fn graphql_error<T>(message: impl Into<String>) -> Result<T> {
    Err(Error::Graphql(message.into()))
}
