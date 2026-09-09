use crate::access::put_standalone_function_receipt;
use crate::access::runtime_state::{checked_key, get, get_json, AccessRead};
use crate::access::validate_coordinate;
use crate::keyspaces::{self, Durability};
use crate::{
    Error, FunctionArtifactRecord, FunctionCatalogueHeadRecord, FunctionCatalogueMembershipRecord,
    FunctionCataloguePublication, FunctionCatalogueSnapshot, FunctionDefinitionRecord,
    FunctionInvocationReceiptRecord, Result, StorageEngine, TransactionFunctionBindingRecord,
};

const FUNCTION_TRANSACTION_ATTEMPTS: usize = 16;

/// Typed governed-function state over the one profile-neutral storage port.
///
/// This repository owns only immutable catalogue records, the active CAS head,
/// and standalone receipts. Authentication, runtime selection, schema
/// semantics, proposal lowering, and transaction coordination remain in
/// `RrdEngine`.
pub struct FunctionCatalogueRepository<'a> {
    storage: &'a dyn StorageEngine,
}

impl<'a> FunctionCatalogueRepository<'a> {
    pub(crate) const fn new(storage: &'a dyn StorageEngine) -> Self {
        Self { storage }
    }

    pub fn publish(
        &self,
        instance: &str,
        publication: &FunctionCataloguePublication,
    ) -> Result<FunctionCatalogueSnapshot> {
        validate_coordinate(instance, "function catalogue instance")?;
        publication.validate()?;
        publication.maximum_physical_batch_bytes(instance)?;
        for _ in 0..FUNCTION_TRANSACTION_ATTEMPTS {
            match self.publish_once(instance, publication) {
                Err(Error::TransactionConflict { .. }) => continue,
                result => return result,
            }
        }
        Err(Error::FunctionConstraint(
            "function catalogue contention exceeded its retry bound".into(),
        ))
    }

    fn publish_once(
        &self,
        instance: &str,
        publication: &FunctionCataloguePublication,
    ) -> Result<FunctionCatalogueSnapshot> {
        let mut transaction = self.storage.begin_transaction()?;
        let head_key = keyspaces::function_catalogue_head_key(instance);
        let current: Option<FunctionCatalogueHeadRecord> = get_json(
            &*transaction,
            keyspaces::FUNCTION_CATALOGUE_HEADS,
            &head_key,
        )?;
        if let Some(head) = &current {
            head.validate()?;
        }
        let next_head = publication.head();
        next_head.validate()?;
        if current.as_ref() == Some(&next_head) {
            return load_snapshot(&*transaction, instance, &next_head);
        }
        let current_revision = current.as_ref().map_or(0, |head| head.revision);
        if current_revision != publication.expected_revision {
            return Err(Error::FunctionConstraint(format!(
                "function catalogue expected revision {} but current revision is {current_revision}",
                publication.expected_revision
            )));
        }
        if current.as_ref().map(|head| head.membership_sha256.as_str())
            != publication.membership.predecessor_sha256.as_deref()
        {
            return Err(Error::FunctionConstraint(
                "function catalogue membership predecessor does not match the active head".into(),
            ));
        }

        for artifact in &publication.artifacts {
            let key = keyspaces::function_artifact_key(instance, &artifact.content_sha256)?;
            put_immutable(
                &mut *transaction,
                keyspaces::FUNCTION_ARTIFACTS,
                &key,
                artifact.encode()?,
                "function artifact",
            )?;
        }
        for definition in &publication.definitions {
            let key = keyspaces::function_definition_key(
                instance,
                &definition.function_id,
                &definition.definition_sha256,
            )?;
            put_immutable_json(
                &mut *transaction,
                keyspaces::FUNCTION_DEFINITIONS,
                &key,
                definition,
                "function definition",
            )?;
        }
        for binding in &publication.bindings {
            let key = keyspaces::transaction_function_binding_key(
                instance,
                &binding.binding_id,
                &binding.binding_sha256,
            )?;
            put_immutable_json(
                &mut *transaction,
                keyspaces::TRANSACTION_FUNCTION_BINDINGS,
                &key,
                binding,
                "transaction function binding",
            )?;
        }
        let membership_key =
            keyspaces::function_catalogue_membership_key(instance, publication.membership.revision);
        put_immutable_json(
            &mut *transaction,
            keyspaces::FUNCTION_CATALOGUE_MEMBERSHIPS,
            &membership_key,
            &publication.membership,
            "function catalogue membership",
        )?;
        transaction.put(
            checked_key(keyspaces::FUNCTION_CATALOGUE_HEADS, &head_key)?,
            serde_json::to_vec(&next_head)?,
        )?;
        transaction.commit(Durability::Authoritative)?;
        self.revision(instance, next_head.revision)?.ok_or_else(|| {
            Error::FunctionConstraint(
                "committed function catalogue revision is not readable".into(),
            )
        })
    }

    pub fn current(&self, instance: &str) -> Result<Option<FunctionCatalogueSnapshot>> {
        validate_coordinate(instance, "function catalogue instance")?;
        let transaction = self.storage.begin_transaction()?;
        let head: Option<FunctionCatalogueHeadRecord> = get_json(
            &*transaction,
            keyspaces::FUNCTION_CATALOGUE_HEADS,
            &keyspaces::function_catalogue_head_key(instance),
        )?;
        head.map(|head| {
            head.validate()?;
            load_snapshot(&*transaction, instance, &head)
        })
        .transpose()
    }

    pub fn revision(
        &self,
        instance: &str,
        revision: u64,
    ) -> Result<Option<FunctionCatalogueSnapshot>> {
        validate_coordinate(instance, "function catalogue instance")?;
        if revision == 0 {
            return Ok(None);
        }
        let transaction = self.storage.begin_transaction()?;
        let membership: Option<FunctionCatalogueMembershipRecord> = get_json(
            &*transaction,
            keyspaces::FUNCTION_CATALOGUE_MEMBERSHIPS,
            &keyspaces::function_catalogue_membership_key(instance, revision),
        )?;
        membership
            .map(|membership| {
                membership.validate()?;
                let head = FunctionCatalogueHeadRecord {
                    format_version: membership.format_version,
                    revision: membership.revision,
                    catalogue_sha256: membership.catalogue_sha256.clone(),
                    membership_sha256: membership.membership_sha256.clone(),
                };
                load_snapshot_with_membership(&*transaction, instance, head, membership)
            })
            .transpose()
    }

    pub fn commit_standalone_receipt(
        &self,
        instance: &str,
        receipt: &FunctionInvocationReceiptRecord,
    ) -> Result<()> {
        for _ in 0..FUNCTION_TRANSACTION_ATTEMPTS {
            let mut transaction = self.storage.begin_transaction()?;
            let changed = put_standalone_function_receipt(&mut *transaction, instance, receipt)?;
            if !changed {
                return Ok(());
            }
            match transaction.commit(Durability::Authoritative) {
                Err(Error::TransactionConflict { .. }) => continue,
                Ok(_) => return Ok(()),
                Err(error) => return Err(error),
            }
        }
        Err(Error::FunctionConstraint(
            "function receipt contention exceeded its retry bound".into(),
        ))
    }

    pub fn invocation_receipt(
        &self,
        instance: &str,
        invocation_id: &str,
    ) -> Result<Option<FunctionInvocationReceiptRecord>> {
        validate_coordinate(instance, "function receipt instance")?;
        validate_coordinate(invocation_id, "function invocation identity")?;
        let transaction = self.storage.begin_transaction()?;
        let receipt: Option<FunctionInvocationReceiptRecord> = get_json(
            &*transaction,
            keyspaces::INVOCATIONS,
            &keyspaces::function_invocation_receipt_key(instance, invocation_id),
        )?;
        if let Some(receipt) = &receipt {
            receipt.validate()?;
        }
        Ok(receipt)
    }
}

fn load_snapshot(
    reader: &(impl AccessRead + ?Sized),
    instance: &str,
    head: &FunctionCatalogueHeadRecord,
) -> Result<FunctionCatalogueSnapshot> {
    let membership: FunctionCatalogueMembershipRecord = get_json(
        reader,
        keyspaces::FUNCTION_CATALOGUE_MEMBERSHIPS,
        &keyspaces::function_catalogue_membership_key(instance, head.revision),
    )?
    .ok_or_else(|| {
        Error::FunctionConstraint("function catalogue head membership is missing".into())
    })?;
    load_snapshot_with_membership(reader, instance, head.clone(), membership)
}

fn load_snapshot_with_membership(
    reader: &(impl AccessRead + ?Sized),
    instance: &str,
    head: FunctionCatalogueHeadRecord,
    membership: FunctionCatalogueMembershipRecord,
) -> Result<FunctionCatalogueSnapshot> {
    head.validate()?;
    membership.validate()?;
    if head.revision != membership.revision
        || head.catalogue_sha256 != membership.catalogue_sha256
        || head.membership_sha256 != membership.membership_sha256
    {
        return Err(Error::FunctionConstraint(
            "function catalogue head does not match its membership".into(),
        ));
    }
    let artifacts = membership
        .artifact_sha256
        .iter()
        .map(|content_sha256| {
            let key = keyspaces::function_artifact_key(instance, content_sha256)?;
            let encoded = get(reader, keyspaces::FUNCTION_ARTIFACTS, &key)?.ok_or_else(|| {
                Error::FunctionConstraint(format!(
                    "function catalogue artifact {content_sha256} is missing"
                ))
            })?;
            FunctionArtifactRecord::decode(content_sha256, &encoded)
        })
        .collect::<Result<Vec<_>>>()?;
    let definitions = membership
        .function_definition_sha256
        .iter()
        .map(|(function_id, definition_sha256)| {
            let key = keyspaces::function_definition_key(instance, function_id, definition_sha256)?;
            let record: FunctionDefinitionRecord =
                get_json(reader, keyspaces::FUNCTION_DEFINITIONS, &key)?.ok_or_else(|| {
                    Error::FunctionConstraint(format!(
                    "function catalogue definition {function_id}@{definition_sha256} is missing"
                ))
                })?;
            record.validate()?;
            if record.function_id != *function_id || record.definition_sha256 != *definition_sha256
            {
                return Err(Error::FunctionConstraint(
                    "function definition record does not match membership".into(),
                ));
            }
            Ok(record)
        })
        .collect::<Result<Vec<_>>>()?;
    let bindings = membership
        .transaction_binding_sha256
        .iter()
        .map(|(binding_id, binding_sha256)| {
            let key =
                keyspaces::transaction_function_binding_key(instance, binding_id, binding_sha256)?;
            let record: TransactionFunctionBindingRecord =
                get_json(reader, keyspaces::TRANSACTION_FUNCTION_BINDINGS, &key)?.ok_or_else(
                    || {
                        Error::FunctionConstraint(format!(
                            "function catalogue binding {binding_id}@{binding_sha256} is missing"
                        ))
                    },
                )?;
            record.validate()?;
            if record.binding_id != *binding_id || record.binding_sha256 != *binding_sha256 {
                return Err(Error::FunctionConstraint(
                    "function binding record does not match membership".into(),
                ));
            }
            Ok(record)
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(FunctionCatalogueSnapshot {
        head,
        membership,
        artifacts,
        definitions,
        bindings,
    })
}

fn put_immutable_json<T: serde::Serialize>(
    transaction: &mut dyn crate::StorageTransaction,
    space: keyspaces::Space,
    key: &[u8],
    value: &T,
    name: &str,
) -> Result<()> {
    put_immutable(transaction, space, key, serde_json::to_vec(value)?, name)
}

fn put_immutable(
    transaction: &mut dyn crate::StorageTransaction,
    space: keyspaces::Space,
    key: &[u8],
    value: Vec<u8>,
    name: &str,
) -> Result<()> {
    let key = checked_key(space, key)?;
    match transaction.get(&key)? {
        Some(existing) if existing == value => Ok(()),
        Some(_) => Err(Error::FunctionConstraint(format!(
            "immutable {name} identity already contains other bytes"
        ))),
        None => transaction.put(key, value),
    }
}
