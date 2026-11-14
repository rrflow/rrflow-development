//! Transaction mutations and their validation.

use crate::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Public multi-model mutation vocabulary. It is deliberately independent of
/// `rrd_core`; adapters lower these values into the authoritative runtime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "mutation", rename_all = "snake_case", deny_unknown_fields)]
pub enum TransactionMutation {
    AssertClaim {
        subject: CanonicalId,
        predicate: CanonicalId,
        object: String,
        valid_from: u64,
        tx_time: u64,
        producer: CanonicalId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        confidence: Option<f32>,
    },
    PutSchema {
        registry: DataSchemaRegistry,
    },
    PutRecord {
        reference: DataReference,
        valid_from: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        valid_to: Option<u64>,
        #[serde(default)]
        properties: DataProperties,
    },
    PutRelation {
        reference: DataReference,
        from: DataReference,
        to: DataReference,
        valid_from: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        valid_to: Option<u64>,
        #[serde(default)]
        properties: DataProperties,
    },
    AppendEvent {
        kind: CanonicalId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subject: Option<DataReference>,
        #[serde(default)]
        properties: DataProperties,
    },
    PutVector {
        reference: DataReference,
        subject: DataReference,
        collection_id: CanonicalId,
        vector_name: CanonicalId,
        field: CanonicalId,
        valid_from: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        valid_to: Option<u64>,
        value: DataVectorValue,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        provenance: Option<DataEmbeddingProvenance>,
        #[serde(default)]
        properties: DataProperties,
    },
    AppendSeriesSample {
        reference: DataReference,
        series: DataReference,
        observed_at: u64,
        value: DataSeriesValue,
        #[serde(default)]
        properties: DataProperties,
    },
    PutGeo {
        reference: DataReference,
        subject: DataReference,
        field: CanonicalId,
        valid_from: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        valid_to: Option<u64>,
        value: DataGeoValue,
        #[serde(default)]
        properties: DataProperties,
    },
    PublishObjectReference {
        reference: DataReference,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subject: Option<DataReference>,
        sha256: String,
        length: u64,
        media_type: String,
        receipt: DataObjectReceipt,
        #[serde(default)]
        properties: DataProperties,
    },
    /// Retire one currently live catalogue identity at modeled valid time.
    /// Event correction is expressed as this mutation plus one replacement
    /// `append_event` in the same transaction.
    RetireData {
        model: DataLogicalModel,
        target: DataTarget,
        effective_at: u64,
    },
}

impl TransactionMutation {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::AssertClaim {
                object,
                valid_from,
                tx_time,
                confidence,
                ..
            } => {
                if object.is_empty() || object.len() > MAX_MESSAGE_BYTES {
                    return invalid(format!(
                        "claim object length must be in 1..={MAX_MESSAGE_BYTES} bytes"
                    ));
                }
                if *valid_from == 0 || *tx_time == 0 {
                    return invalid("claim timestamps must be greater than zero");
                }
                if confidence
                    .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
                {
                    return invalid("claim confidence must be finite and in 0..=1");
                }
                Ok(())
            }
            Self::PutSchema { registry } => validate_data_schema(registry),
            Self::PutRecord {
                valid_from,
                valid_to,
                properties,
                ..
            }
            | Self::PutRelation {
                valid_from,
                valid_to,
                properties,
                ..
            }
            | Self::PutGeo {
                valid_from,
                valid_to,
                properties,
                ..
            } => {
                validate_data_window(*valid_from, *valid_to)?;
                validate_data_properties(properties)
            }
            Self::AppendEvent { properties, .. } => validate_data_properties(properties),
            Self::PutVector {
                valid_from,
                valid_to,
                value,
                provenance,
                properties,
                ..
            } => {
                validate_data_window(*valid_from, *valid_to)?;
                let dimensions = validate_data_vector(value)?;
                if let Some(provenance) = provenance {
                    validate_sha256(&provenance.source_sha256, "embedding source_sha256")?;
                    validate_sha256(&provenance.model_sha256, "embedding model_sha256")?;
                    if provenance.model.trim().is_empty()
                        || provenance.model.len() > MAX_MESSAGE_BYTES
                        || provenance.dimensions as usize != dimensions
                    {
                        return invalid("embedding provenance is inconsistent with the vector");
                    }
                    validate_data_properties(&provenance.generation_parameters)?;
                }
                validate_data_properties(properties)
            }
            Self::AppendSeriesSample {
                observed_at,
                value,
                properties,
                ..
            } => {
                if *observed_at == 0 {
                    return invalid("series observed_at must be greater than zero");
                }
                if matches!(value, DataSeriesValue::Decimal(value) if value.trim().is_empty()) {
                    return invalid("series decimal must not be empty");
                }
                validate_data_properties(properties)
            }
            Self::PublishObjectReference {
                sha256,
                media_type,
                receipt,
                properties,
                ..
            } => {
                validate_sha256(sha256, "object sha256")?;
                if media_type.trim().is_empty()
                    || receipt.backend.trim().is_empty()
                    || receipt.key.trim().is_empty()
                {
                    return invalid(
                        "object media type, receipt backend, and key must not be empty",
                    );
                }
                validate_data_properties(properties)
            }
            Self::RetireData {
                model,
                target,
                effective_at,
                ..
            } => {
                if *model == DataLogicalModel::ReasoningClaim {
                    return invalid(
                        "reasoning claims retire through their bitemporal claim contract",
                    );
                }
                if *effective_at == 0 {
                    return invalid("data retirement effective_at must be greater than zero");
                }
                match (data_model_is_event(*model), target) {
                    (true, DataTarget::Event { cursor, .. }) if *cursor > 0 => {}
                    (false, DataTarget::Reference { .. }) => {}
                    (true, DataTarget::Event { .. }) => {
                        return invalid("event retirement cursor must be greater than zero");
                    }
                    (true, DataTarget::Reference { .. }) => {
                        return invalid("event models retire by authenticated event cursor");
                    }
                    (false, DataTarget::Event { .. }) => {
                        return invalid("non-event models retire by canonical reference");
                    }
                }
                Ok(())
            }
        }
    }
}

pub(crate) fn validate_data_window(valid_from: u64, valid_to: Option<u64>) -> Result<()> {
    if valid_from == 0 || valid_to.is_some_and(|end| end <= valid_from) {
        return invalid("data validity window must start above zero and end after its start");
    }
    Ok(())
}

pub(crate) fn validate_data_properties(properties: &DataProperties) -> Result<()> {
    if properties
        .keys()
        .any(|name| name.is_empty() || name.len() > MAX_ID_BYTES || name.as_bytes().contains(&0))
    {
        return invalid("data property names must be bounded, non-empty, and contain no NUL");
    }
    let bytes = serde_json::to_vec(properties).map_err(|error| ContractError(error.to_string()))?;
    if bytes.len() > MAX_QUERY_PARAMETER_BYTES {
        return invalid(format!(
            "data properties may encode at most {MAX_QUERY_PARAMETER_BYTES} bytes per object"
        ));
    }
    Ok(())
}

pub(crate) fn validate_data_schema(registry: &DataSchemaRegistry) -> Result<()> {
    if registry.revision == 0 || registry.migration.trim().is_empty() {
        return invalid("data schema requires a positive revision and migration description");
    }
    if registry.tables.is_empty() {
        return invalid("data schema requires an explicit non-empty table map");
    }
    for (kind, table) in &registry.tables {
        if table.mode == DataSchemaMode::Schemaless
            && (!table.properties.is_empty() || table.allow_additional_properties)
        {
            return invalid(format!(
                "schemaless table {kind} cannot declare a strict property contract"
            ));
        }
        let family = data_model_family(table.model);
        if matches!(family, "record" | "relation" | "event")
            && (!table.properties.is_empty() || table.allow_additional_properties)
        {
            return invalid(format!(
                "table {kind} must keep {family} properties in its specialized schema"
            ));
        }
        let specialized = match family {
            "record" => registry.records.contains_key(kind),
            "relation" => registry.relations.contains_key(kind),
            "event" => registry.events.contains_key(kind),
            _ => false,
        };
        if matches!(family, "record" | "relation" | "event") {
            match (table.mode, specialized) {
                (DataSchemaMode::Strict, false) => {
                    return invalid(format!(
                        "strict table {kind} lacks its specialized {family} schema"
                    ));
                }
                (DataSchemaMode::Schemaless, true) => {
                    return invalid(format!(
                        "schemaless table {kind} also declares a strict {family} schema"
                    ));
                }
                _ => {}
            }
        }
    }
    for kind in registry.records.keys() {
        validate_specialized_data_table(registry, kind, "record")?;
    }
    for kind in registry.relations.keys() {
        validate_specialized_data_table(registry, kind, "relation")?;
    }
    for kind in registry.events.keys() {
        validate_specialized_data_table(registry, kind, "event")?;
    }
    for schema in registry.records.values() {
        if schema
            .unique_properties
            .iter()
            .any(|name| !schema.properties.contains_key(name))
        {
            return invalid("record schema unique properties must be declared properties");
        }
    }
    for schema in registry.relations.values() {
        if schema.from.is_empty()
            || schema.to.is_empty()
            || schema.max_outgoing == Some(0)
            || schema.max_incoming == Some(0)
        {
            return invalid("relation schema requires endpoints and positive cardinality limits");
        }
    }
    validate_data_organisation(registry)
}

/// Public-boundary checks of the engram organisation. `rrd-core` repeats them
/// as the authority at commit; an engine parity test keeps the two aligned.
pub(crate) fn validate_data_organisation(registry: &DataSchemaRegistry) -> Result<()> {
    let organisation = &registry.organisation;
    let is_builtin = |domain: &str| domain == DATA_DEFAULT_DOMAIN || domain == DATA_ENGINE_DOMAIN;
    let is_reserved =
        |id: &str| id == DATA_ENGINE_DOMAIN || id.starts_with(DATA_ENGINE_OWNED_PREFIX);
    if organisation.domains.len() + 2 > MAX_DATA_DOMAINS {
        return invalid(format!(
            "an engram may declare at most {} domains besides default and rrflow",
            MAX_DATA_DOMAINS - 2
        ));
    }
    for (id, domain) in &organisation.domains {
        if is_builtin(id.as_str()) || is_reserved(id.as_str()) {
            return invalid(format!(
                "domain {} is built in or reserved for the engine and cannot be declared",
                id.as_str()
            ));
        }
        validate_data_label("domain", id, &domain.title, &domain.description)?;
        if let Some(parent) = &domain.parent {
            if parent.as_str() == DATA_ENGINE_DOMAIN {
                return invalid(format!(
                    "domain {} cannot be placed inside rrflow",
                    id.as_str()
                ));
            }
            if parent.as_str() != DATA_DEFAULT_DOMAIN && !organisation.domains.contains_key(parent)
            {
                return invalid(format!(
                    "domain {} names unknown parent {}",
                    id.as_str(),
                    parent.as_str()
                ));
            }
        }
        let mut depth = 1;
        let mut current = id;
        while let Some(parent) = organisation.domains[current]
            .parent
            .as_ref()
            .filter(|parent| parent.as_str() != DATA_DEFAULT_DOMAIN)
        {
            depth += 1;
            if parent == id || depth > MAX_DATA_DOMAIN_DEPTH {
                return invalid(format!(
                    "domain {} is deeper than {MAX_DATA_DOMAIN_DEPTH} levels or part of a cycle",
                    id.as_str()
                ));
            }
            current = parent;
        }
    }
    let memberships = organisation
        .kind_domains
        .iter()
        .map(|entry| ("table type", entry))
        .chain(
            organisation
                .collection_domains
                .iter()
                .map(|entry| ("vector collection", entry)),
        );
    for (subject, (identity, domain)) in memberships {
        if subject == "table type" && !registry.tables.contains_key(identity) {
            return invalid(format!(
                "domain membership names unregistered table type {}",
                identity.as_str()
            ));
        }
        if identity.as_str().starts_with(DATA_ENGINE_OWNED_PREFIX) {
            return invalid(format!(
                "engine-owned {subject} {} always belongs to rrflow",
                identity.as_str()
            ));
        }
        if domain.as_str() == DATA_ENGINE_DOMAIN {
            return invalid(format!(
                "{subject} {} is not engine-owned and cannot be placed in rrflow",
                identity.as_str()
            ));
        }
        if domain.as_str() == DATA_DEFAULT_DOMAIN {
            return invalid(format!(
                "{subject} {} already belongs to default; omit the entry instead",
                identity.as_str()
            ));
        }
        if !organisation.domains.contains_key(domain) {
            return invalid(format!(
                "{subject} {} names unknown domain {}",
                identity.as_str(),
                domain.as_str()
            ));
        }
    }
    if organisation.categories.len() > MAX_DATA_CATEGORIES {
        return invalid(format!(
            "an engram may declare at most {MAX_DATA_CATEGORIES} categories"
        ));
    }
    for (id, category) in &organisation.categories {
        if is_reserved(id.as_str()) {
            return invalid(format!(
                "category {} is reserved for the engine",
                id.as_str()
            ));
        }
        validate_data_label("category", id, &category.title, &category.description)?;
        if category.tags.len() > MAX_DATA_TAGS_PER_CATEGORY {
            return invalid(format!(
                "category {} may declare at most {MAX_DATA_TAGS_PER_CATEGORY} tags",
                id.as_str()
            ));
        }
        for (tag, schema) in &category.tags {
            validate_data_label("tag", tag, &schema.title, &schema.description)?;
        }
    }
    for (kind, tags) in &organisation.kind_tags {
        if !registry.tables.contains_key(kind) {
            return invalid(format!(
                "kind tags name unregistered table type {}",
                kind.as_str()
            ));
        }
        if tags.is_empty() || tags.len() > MAX_DATA_TAGS_PER_KIND {
            return invalid(format!(
                "table type {} must carry 1 to {MAX_DATA_TAGS_PER_KIND} tags, or omit the entry",
                kind.as_str()
            ));
        }
        let mut single = BTreeSet::new();
        for reference in tags {
            let Some(category) = organisation.categories.get(&reference.category) else {
                return invalid(format!(
                    "table type {} uses unknown category {}",
                    kind.as_str(),
                    reference.category.as_str()
                ));
            };
            if category.targets == DataCategoryTargets::Records {
                return invalid(format!(
                    "category {} does not tag table types",
                    reference.category.as_str()
                ));
            }
            if !category.tags.contains_key(&reference.tag) {
                return invalid(format!(
                    "table type {} uses unknown tag {}:{}",
                    kind.as_str(),
                    reference.category.as_str(),
                    reference.tag.as_str()
                ));
            }
            if category.cardinality == DataTagCardinality::Single
                && !single.insert(&reference.category)
            {
                return invalid(format!(
                    "table type {} carries more than one tag of single-tag category {}",
                    kind.as_str(),
                    reference.category.as_str()
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_data_label(
    subject: &str,
    id: &CanonicalId,
    title: &str,
    description: &str,
) -> Result<()> {
    if title.trim().is_empty() || title.len() > MAX_DATA_ORGANISATION_TITLE_BYTES {
        return invalid(format!(
            "{subject} {} needs a title of 1 to {MAX_DATA_ORGANISATION_TITLE_BYTES} bytes",
            id.as_str()
        ));
    }
    if description.len() > MAX_DATA_ORGANISATION_DESCRIPTION_BYTES {
        return invalid(format!(
            "{subject} {} description exceeds {MAX_DATA_ORGANISATION_DESCRIPTION_BYTES} bytes",
            id.as_str()
        ));
    }
    if title
        .chars()
        .chain(description.chars())
        .any(char::is_control)
    {
        return invalid(format!(
            "{subject} {} title and description cannot contain control characters",
            id.as_str()
        ));
    }
    Ok(())
}

pub(crate) fn data_model_family(model: DataLogicalModel) -> &'static str {
    match model {
        DataLogicalModel::Document
        | DataLogicalModel::Relational
        | DataLogicalModel::GraphNode
        | DataLogicalModel::KeyValue
        | DataLogicalModel::ReasoningRecord
        | DataLogicalModel::LifecycleRecord => "record",
        DataLogicalModel::GraphRelation => "relation",
        DataLogicalModel::Event
        | DataLogicalModel::ReasoningEvent
        | DataLogicalModel::LifecycleEvent => "event",
        DataLogicalModel::Vector => "vector",
        DataLogicalModel::TimeSeries => "time_series",
        DataLogicalModel::Geo => "geo",
        DataLogicalModel::Object => "object",
        DataLogicalModel::ReasoningClaim => "claim",
    }
}

pub(crate) fn data_model_is_event(model: DataLogicalModel) -> bool {
    matches!(
        model,
        DataLogicalModel::Event
            | DataLogicalModel::ReasoningEvent
            | DataLogicalModel::LifecycleEvent
    )
}

pub(crate) fn validate_specialized_data_table(
    registry: &DataSchemaRegistry,
    kind: &CanonicalId,
    family: &str,
) -> Result<()> {
    let table = registry.tables.get(kind).ok_or_else(|| {
        ContractError(format!(
            "specialized {family} schema {kind} has no explicit table entry"
        ))
    })?;
    if data_model_family(table.model) != family || table.mode != DataSchemaMode::Strict {
        return invalid(format!(
            "table {kind} conflicts with its strict specialized {family} schema"
        ));
    }
    Ok(())
}

pub(crate) fn validate_data_vector(value: &DataVectorValue) -> Result<usize> {
    let finite = |values: &[f32]| values.iter().all(|value| value.is_finite());
    let dimensions = match value {
        DataVectorValue::Dense { values } => {
            if !finite(values) {
                return invalid("dense vector values must be finite");
            }
            values.len()
        }
        DataVectorValue::Sparse {
            dimensions,
            indices,
            values,
        } => {
            if indices.is_empty()
                || indices.len() != values.len()
                || indices.iter().any(|index| index >= dimensions)
                || indices.windows(2).any(|pair| pair[0] >= pair[1])
                || !finite(values)
            {
                return invalid("sparse vector indices and values are invalid");
            }
            *dimensions as usize
        }
        DataVectorValue::MultiDense {
            dimensions,
            vectors,
        } => {
            if vectors.is_empty()
                || vectors
                    .iter()
                    .any(|vector| vector.len() != *dimensions as usize || !finite(vector))
            {
                return invalid("multi-dense vector rows are invalid");
            }
            *dimensions as usize
        }
    };
    if dimensions == 0 || dimensions > MAX_VECTOR_DIMENSIONS {
        return invalid(format!(
            "vector dimensions must be in 1..={MAX_VECTOR_DIMENSIONS}"
        ));
    }
    Ok(dimensions)
}
