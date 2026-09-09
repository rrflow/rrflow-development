use super::super::*;

const FUNCTION_CATALOGUE_HEAD_FORMAT_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FunctionCatalogueHead {
    format_version: u16,
    revision: u64,
    catalogue_sha256: String,
}

impl RrdEngine {
    #[allow(clippy::too_many_arguments)]
    pub fn replace_function_catalogue(
        &self,
        session_id: &CorrelationId,
        token: &CorrelationId,
        request: &ReplaceFunctionCatalogue,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<FunctionCatalogue> {
        request
            .validate()
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        self.authorize(
            session_id,
            token,
            SecurityAction::FunctionCatalogueWrite,
            now,
            request_id,
            operation_id,
        )?;
        let _transaction_guard = self
            .transaction_gate
            .lock()
            .map_err(|_| ServiceError::Storage("engine transaction gate is poisoned".into()))?;
        let head_key = function_catalogue_head_key(&self.instance);
        let expected_head = self.storage.control().get(&head_key)?;
        let current = self.load_function_catalogue_from_head(expected_head.as_deref())?;
        if current.revision == request.catalogue.revision
            && current.sha256() == request.catalogue.sha256()
        {
            return Ok(current);
        }
        if current.revision != request.expected_revision {
            return Err(ServiceError::StorageConflict(format!(
                "function catalogue expected revision {} but current revision is {}",
                request.expected_revision, current.revision
            )));
        }
        let catalogue_bytes = serde_json::to_vec(&request.catalogue).map_err(contract_json)?;
        let catalogue_sha256 = request.catalogue.sha256();
        let revision_key =
            function_catalogue_revision_key(&self.instance, request.catalogue.revision);
        if self.storage.control().get(&revision_key)?.is_some() {
            return Err(ServiceError::StorageConflict(
                "function catalogue revision identity already exists".into(),
            ));
        }
        let head = FunctionCatalogueHead {
            format_version: FUNCTION_CATALOGUE_HEAD_FORMAT_VERSION,
            revision: request.catalogue.revision,
            catalogue_sha256,
        };
        self.storage.control().commit_batch(&[
            ControlTransition {
                key: revision_key,
                expected: None,
                replacement: Some(catalogue_bytes),
                at: now,
                actor: format!("session:{}", session_id.as_str()),
                action: "function.catalogue.revision_published".into(),
                request_id: request_id.into(),
                operation_id: operation_id.into(),
            },
            ControlTransition {
                key: head_key,
                expected: expected_head,
                replacement: Some(serde_json::to_vec(&head).map_err(contract_json)?),
                at: now,
                actor: format!("session:{}", session_id.as_str()),
                action: "function.catalogue.head_advanced".into(),
                request_id: request_id.into(),
                operation_id: operation_id.into(),
            },
        ])?;
        Ok(request.catalogue.clone())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn function_catalogue(
        &self,
        session_id: &CorrelationId,
        token: &CorrelationId,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<FunctionCatalogue> {
        self.authorize(
            session_id,
            token,
            SecurityAction::FunctionCatalogueRead,
            now,
            request_id,
            operation_id,
        )?;
        self.load_current_function_catalogue()
    }

    pub(in crate::engine) fn load_current_function_catalogue(&self) -> Result<FunctionCatalogue> {
        let head = self
            .storage
            .control()
            .get(&function_catalogue_head_key(&self.instance))?;
        self.load_function_catalogue_from_head(head.as_deref())
    }

    pub(in crate::engine) fn load_function_catalogue_revision(
        &self,
        revision: u64,
    ) -> Result<FunctionCatalogue> {
        if revision == 0 {
            return Ok(FunctionCatalogue::empty());
        }
        let bytes = self
            .storage
            .control()
            .get(&function_catalogue_revision_key(&self.instance, revision))?
            .ok_or(ServiceError::FunctionCatalogueRevisionNotFound)?;
        decode_catalogue(&bytes, Some(revision), None)
    }

    fn load_function_catalogue_from_head(
        &self,
        head_bytes: Option<&[u8]>,
    ) -> Result<FunctionCatalogue> {
        let Some(head_bytes) = head_bytes else {
            return Ok(FunctionCatalogue::empty());
        };
        let head: FunctionCatalogueHead =
            serde_json::from_slice(head_bytes).map_err(contract_json)?;
        if head.format_version != FUNCTION_CATALOGUE_HEAD_FORMAT_VERSION || head.revision == 0 {
            return Err(ServiceError::Contract(
                "function catalogue head is invalid".into(),
            ));
        }
        let bytes = self
            .storage
            .control()
            .get(&function_catalogue_revision_key(
                &self.instance,
                head.revision,
            ))?
            .ok_or(ServiceError::FunctionCatalogueRevisionNotFound)?;
        decode_catalogue(&bytes, Some(head.revision), Some(&head.catalogue_sha256))
    }
}

fn decode_catalogue(
    bytes: &[u8],
    expected_revision: Option<u64>,
    expected_sha256: Option<&str>,
) -> Result<FunctionCatalogue> {
    let catalogue: FunctionCatalogue = serde_json::from_slice(bytes).map_err(contract_json)?;
    catalogue
        .validate()
        .map_err(|error| ServiceError::Contract(error.to_string()))?;
    if expected_revision.is_some_and(|revision| catalogue.revision != revision)
        || expected_sha256.is_some_and(|sha256| catalogue.sha256() != sha256)
    {
        return Err(ServiceError::Contract(
            "function catalogue revision or digest does not match its head".into(),
        ));
    }
    Ok(catalogue)
}

fn function_catalogue_head_key(instance: &CanonicalId) -> String {
    format!("server/state/{instance}/function-catalogue/head-v1")
}

fn function_catalogue_revision_key(instance: &CanonicalId, revision: u64) -> String {
    format!("server/state/{instance}/function-catalogue/revision/{revision:020}")
}
