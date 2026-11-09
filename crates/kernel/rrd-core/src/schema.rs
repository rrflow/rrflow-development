//! One persisted catalogue and schema contract for every logical runtime model.

use crate::{
    Error, Result, RuntimeEvent, RuntimeMutation, RuntimeProperties, RuntimeRecord,
    RuntimeRelation, RuntimeType, RuntimeValue,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeValueType {
    Null,
    Bool,
    Integer,
    Unsigned,
    Decimal,
    String,
    Digest,
    List,
    Map,
}

impl RuntimeValueType {
    pub fn matches(self, value: &RuntimeValue) -> bool {
        matches!(
            (self, value),
            (Self::Null, RuntimeValue::Null)
                | (Self::Bool, RuntimeValue::Bool(_))
                | (Self::Integer, RuntimeValue::Integer(_))
                | (Self::Unsigned, RuntimeValue::Unsigned(_))
                | (Self::Decimal, RuntimeValue::Decimal(_))
                | (Self::String, RuntimeValue::String(_))
                | (Self::Digest, RuntimeValue::Digest(_))
                | (Self::List, RuntimeValue::List(_))
                | (Self::Map, RuntimeValue::Map(_))
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimePropertySchema {
    pub value_type: RuntimeValueType,
    #[serde(default)]
    pub required: bool,
}

impl RuntimePropertySchema {
    pub const fn required(value_type: RuntimeValueType) -> Self {
        Self {
            value_type,
            required: true,
        }
    }

    pub const fn optional(value_type: RuntimeValueType) -> Self {
        Self {
            value_type,
            required: false,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeRecordSchema {
    #[serde(default)]
    pub properties: BTreeMap<String, RuntimePropertySchema>,
    #[serde(default)]
    pub allow_additional_properties: bool,
    #[serde(default)]
    pub unique_properties: BTreeSet<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeRelationSchema {
    #[serde(default)]
    pub from: BTreeSet<RuntimeType>,
    #[serde(default)]
    pub to: BTreeSet<RuntimeType>,
    #[serde(default)]
    pub properties: BTreeMap<String, RuntimePropertySchema>,
    #[serde(default)]
    pub allow_additional_properties: bool,
    #[serde(default)]
    pub unique_pair: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_outgoing: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_incoming: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeEventSchema {
    #[serde(default)]
    pub subject_required: bool,
    #[serde(default)]
    pub subject_types: BTreeSet<RuntimeType>,
    #[serde(default)]
    pub properties: BTreeMap<String, RuntimePropertySchema>,
    #[serde(default)]
    pub allow_additional_properties: bool,
}

/// Logical meaning of one table identity inside a database catalogue.
///
/// Record-like models deliberately share the canonical [`RuntimeRecord`]
/// identity. Their distinct catalogue kind prevents document, relational,
/// graph, key-value, reasoning, and lifecycle APIs from creating competing
/// persisted truths for the same table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeLogicalModel {
    Document,
    Relational,
    GraphNode,
    GraphRelation,
    KeyValue,
    Vector,
    Event,
    TimeSeries,
    Geo,
    Object,
    ReasoningClaim,
    ReasoningRecord,
    ReasoningEvent,
    LifecycleRecord,
    LifecycleEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeSchemaMode {
    Strict,
    Schemaless,
}

/// A single table entry in the revisioned logical catalogue.
///
/// Record, relation, and event tables keep their richer constraints in the
/// existing specialized maps below. The property contract here governs the
/// structurally typed vector, time-series, geo, and object families. In
/// schemaless mode these fields must remain empty so mode cannot be inferred or
/// contradicted by a second boolean contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeTableSchema {
    pub model: RuntimeLogicalModel,
    pub mode: RuntimeSchemaMode,
    #[serde(default)]
    pub properties: BTreeMap<String, RuntimePropertySchema>,
    #[serde(default)]
    pub allow_additional_properties: bool,
}

impl RuntimeTableSchema {
    pub fn strict(model: RuntimeLogicalModel) -> Self {
        Self {
            model,
            mode: RuntimeSchemaMode::Strict,
            properties: BTreeMap::new(),
            allow_additional_properties: false,
        }
    }

    pub fn schemaless(model: RuntimeLogicalModel) -> Self {
        Self {
            model,
            mode: RuntimeSchemaMode::Schemaless,
            properties: BTreeMap::new(),
            allow_additional_properties: false,
        }
    }
}

/// Built-in domain holding every kind and collection not placed elsewhere.
pub const DEFAULT_DOMAIN: &str = "default";
/// Built-in domain holding the engine-owned kinds and collections.
pub const ENGINE_DOMAIN: &str = "rrflow";
/// Identity prefix that marks a kind or collection as engine-owned.
pub const ENGINE_OWNED_PREFIX: &str = "rrflow-";
pub const MAX_DOMAINS: usize = 256;
pub const MAX_DOMAIN_DEPTH: usize = 8;
pub const MAX_CATEGORIES: usize = 64;
pub const MAX_TAGS_PER_CATEGORY: usize = 4_096;
pub const MAX_TAGS_PER_KIND: usize = 64;
pub const MAX_ORGANISATION_TITLE_BYTES: usize = 128;
pub const MAX_ORGANISATION_DESCRIPTION_BYTES: usize = 1_024;

/// One configurable part of an estate's catalogue, such as a service, package
/// or subject area. Isolation arrives with its enforcement in H-11d.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeDomainSchema {
    pub title: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// `None` and `default` both place the domain at the top level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<RuntimeType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeCategoryTargets {
    Kinds,
    Records,
    KindsAndRecords,
}

impl RuntimeCategoryTargets {
    pub fn includes_kinds(self) -> bool {
        matches!(self, Self::Kinds | Self::KindsAndRecords)
    }

    pub fn includes_records(self) -> bool {
        matches!(self, Self::Records | Self::KindsAndRecords)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeTagCardinality {
    Single,
    Multiple,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeTagSchema {
    pub title: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
}

/// A governed vocabulary of tags. A closed category gains tags only through a
/// schema revision; an open one also admits tags written under its grant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCategorySchema {
    pub title: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub targets: RuntimeCategoryTargets,
    pub cardinality: RuntimeTagCardinality,
    #[serde(default)]
    pub open: bool,
    #[serde(default)]
    pub tags: BTreeMap<RuntimeType, RuntimeTagSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RuntimeTagRef {
    pub category: RuntimeType,
    pub tag: RuntimeType,
}

/// Domains, categories and memberships of one estate catalogue. A kind or
/// collection without an entry belongs to its default domain, so a registry
/// written before organisation existed keeps its meaning and its digest.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeEstateOrganisation {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub domains: BTreeMap<RuntimeType, RuntimeDomainSchema>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub categories: BTreeMap<RuntimeType, RuntimeCategorySchema>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub kind_domains: BTreeMap<RuntimeType, RuntimeType>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub collection_domains: BTreeMap<RuntimeType, RuntimeType>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub kind_tags: BTreeMap<RuntimeType, BTreeSet<RuntimeTagRef>>,
}

impl RuntimeEstateOrganisation {
    pub fn is_default(&self) -> bool {
        self.domains.is_empty()
            && self.categories.is_empty()
            && self.kind_domains.is_empty()
            && self.collection_domains.is_empty()
            && self.kind_tags.is_empty()
    }

    /// The one domain a kind belongs to.
    pub fn domain_of_kind<'a>(&'a self, kind: &'a RuntimeType) -> &'a str {
        self.kind_domains
            .get(kind)
            .map_or_else(|| default_domain(kind.as_str()), RuntimeType::as_str)
    }

    /// The one domain a vector collection belongs to.
    pub fn domain_of_collection<'a>(&'a self, collection: &'a str) -> &'a str {
        self.collection_domains
            .iter()
            .find(|(id, _)| id.as_str() == collection)
            .map_or_else(|| default_domain(collection), |(_, domain)| domain.as_str())
    }

    pub fn has_domain(&self, domain: &str) -> bool {
        is_builtin_domain(domain) || self.domains.keys().any(|id| id.as_str() == domain)
    }

    fn validate(&self, tables: &BTreeMap<RuntimeType, RuntimeTableSchema>) -> Result<()> {
        if self.domains.len() + 2 > MAX_DOMAINS {
            return invalid(format!(
                "an estate may declare at most {} domains besides default and rrflow",
                MAX_DOMAINS - 2
            ));
        }
        for (id, domain) in &self.domains {
            if is_builtin_domain(id.as_str()) || id.as_str().starts_with(ENGINE_OWNED_PREFIX) {
                return invalid(format!(
                    "domain {id} is built in or reserved for the engine and cannot be declared"
                ));
            }
            validate_label("domain", id, &domain.title, &domain.description)?;
            if let Some(parent) = &domain.parent {
                if parent.as_str() == ENGINE_DOMAIN {
                    return invalid(format!("domain {id} cannot be placed inside rrflow"));
                }
                if parent.as_str() != DEFAULT_DOMAIN && !self.domains.contains_key(parent) {
                    return invalid(format!("domain {id} names unknown parent {parent}"));
                }
            }
            self.domain_depth(id)?;
        }
        for (kind, domain) in &self.kind_domains {
            if !tables.contains_key(kind) {
                return invalid(format!(
                    "domain membership names unregistered table type {kind}"
                ));
            }
            self.validate_membership("table type", kind, domain)?;
        }
        for (collection, domain) in &self.collection_domains {
            self.validate_membership("vector collection", collection, domain)?;
        }
        if self.categories.len() > MAX_CATEGORIES {
            return invalid(format!(
                "an estate may declare at most {MAX_CATEGORIES} categories"
            ));
        }
        for (id, category) in &self.categories {
            if id.as_str() == ENGINE_DOMAIN || id.as_str().starts_with(ENGINE_OWNED_PREFIX) {
                return invalid(format!("category {id} is reserved for the engine"));
            }
            validate_label("category", id, &category.title, &category.description)?;
            if category.tags.len() > MAX_TAGS_PER_CATEGORY {
                return invalid(format!(
                    "category {id} may declare at most {MAX_TAGS_PER_CATEGORY} tags"
                ));
            }
            for (tag, schema) in &category.tags {
                validate_label("tag", tag, &schema.title, &schema.description)?;
            }
        }
        for (kind, tags) in &self.kind_tags {
            if !tables.contains_key(kind) {
                return invalid(format!("kind tags name unregistered table type {kind}"));
            }
            if tags.is_empty() {
                return invalid(format!(
                    "table type {kind} lists an empty tag set; omit the entry instead"
                ));
            }
            if tags.len() > MAX_TAGS_PER_KIND {
                return invalid(format!(
                    "table type {kind} may carry at most {MAX_TAGS_PER_KIND} tags"
                ));
            }
            let mut single = BTreeSet::new();
            for reference in tags {
                let category = self.categories.get(&reference.category).ok_or_else(|| {
                    Error::InvalidRuntime {
                        reason: format!(
                            "table type {kind} uses unknown category {}",
                            reference.category
                        ),
                    }
                })?;
                if !category.targets.includes_kinds() {
                    return invalid(format!(
                        "category {} does not tag table types",
                        reference.category
                    ));
                }
                if !category.tags.contains_key(&reference.tag) {
                    return invalid(format!(
                        "table type {kind} uses unknown tag {}:{}",
                        reference.category, reference.tag
                    ));
                }
                if category.cardinality == RuntimeTagCardinality::Single
                    && !single.insert(&reference.category)
                {
                    return invalid(format!(
                        "table type {kind} carries more than one tag of single-tag category {}",
                        reference.category
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_membership(
        &self,
        subject: &str,
        identity: &RuntimeType,
        domain: &RuntimeType,
    ) -> Result<()> {
        if identity.as_str().starts_with(ENGINE_OWNED_PREFIX) {
            return invalid(format!(
                "engine-owned {subject} {identity} always belongs to rrflow"
            ));
        }
        if domain.as_str() == ENGINE_DOMAIN {
            return invalid(format!(
                "{subject} {identity} is not engine-owned and cannot be placed in rrflow"
            ));
        }
        if domain.as_str() == DEFAULT_DOMAIN {
            return invalid(format!(
                "{subject} {identity} already belongs to default; omit the entry instead"
            ));
        }
        if !self.domains.contains_key(domain) {
            return invalid(format!(
                "{subject} {identity} names unknown domain {domain}"
            ));
        }
        Ok(())
    }

    /// Levels below `default`, refusing cycles and excess depth.
    fn domain_depth(&self, id: &RuntimeType) -> Result<usize> {
        let mut depth = 1;
        let mut current = id;
        while let Some(parent) = self.domains[current]
            .parent
            .as_ref()
            .filter(|parent| parent.as_str() != DEFAULT_DOMAIN)
        {
            if parent == id {
                return invalid(format!("domain {id} is its own ancestor"));
            }
            depth += 1;
            if depth > MAX_DOMAIN_DEPTH {
                return invalid(format!(
                    "domain {id} is deeper than {MAX_DOMAIN_DEPTH} levels or part of a cycle"
                ));
            }
            current = parent;
        }
        Ok(depth)
    }
}

fn is_builtin_domain(domain: &str) -> bool {
    domain == DEFAULT_DOMAIN || domain == ENGINE_DOMAIN
}

fn default_domain(identity: &str) -> &'static str {
    if identity.starts_with(ENGINE_OWNED_PREFIX) {
        ENGINE_DOMAIN
    } else {
        DEFAULT_DOMAIN
    }
}

fn validate_label(subject: &str, id: &RuntimeType, title: &str, description: &str) -> Result<()> {
    if title.trim().is_empty() || title.len() > MAX_ORGANISATION_TITLE_BYTES {
        return invalid(format!(
            "{subject} {id} needs a title of 1 to {MAX_ORGANISATION_TITLE_BYTES} bytes"
        ));
    }
    if description.len() > MAX_ORGANISATION_DESCRIPTION_BYTES {
        return invalid(format!(
            "{subject} {id} description exceeds {MAX_ORGANISATION_DESCRIPTION_BYTES} bytes"
        ));
    }
    if title
        .chars()
        .chain(description.chars())
        .any(char::is_control)
    {
        return invalid(format!(
            "{subject} {id} title and description cannot contain control characters"
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSchemaRegistry {
    pub revision: u64,
    pub migration: String,
    #[serde(default, skip_serializing_if = "RuntimeEstateOrganisation::is_default")]
    pub organisation: RuntimeEstateOrganisation,
    /// Sole logical-model authority for every schema-bound table. Specialized
    /// maps below add family-specific constraints; they never create a table
    /// or infer its model.
    pub tables: BTreeMap<RuntimeType, RuntimeTableSchema>,
    #[serde(default)]
    pub records: BTreeMap<RuntimeType, RuntimeRecordSchema>,
    #[serde(default)]
    pub relations: BTreeMap<RuntimeType, RuntimeRelationSchema>,
    #[serde(default)]
    pub events: BTreeMap<RuntimeType, RuntimeEventSchema>,
}

impl RuntimeSchemaRegistry {
    pub fn empty(revision: u64, migration: impl Into<String>) -> Self {
        Self {
            revision,
            migration: migration.into(),
            organisation: RuntimeEstateOrganisation::default(),
            tables: BTreeMap::new(),
            records: BTreeMap::new(),
            relations: BTreeMap::new(),
            events: BTreeMap::new(),
        }
    }

    /// Defines one strict record-like table and its specialized constraints as
    /// one in-memory authoring operation. A new schema revision may replace an
    /// existing definition in the same family, but cannot silently change the
    /// mutation family represented by a table identity.
    pub fn define_record_table(
        &mut self,
        kind: RuntimeType,
        model: RuntimeLogicalModel,
        schema: RuntimeRecordSchema,
    ) -> Result<bool> {
        if !model.is_record_like() {
            return invalid(format!(
                "record table type {kind} cannot use non-record model {model:?}"
            ));
        }
        self.reject_family_conflict(&kind, RuntimeMutationFamily::Record)?;
        let table = RuntimeTableSchema::strict(model);
        let changed =
            self.tables.get(&kind) != Some(&table) || self.records.get(&kind) != Some(&schema);
        self.tables.insert(kind.clone(), table);
        self.records.insert(kind, schema);
        Ok(changed)
    }

    /// Defines one strict graph-relation table and its endpoint/cardinality
    /// constraints as one in-memory authoring operation.
    pub fn define_relation_table(
        &mut self,
        kind: RuntimeType,
        schema: RuntimeRelationSchema,
    ) -> Result<bool> {
        self.reject_family_conflict(&kind, RuntimeMutationFamily::Relation)?;
        let table = RuntimeTableSchema::strict(RuntimeLogicalModel::GraphRelation);
        let changed =
            self.tables.get(&kind) != Some(&table) || self.relations.get(&kind) != Some(&schema);
        self.tables.insert(kind.clone(), table);
        self.relations.insert(kind, schema);
        Ok(changed)
    }

    /// Defines one strict event-like table and its subject/property constraints
    /// as one in-memory authoring operation.
    pub fn define_event_table(
        &mut self,
        kind: RuntimeType,
        model: RuntimeLogicalModel,
        schema: RuntimeEventSchema,
    ) -> Result<bool> {
        if !model.is_event_like() {
            return invalid(format!(
                "event table type {kind} cannot use non-event model {model:?}"
            ));
        }
        self.reject_family_conflict(&kind, RuntimeMutationFamily::Event)?;
        let table = RuntimeTableSchema::strict(model);
        let changed =
            self.tables.get(&kind) != Some(&table) || self.events.get(&kind) != Some(&schema);
        self.tables.insert(kind.clone(), table);
        self.events.insert(kind, schema);
        Ok(changed)
    }

    fn reject_family_conflict(
        &self,
        kind: &RuntimeType,
        family: RuntimeMutationFamily,
    ) -> Result<()> {
        if self
            .tables
            .get(kind)
            .is_some_and(|table| table.model.family() != family)
            || (family != RuntimeMutationFamily::Record && self.records.contains_key(kind))
            || (family != RuntimeMutationFamily::Relation && self.relations.contains_key(kind))
            || (family != RuntimeMutationFamily::Event && self.events.contains_key(kind))
        {
            return invalid(format!(
                "table type {kind} already belongs to a different mutation family"
            ));
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if self.revision == 0 {
            return invalid("runtime schema revision must be greater than zero");
        }
        if self.migration.trim().is_empty() {
            return invalid("runtime schema migration description must not be empty");
        }
        if self.tables.is_empty() {
            return invalid("runtime schema must declare an explicit canonical table map");
        }
        self.catalogue_tables()?;
        self.organisation.validate(&self.tables)?;
        for (kind, schema) in &self.records {
            validate_property_schema(kind, &schema.properties, schema.allow_additional_properties)?;
            for property in &schema.unique_properties {
                if !schema.properties.contains_key(property) {
                    return invalid(format!(
                        "record type {kind} declares unknown unique property {property:?}"
                    ));
                }
            }
        }
        for (kind, schema) in &self.relations {
            if schema.from.is_empty() || schema.to.is_empty() {
                return invalid(format!(
                    "relation type {kind} must declare at least one from and to endpoint type"
                ));
            }
            if schema.max_outgoing == Some(0) || schema.max_incoming == Some(0) {
                return invalid(format!(
                    "relation type {kind} cardinality limits must be greater than zero"
                ));
            }
            validate_property_schema(kind, &schema.properties, schema.allow_additional_properties)?;
        }
        for (kind, schema) in &self.events {
            validate_property_schema(kind, &schema.properties, schema.allow_additional_properties)?;
        }
        Ok(())
    }

    /// Validates and returns the exact authoritative table catalogue. No table
    /// identity or logical model is derived from specialized constraints.
    pub fn catalogue_tables(&self) -> Result<BTreeMap<RuntimeType, RuntimeTableSchema>> {
        for kind in self.records.keys() {
            validate_specialized_table(&self.tables, kind, RuntimeMutationFamily::Record)?;
        }
        for kind in self.relations.keys() {
            validate_specialized_table(&self.tables, kind, RuntimeMutationFamily::Relation)?;
        }
        for kind in self.events.keys() {
            validate_specialized_table(&self.tables, kind, RuntimeMutationFamily::Event)?;
        }
        for (kind, table) in &self.tables {
            validate_table(kind, table)?;
            let specialized = match table.model.family() {
                RuntimeMutationFamily::Record => self.records.contains_key(kind),
                RuntimeMutationFamily::Relation => self.relations.contains_key(kind),
                RuntimeMutationFamily::Event => self.events.contains_key(kind),
                RuntimeMutationFamily::Vector
                | RuntimeMutationFamily::Series
                | RuntimeMutationFamily::Geo
                | RuntimeMutationFamily::Object
                | RuntimeMutationFamily::Claim => false,
            };
            match (table.mode, table.model.family(), specialized) {
                (
                    RuntimeSchemaMode::Strict,
                    RuntimeMutationFamily::Record
                    | RuntimeMutationFamily::Relation
                    | RuntimeMutationFamily::Event,
                    false,
                ) => {
                    return invalid(format!(
                        "strict table type {kind} lacks its specialized {:?} schema",
                        table.model.family()
                    ));
                }
                (
                    RuntimeSchemaMode::Schemaless,
                    RuntimeMutationFamily::Record
                    | RuntimeMutationFamily::Relation
                    | RuntimeMutationFamily::Event,
                    true,
                ) => {
                    return invalid(format!(
                        "schemaless table type {kind} also declares a strict specialized schema"
                    ));
                }
                _ => {}
            }
        }
        Ok(self.tables.clone())
    }

    pub fn validate_objects<'a>(
        &self,
        mutations: impl IntoIterator<Item = &'a RuntimeMutation>,
        existing_records: impl IntoIterator<Item = &'a RuntimeRecord>,
        existing_relations: impl IntoIterator<Item = &'a RuntimeRelation>,
    ) -> Result<()> {
        self.validate()?;
        let tables = self.catalogue_tables()?;
        let mutations = mutations.into_iter().collect::<Vec<_>>();
        for mutation in &mutations {
            match mutation {
                RuntimeMutation::Claim { .. } | RuntimeMutation::Schema { .. } => {}
                RuntimeMutation::Retire { retirement } => {
                    let table = tables.get(&retirement.reference.kind).ok_or_else(|| {
                        Error::InvalidRuntime {
                            reason: format!(
                                "retirement table type {} is not registered in the catalogue",
                                retirement.reference.kind
                            ),
                        }
                    })?;
                    if table.model != retirement.model {
                        return invalid(format!(
                            "retirement target {}/{} is declared {:?}, not {:?}",
                            retirement.reference.kind,
                            retirement.reference.id,
                            table.model,
                            retirement.model
                        ));
                    }
                }
                RuntimeMutation::Record { record } => self.validate_record(record, &tables)?,
                RuntimeMutation::Relation { relation } => {
                    self.validate_relation(relation, &tables)?
                }
                RuntimeMutation::Event { event } => self.validate_event(event, &tables)?,
                RuntimeMutation::Vector { vector } => validate_structural_table(
                    "vector",
                    &vector.reference.kind,
                    &vector.properties,
                    RuntimeMutationFamily::Vector,
                    &tables,
                )?,
                RuntimeMutation::SeriesSample { sample } => validate_structural_table(
                    "time-series sample",
                    &sample.reference.kind,
                    &sample.properties,
                    RuntimeMutationFamily::Series,
                    &tables,
                )?,
                RuntimeMutation::Geo { geo } => validate_structural_table(
                    "geo value",
                    &geo.reference.kind,
                    &geo.properties,
                    RuntimeMutationFamily::Geo,
                    &tables,
                )?,
                RuntimeMutation::Object { object } => validate_structural_table(
                    "object",
                    &object.reference.kind,
                    &object.properties,
                    RuntimeMutationFamily::Object,
                    &tables,
                )?,
            }
        }

        let mut records = existing_records
            .into_iter()
            .map(|record| (record.reference.clone(), record.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut relations = existing_relations
            .into_iter()
            .map(|relation| (relation.reference.clone(), relation.clone()))
            .collect::<BTreeMap<_, _>>();
        for mutation in mutations {
            match mutation {
                RuntimeMutation::Record { record } => {
                    records.insert(record.reference.clone(), record.clone());
                }
                RuntimeMutation::Relation { relation } => {
                    relations.insert(relation.reference.clone(), relation.clone());
                }
                RuntimeMutation::Claim { .. }
                | RuntimeMutation::Event { .. }
                | RuntimeMutation::Schema { .. }
                | RuntimeMutation::Vector { .. }
                | RuntimeMutation::SeriesSample { .. }
                | RuntimeMutation::Geo { .. }
                | RuntimeMutation::Object { .. }
                | RuntimeMutation::Retire { .. } => {}
            }
        }
        self.validate_record_uniqueness(records.values())?;
        self.validate_relation_cardinality(relations.values())?;
        Ok(())
    }

    fn validate_record(
        &self,
        record: &RuntimeRecord,
        tables: &BTreeMap<RuntimeType, RuntimeTableSchema>,
    ) -> Result<()> {
        let table = require_table(
            &record.reference.kind,
            RuntimeMutationFamily::Record,
            tables,
        )?;
        if table.mode == RuntimeSchemaMode::Schemaless {
            return Ok(());
        }
        let schema =
            self.records
                .get(&record.reference.kind)
                .ok_or_else(|| Error::InvalidRuntime {
                    reason: format!("record type {} is not registered", record.reference.kind),
                })?;
        validate_properties(
            "record",
            &record.reference.kind,
            &record.properties,
            &schema.properties,
            schema.allow_additional_properties,
        )
    }

    fn validate_relation(
        &self,
        relation: &RuntimeRelation,
        tables: &BTreeMap<RuntimeType, RuntimeTableSchema>,
    ) -> Result<()> {
        let table = require_table(
            &relation.reference.kind,
            RuntimeMutationFamily::Relation,
            tables,
        )?;
        if table.mode == RuntimeSchemaMode::Schemaless {
            return Ok(());
        }
        let schema = self
            .relations
            .get(&relation.reference.kind)
            .ok_or_else(|| Error::InvalidRuntime {
                reason: format!(
                    "relation type {} is not registered",
                    relation.reference.kind
                ),
            })?;
        if !schema.from.contains(&relation.from.kind) || !schema.to.contains(&relation.to.kind) {
            return invalid(format!(
                "relation type {} rejects endpoint {} -> {}",
                relation.reference.kind, relation.from.kind, relation.to.kind
            ));
        }
        validate_properties(
            "relation",
            &relation.reference.kind,
            &relation.properties,
            &schema.properties,
            schema.allow_additional_properties,
        )
    }

    fn validate_event(
        &self,
        event: &RuntimeEvent,
        tables: &BTreeMap<RuntimeType, RuntimeTableSchema>,
    ) -> Result<()> {
        let table = require_table(&event.kind, RuntimeMutationFamily::Event, tables)?;
        if table.mode == RuntimeSchemaMode::Schemaless {
            return Ok(());
        }
        let schema = self
            .events
            .get(&event.kind)
            .ok_or_else(|| Error::InvalidRuntime {
                reason: format!("event type {} is not registered", event.kind),
            })?;
        if schema.subject_required && event.subject.is_none() {
            return invalid(format!("event type {} requires a subject", event.kind));
        }
        if let Some(subject) = &event.subject {
            if !schema.subject_types.is_empty() && !schema.subject_types.contains(&subject.kind) {
                return invalid(format!(
                    "event type {} rejects subject type {}",
                    event.kind, subject.kind
                ));
            }
        }
        validate_properties(
            "event",
            &event.kind,
            &event.properties,
            &schema.properties,
            schema.allow_additional_properties,
        )
    }

    fn validate_record_uniqueness<'a>(
        &self,
        records: impl IntoIterator<Item = &'a RuntimeRecord>,
    ) -> Result<()> {
        let records = records.into_iter().collect::<Vec<_>>();
        for (kind, schema) in &self.records {
            for property in &schema.unique_properties {
                let candidates = records
                    .iter()
                    .filter(|record| &record.reference.kind == kind)
                    .filter_map(|record| {
                        record
                            .properties
                            .get(property)
                            .map(|value| (*record, value))
                    })
                    .collect::<Vec<_>>();
                for (index, (left, value)) in candidates.iter().enumerate() {
                    if candidates[index + 1..].iter().any(|(right, other)| {
                        *value == *other
                            && windows_overlap(
                                left.valid_from,
                                left.valid_to,
                                right.valid_from,
                                right.valid_to,
                            )
                    }) {
                        return invalid(format!(
                            "record type {kind} property {property:?} is not unique for overlapping validity windows"
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_relation_cardinality<'a>(
        &self,
        relations: impl IntoIterator<Item = &'a RuntimeRelation>,
    ) -> Result<()> {
        let relations = relations.into_iter().collect::<Vec<_>>();
        for (kind, schema) in &self.relations {
            let typed = relations
                .iter()
                .filter(|relation| &relation.reference.kind == kind)
                .copied()
                .collect::<Vec<_>>();
            if schema.unique_pair {
                let mut groups = BTreeMap::new();
                for relation in &typed {
                    groups
                        .entry((relation.from.clone(), relation.to.clone()))
                        .or_insert_with(Vec::new)
                        .push(*relation);
                }
                for ((from, to), group) in groups {
                    if max_overlap(&group) > 1 {
                        return invalid(format!(
                            "relation type {kind} requires a unique pair for {}/{} -> {}/{}",
                            from.kind, from.id, to.kind, to.id
                        ));
                    }
                }
            }
            if let Some(limit) = schema.max_outgoing {
                let mut groups = BTreeMap::new();
                for relation in &typed {
                    groups
                        .entry(relation.from.clone())
                        .or_insert_with(Vec::new)
                        .push(*relation);
                }
                for (from, group) in groups {
                    if max_overlap(&group) > limit {
                        return invalid(format!(
                            "relation type {kind} exceeds max_outgoing={limit} at {}/{}",
                            from.kind, from.id
                        ));
                    }
                }
            }
            if let Some(limit) = schema.max_incoming {
                let mut groups = BTreeMap::new();
                for relation in &typed {
                    groups
                        .entry(relation.to.clone())
                        .or_insert_with(Vec::new)
                        .push(*relation);
                }
                for (to, group) in groups {
                    if max_overlap(&group) > limit {
                        return invalid(format!(
                            "relation type {kind} exceeds max_incoming={limit} at {}/{}",
                            to.kind, to.id
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeMutationFamily {
    Record,
    Relation,
    Event,
    Vector,
    Series,
    Geo,
    Object,
    Claim,
}

impl RuntimeLogicalModel {
    pub const fn is_record_like(self) -> bool {
        matches!(
            self,
            Self::Document
                | Self::Relational
                | Self::GraphNode
                | Self::KeyValue
                | Self::ReasoningRecord
                | Self::LifecycleRecord
        )
    }

    pub const fn is_event_like(self) -> bool {
        matches!(
            self,
            Self::Event | Self::ReasoningEvent | Self::LifecycleEvent
        )
    }

    fn family(self) -> RuntimeMutationFamily {
        match self {
            Self::Document
            | Self::Relational
            | Self::GraphNode
            | Self::KeyValue
            | Self::ReasoningRecord
            | Self::LifecycleRecord => RuntimeMutationFamily::Record,
            Self::GraphRelation => RuntimeMutationFamily::Relation,
            Self::Event | Self::ReasoningEvent | Self::LifecycleEvent => {
                RuntimeMutationFamily::Event
            }
            Self::Vector => RuntimeMutationFamily::Vector,
            Self::TimeSeries => RuntimeMutationFamily::Series,
            Self::Geo => RuntimeMutationFamily::Geo,
            Self::Object => RuntimeMutationFamily::Object,
            Self::ReasoningClaim => RuntimeMutationFamily::Claim,
        }
    }
}

impl RuntimeSchemaRegistry {
    pub fn logical_model(&self, kind: &RuntimeType) -> Result<RuntimeLogicalModel> {
        self.catalogue_tables()?
            .get(kind)
            .map(|table| table.model)
            .ok_or_else(|| Error::InvalidRuntime {
                reason: format!("table type {kind} is not registered in the catalogue"),
            })
    }
}

fn validate_specialized_table(
    tables: &BTreeMap<RuntimeType, RuntimeTableSchema>,
    kind: &RuntimeType,
    family: RuntimeMutationFamily,
) -> Result<()> {
    let table = require_table(kind, family, tables)?;
    if table.mode != RuntimeSchemaMode::Strict {
        return invalid(format!(
            "table type {kind} is schemaless but also has a strict {family:?} schema"
        ));
    }
    Ok(())
}

fn validate_table(kind: &RuntimeType, table: &RuntimeTableSchema) -> Result<()> {
    validate_property_schema(kind, &table.properties, table.allow_additional_properties)?;
    if table.mode == RuntimeSchemaMode::Schemaless
        && (!table.properties.is_empty() || table.allow_additional_properties)
    {
        return invalid(format!(
            "schemaless table type {kind} cannot also declare a strict property contract"
        ));
    }
    if matches!(
        table.model.family(),
        RuntimeMutationFamily::Record
            | RuntimeMutationFamily::Relation
            | RuntimeMutationFamily::Event
    ) && (!table.properties.is_empty() || table.allow_additional_properties)
    {
        return invalid(format!(
            "table type {kind} must keep record/relation/event properties in its specialized schema"
        ));
    }
    Ok(())
}

fn require_table<'a>(
    kind: &RuntimeType,
    family: RuntimeMutationFamily,
    tables: &'a BTreeMap<RuntimeType, RuntimeTableSchema>,
) -> Result<&'a RuntimeTableSchema> {
    let table = tables.get(kind).ok_or_else(|| Error::InvalidRuntime {
        reason: format!("{family:?} table type {kind} is not registered in the catalogue"),
    })?;
    if table.model.family() != family {
        return invalid(format!(
            "table type {kind} is catalogued as {:?}, not {family:?}",
            table.model
        ));
    }
    Ok(table)
}

fn validate_structural_table(
    object: &str,
    kind: &RuntimeType,
    properties: &RuntimeProperties,
    family: RuntimeMutationFamily,
    tables: &BTreeMap<RuntimeType, RuntimeTableSchema>,
) -> Result<()> {
    let table = require_table(kind, family, tables)?;
    if table.mode == RuntimeSchemaMode::Schemaless {
        return Ok(());
    }
    validate_properties(
        object,
        kind,
        properties,
        &table.properties,
        table.allow_additional_properties,
    )
}

fn validate_property_schema(
    kind: &RuntimeType,
    properties: &BTreeMap<String, RuntimePropertySchema>,
    _allow_additional: bool,
) -> Result<()> {
    for name in properties.keys() {
        if name.is_empty() || name.as_bytes().contains(&0) {
            return invalid(format!(
                "type {kind} contains invalid property name {name:?}"
            ));
        }
    }
    Ok(())
}

fn validate_properties(
    object: &str,
    kind: &RuntimeType,
    actual: &RuntimeProperties,
    declared: &BTreeMap<String, RuntimePropertySchema>,
    allow_additional: bool,
) -> Result<()> {
    for (name, property) in declared {
        match actual.get(name) {
            Some(value) if property.value_type.matches(value) => {}
            Some(_) => {
                return invalid(format!(
                    "{object} type {kind} property {name:?} has the wrong value type"
                ));
            }
            None if property.required => {
                return invalid(format!(
                    "{object} type {kind} is missing required property {name:?}"
                ));
            }
            None => {}
        }
    }
    if !allow_additional {
        if let Some(name) = actual.keys().find(|name| !declared.contains_key(*name)) {
            return invalid(format!(
                "{object} type {kind} contains undeclared property {name:?}"
            ));
        }
    }
    Ok(())
}

fn max_overlap(relations: &[&RuntimeRelation]) -> u64 {
    let mut points = Vec::with_capacity(relations.len() * 2);
    for relation in relations {
        points.push((relation.valid_from, 1_i8));
        if let Some(end) = relation.valid_to {
            points.push((end, -1_i8));
        }
    }
    // Half-open intervals: ends are processed before starts at the same time.
    points.sort_by_key(|(at, delta)| (*at, *delta));
    let mut current = 0_i64;
    let mut maximum = 0_i64;
    for (_, delta) in points {
        current += i64::from(delta);
        maximum = maximum.max(current);
    }
    maximum as u64
}

fn windows_overlap(
    left_from: u64,
    left_to: Option<u64>,
    right_from: u64,
    right_to: Option<u64>,
) -> bool {
    left_from < right_to.unwrap_or(u64::MAX) && right_from < left_to.unwrap_or(u64::MAX)
}

fn invalid<T>(reason: impl Into<String>) -> Result<T> {
    Err(Error::InvalidRuntime {
        reason: reason.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RuntimeId, RuntimeRef};

    fn reference(kind: &str, id: &str) -> RuntimeRef {
        RuntimeRef {
            kind: RuntimeType::new(kind).unwrap(),
            id: RuntimeId::new(id).unwrap(),
        }
    }

    #[test]
    fn canonical_table_map_is_required_and_never_inferred() {
        let prompt = RuntimeType::new("prompt").unwrap();
        let mut registry = RuntimeSchemaRegistry::empty(1, "reject missing table authority");
        registry
            .records
            .insert(prompt, RuntimeRecordSchema::default());

        assert!(registry
            .validate()
            .unwrap_err()
            .to_string()
            .contains("table"));
    }

    #[test]
    fn serialized_schema_cannot_omit_the_canonical_table_map() {
        let serialized = serde_json::json!({
            "revision": 1,
            "migration": "reject omitted table authority",
            "records": {
                "prompt": {}
            },
            "relations": {},
            "events": {}
        });

        assert!(serde_json::from_value::<RuntimeSchemaRegistry>(serialized).is_err());
    }

    #[test]
    fn paired_table_authoring_rejects_cross_family_reclassification() {
        let kind = RuntimeType::new("shared").unwrap();
        let mut registry = RuntimeSchemaRegistry::empty(1, "reject family reclassification");
        registry
            .define_record_table(
                kind.clone(),
                RuntimeLogicalModel::Relational,
                RuntimeRecordSchema::default(),
            )
            .unwrap();

        assert!(registry
            .define_relation_table(kind.clone(), RuntimeRelationSchema::default())
            .is_err());
        assert!(registry
            .define_event_table(
                kind,
                RuntimeLogicalModel::Event,
                RuntimeEventSchema::default(),
            )
            .is_err());
        assert!(registry
            .define_record_table(
                RuntimeType::new("vector").unwrap(),
                RuntimeLogicalModel::Vector,
                RuntimeRecordSchema::default(),
            )
            .is_err());
    }

    #[test]
    fn required_properties_and_endpoint_types_fail_closed() {
        let mut registry = RuntimeSchemaRegistry::empty(1, "bootstrap");
        let prompt = RuntimeType::new("prompt").unwrap();
        registry
            .define_record_table(
                prompt.clone(),
                RuntimeLogicalModel::Relational,
                RuntimeRecordSchema {
                    properties: BTreeMap::from([(
                        "text".into(),
                        RuntimePropertySchema::required(RuntimeValueType::String),
                    )]),
                    ..RuntimeRecordSchema::default()
                },
            )
            .unwrap();
        registry
            .define_relation_table(
                RuntimeType::new("caused").unwrap(),
                RuntimeRelationSchema {
                    from: BTreeSet::from([prompt.clone()]),
                    to: BTreeSet::from([prompt]),
                    unique_pair: true,
                    ..RuntimeRelationSchema::default()
                },
            )
            .unwrap();
        let record = RuntimeRecord {
            reference: reference("prompt", "p1"),
            valid_from: 1,
            valid_to: None,
            properties: RuntimeProperties::new(),
        };
        assert!(registry
            .validate_objects(
                [&RuntimeMutation::Record { record }],
                std::iter::empty(),
                std::iter::empty()
            )
            .unwrap_err()
            .to_string()
            .contains("missing required property"));
    }

    #[test]
    fn cardinality_is_measured_across_temporal_overlap() {
        let mut registry = RuntimeSchemaRegistry::empty(1, "bootstrap");
        let prompt = RuntimeType::new("prompt").unwrap();
        registry
            .define_record_table(
                prompt.clone(),
                RuntimeLogicalModel::Relational,
                RuntimeRecordSchema::default(),
            )
            .unwrap();
        registry
            .define_relation_table(
                RuntimeType::new("next").unwrap(),
                RuntimeRelationSchema {
                    from: BTreeSet::from([prompt.clone()]),
                    to: BTreeSet::from([prompt]),
                    max_outgoing: Some(1),
                    ..RuntimeRelationSchema::default()
                },
            )
            .unwrap();
        let relation = |id: &str, to: &str, from: u64, until: Option<u64>| RuntimeRelation {
            reference: reference("next", id),
            from: reference("prompt", "p1"),
            to: reference("prompt", to),
            valid_from: from,
            valid_to: until,
            properties: RuntimeProperties::new(),
        };
        let first = relation("a", "p2", 1, Some(5));
        let non_overlapping = relation("b", "p3", 5, None);
        registry
            .validate_objects(
                std::iter::empty(),
                std::iter::empty(),
                [&first, &non_overlapping],
            )
            .unwrap();
        let overlapping = relation("c", "p4", 4, None);
        assert!(registry
            .validate_objects(
                std::iter::empty(),
                std::iter::empty(),
                [&first, &overlapping]
            )
            .is_err());
    }

    #[test]
    fn one_catalogue_names_every_logical_model_and_schema_mode() {
        let mut registry = RuntimeSchemaRegistry::empty(1, "freeze unified catalogue");
        let models = [
            ("documents", RuntimeLogicalModel::Document),
            ("rows", RuntimeLogicalModel::Relational),
            ("nodes", RuntimeLogicalModel::GraphNode),
            ("edges", RuntimeLogicalModel::GraphRelation),
            ("entries", RuntimeLogicalModel::KeyValue),
            ("vectors", RuntimeLogicalModel::Vector),
            ("events", RuntimeLogicalModel::Event),
            ("samples", RuntimeLogicalModel::TimeSeries),
            ("places", RuntimeLogicalModel::Geo),
            ("objects", RuntimeLogicalModel::Object),
            ("claims", RuntimeLogicalModel::ReasoningClaim),
            ("reasoning", RuntimeLogicalModel::ReasoningRecord),
            ("reasoning_events", RuntimeLogicalModel::ReasoningEvent),
            ("lifecycles", RuntimeLogicalModel::LifecycleRecord),
            ("lifecycle_events", RuntimeLogicalModel::LifecycleEvent),
        ];
        for (kind, model) in models {
            registry.tables.insert(
                RuntimeType::new(kind).unwrap(),
                RuntimeTableSchema::schemaless(model),
            );
        }
        registry.validate().unwrap();
        assert_eq!(registry.catalogue_tables().unwrap().len(), models.len());
        assert_eq!(
            registry.tables[&RuntimeType::new("documents").unwrap()].mode,
            RuntimeSchemaMode::Schemaless
        );
    }

    #[test]
    fn strict_and_schemaless_or_conflicting_model_families_cannot_be_confused() {
        let prompt = RuntimeType::new("prompt").unwrap();
        let mut registry = RuntimeSchemaRegistry::empty(1, "reject ambiguous schema");
        registry
            .records
            .insert(prompt.clone(), RuntimeRecordSchema::default());
        registry.tables.insert(
            prompt.clone(),
            RuntimeTableSchema::schemaless(RuntimeLogicalModel::Document),
        );
        assert!(registry
            .validate()
            .unwrap_err()
            .to_string()
            .contains("schemaless"));

        registry.tables.insert(
            prompt,
            RuntimeTableSchema::strict(RuntimeLogicalModel::GraphRelation),
        );
        assert!(registry
            .validate()
            .unwrap_err()
            .to_string()
            .contains("not Record"));
    }

    fn kind(value: &str) -> RuntimeType {
        RuntimeType::new(value).unwrap()
    }

    fn domain(title: &str, parent: Option<&str>) -> RuntimeDomainSchema {
        RuntimeDomainSchema {
            title: title.into(),
            description: String::new(),
            parent: parent.map(kind),
        }
    }

    fn component_category(cardinality: RuntimeTagCardinality) -> RuntimeCategorySchema {
        RuntimeCategorySchema {
            title: "Component".into(),
            description: String::new(),
            targets: RuntimeCategoryTargets::KindsAndRecords,
            cardinality,
            open: false,
            tags: BTreeMap::from([
                (
                    kind("storage"),
                    RuntimeTagSchema {
                        title: "Storage".into(),
                        description: String::new(),
                    },
                ),
                (
                    kind("query"),
                    RuntimeTagSchema {
                        title: "Query".into(),
                        description: String::new(),
                    },
                ),
            ]),
        }
    }

    fn tag(category: &str, value: &str) -> RuntimeTagRef {
        RuntimeTagRef {
            category: kind(category),
            tag: kind(value),
        }
    }

    /// Two project kinds and one engine-owned kind, organised into a small tree.
    fn organised() -> RuntimeSchemaRegistry {
        let mut registry = RuntimeSchemaRegistry::empty(1, "organise the estate");
        for name in ["invoice", "customer", "rrflow-seat"] {
            registry.tables.insert(
                kind(name),
                RuntimeTableSchema::schemaless(RuntimeLogicalModel::Document),
            );
        }
        registry.organisation.domains = BTreeMap::from([
            (kind("billing"), domain("Billing", None)),
            (kind("invoices"), domain("Invoices", Some("billing"))),
        ]);
        registry.organisation.kind_domains = BTreeMap::from([(kind("invoice"), kind("invoices"))]);
        registry.organisation.categories = BTreeMap::from([(
            kind("component"),
            component_category(RuntimeTagCardinality::Single),
        )]);
        registry.organisation.kind_tags = BTreeMap::from([(
            kind("invoice"),
            BTreeSet::from([tag("component", "storage")]),
        )]);
        registry
    }

    fn rejection(registry: &RuntimeSchemaRegistry) -> String {
        registry.validate().unwrap_err().to_string()
    }

    #[test]
    fn every_kind_and_collection_belongs_to_exactly_one_domain() {
        let registry = organised();
        registry.validate().unwrap();
        let organisation = &registry.organisation;
        assert_eq!(organisation.domain_of_kind(&kind("invoice")), "invoices");
        assert_eq!(
            organisation.domain_of_kind(&kind("customer")),
            DEFAULT_DOMAIN
        );
        assert_eq!(
            organisation.domain_of_kind(&kind("rrflow-seat")),
            ENGINE_DOMAIN
        );
        assert_eq!(
            organisation.domain_of_collection("documents"),
            DEFAULT_DOMAIN
        );
        assert_eq!(
            organisation.domain_of_collection("rrflow-public-embedding"),
            ENGINE_DOMAIN
        );
        assert!(organisation.has_domain("billing"));
        assert!(organisation.has_domain(DEFAULT_DOMAIN));
        assert!(!organisation.has_domain("shipping"));
    }

    #[test]
    fn domains_reject_unknown_parents_cycles_and_excess_depth() {
        let mut registry = organised();
        registry
            .organisation
            .domains
            .insert(kind("orphan"), domain("Orphan", Some("missing")));
        assert!(rejection(&registry).contains("unknown parent"));

        let mut registry = organised();
        registry
            .organisation
            .domains
            .insert(kind("billing"), domain("Billing", Some("invoices")));
        assert!(
            rejection(&registry).contains("cycle") || rejection(&registry).contains("ancestor")
        );

        let mut registry = organised();
        let mut parent = None;
        for level in 0..=MAX_DOMAIN_DEPTH {
            let id = format!("level-{level}");
            registry
                .organisation
                .domains
                .insert(kind(&id), domain("Level", parent.as_deref()));
            parent = Some(id);
        }
        assert!(rejection(&registry).contains("deeper than"));

        let mut registry = organised();
        for index in 0..MAX_DOMAINS {
            registry
                .organisation
                .domains
                .insert(kind(&format!("domain-{index}")), domain("Many", None));
        }
        assert!(rejection(&registry).contains("at most"));
    }

    #[test]
    fn built_in_and_engine_domains_cannot_be_redefined_or_borrowed() {
        for reserved in [DEFAULT_DOMAIN, ENGINE_DOMAIN, "rrflow-private"] {
            let mut registry = organised();
            registry
                .organisation
                .domains
                .insert(kind(reserved), domain("Reserved", None));
            assert!(rejection(&registry).contains("reserved"), "{reserved}");
        }

        let mut registry = organised();
        registry
            .organisation
            .domains
            .insert(kind("inside-engine"), domain("Inside", Some(ENGINE_DOMAIN)));
        assert!(rejection(&registry).contains("inside rrflow"));

        let mut registry = organised();
        registry
            .organisation
            .kind_domains
            .insert(kind("rrflow-seat"), kind("billing"));
        assert!(rejection(&registry).contains("always belongs to rrflow"));

        let mut registry = organised();
        registry
            .organisation
            .kind_domains
            .insert(kind("customer"), kind(ENGINE_DOMAIN));
        assert!(rejection(&registry).contains("cannot be placed in rrflow"));

        let mut registry = organised();
        registry
            .organisation
            .collection_domains
            .insert(kind("rrflow-public-embedding"), kind("billing"));
        assert!(rejection(&registry).contains("always belongs to rrflow"));
    }

    #[test]
    fn memberships_name_registered_kinds_known_domains_and_no_redundant_default() {
        let mut registry = organised();
        registry
            .organisation
            .kind_domains
            .insert(kind("unregistered"), kind("billing"));
        assert!(rejection(&registry).contains("unregistered table type"));

        let mut registry = organised();
        registry
            .organisation
            .kind_domains
            .insert(kind("customer"), kind("shipping"));
        assert!(rejection(&registry).contains("unknown domain"));

        let mut registry = organised();
        registry
            .organisation
            .kind_domains
            .insert(kind("customer"), kind(DEFAULT_DOMAIN));
        assert!(rejection(&registry).contains("omit the entry"));

        let mut registry = organised();
        registry
            .organisation
            .collection_domains
            .insert(kind("documents"), kind("billing"));
        registry.validate().unwrap();
        assert_eq!(
            registry.organisation.domain_of_collection("documents"),
            "billing"
        );
    }

    #[test]
    fn kind_tags_follow_their_category_vocabulary_targets_and_cardinality() {
        let mut registry = organised();
        registry.organisation.kind_tags.insert(
            kind("customer"),
            BTreeSet::from([tag("component", "storage"), tag("component", "query")]),
        );
        assert!(rejection(&registry).contains("single-tag category"));

        registry.organisation.categories.insert(
            kind("component"),
            component_category(RuntimeTagCardinality::Multiple),
        );
        registry.validate().unwrap();

        let mut registry = organised();
        registry.organisation.kind_tags.insert(
            kind("customer"),
            BTreeSet::from([tag("component", "billing")]),
        );
        assert!(rejection(&registry).contains("unknown tag"));

        let mut registry = organised();
        registry.organisation.kind_tags.insert(
            kind("customer"),
            BTreeSet::from([tag("lifecycle", "draft")]),
        );
        assert!(rejection(&registry).contains("unknown category"));

        let mut registry = organised();
        let mut records_only = component_category(RuntimeTagCardinality::Single);
        records_only.targets = RuntimeCategoryTargets::Records;
        registry
            .organisation
            .categories
            .insert(kind("component"), records_only);
        assert!(rejection(&registry).contains("does not tag table types"));

        let mut registry = organised();
        registry
            .organisation
            .kind_tags
            .insert(kind("customer"), BTreeSet::new());
        assert!(rejection(&registry).contains("empty tag set"));
    }

    #[test]
    fn organisation_labels_are_bounded_titled_text() {
        let mut registry = organised();
        registry
            .organisation
            .domains
            .insert(kind("untitled"), domain(" ", None));
        assert!(rejection(&registry).contains("needs a title"));

        let mut registry = organised();
        registry.organisation.domains.insert(
            kind("long"),
            domain(&"x".repeat(MAX_ORGANISATION_TITLE_BYTES + 1), None),
        );
        assert!(rejection(&registry).contains("needs a title"));

        let mut registry = organised();
        let mut described = domain("Described", None);
        described.description = "x".repeat(MAX_ORGANISATION_DESCRIPTION_BYTES + 1);
        registry
            .organisation
            .domains
            .insert(kind("described"), described);
        assert!(rejection(&registry).contains("description exceeds"));

        let mut registry = organised();
        registry
            .organisation
            .domains
            .insert(kind("control"), domain("Line\nbreak", None));
        assert!(rejection(&registry).contains("control characters"));
    }

    #[test]
    fn default_organisation_is_omitted_from_the_persisted_registry() {
        let mut registry = organised();
        registry.organisation = RuntimeEstateOrganisation::default();
        let json = serde_json::to_value(&registry).unwrap();
        assert!(json.get("organisation").is_none());
        let organised = serde_json::to_value(organised()).unwrap();
        assert_eq!(
            organised["organisation"]["kind_domains"]["invoice"],
            serde_json::json!("invoices")
        );
    }
}
