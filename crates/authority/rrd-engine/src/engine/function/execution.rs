use super::super::*;
use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExecutedFunction {
    pub output: QueryValue,
    pub interrupts_consumed: u32,
    pub fuel_consumed: u64,
}

impl RrdEngine {
    #[allow(clippy::too_many_arguments)]
    pub fn execute_function(
        &self,
        session_id: &CorrelationId,
        token: &CorrelationId,
        request: &ExecuteFunction,
        now: u64,
        request_id: &str,
        operation_id: &str,
    ) -> Result<FunctionExecutionResult> {
        request
            .validate()
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        self.authorize(
            session_id,
            token,
            SecurityAction::FunctionExecute,
            now,
            request_id,
            operation_id,
        )?;
        let catalogue = self.load_current_function_catalogue()?;
        let definition = catalogue
            .functions
            .get(&request.function_id)
            .ok_or(ServiceError::FunctionNotFound)?;
        let executed = execute_definition(definition, &request.input)?;
        let input_bytes = serde_json::to_vec(&request.input).map_err(contract_json)?;
        let output_bytes = serde_json::to_vec(&executed.output).map_err(contract_json)?;
        let result = FunctionExecutionResult {
            contract_version: rrd_contract::FUNCTION_CONTRACT_VERSION,
            catalogue_revision: catalogue.revision,
            catalogue_sha256: catalogue.sha256(),
            function_id: request.function_id.clone(),
            runtime: definition.runtime.kind(),
            source_sha256: definition.runtime.content_sha256().into(),
            input_sha256: digest::sha256_hex(&input_bytes),
            output_sha256: digest::sha256_hex(&output_bytes),
            output: executed.output,
            attempts: 1,
            interrupts_consumed: executed.interrupts_consumed,
            fuel_consumed: executed.fuel_consumed,
        };
        result
            .validate()
            .map_err(|error| ServiceError::Contract(error.to_string()))?;
        Ok(result)
    }
}

pub(super) fn execute_definition(
    definition: &FunctionDefinition,
    input: &QueryValue,
) -> Result<ExecutedFunction> {
    definition
        .validate()
        .map_err(|error| ServiceError::Contract(error.to_string()))?;
    let natural_input = query_value_to_json(input)?;
    let input_bytes = serde_json::to_vec(&natural_input).map_err(contract_json)?;
    if input_bytes.len() > definition.limits.max_input_bytes as usize {
        return Err(ServiceError::FunctionLimit(
            "function input exceeds its configured byte limit".into(),
        ));
    }
    let (output_bytes, interrupts_consumed, fuel_consumed) = match &definition.runtime {
        FunctionRuntime::JavaScriptEs2020 { source, .. } => {
            super::javascript::execute(source, &input_bytes, definition)?
        }
        FunctionRuntime::WebAssemblyV1 { module_base64, .. } => {
            let module = STANDARD
                .decode(module_base64)
                .map_err(|_| ServiceError::Contract("WebAssembly module is not base64".into()))?;
            super::webassembly::execute(&module, &input_bytes, definition)?
        }
    };
    if output_bytes.len() > definition.limits.max_output_bytes as usize {
        return Err(ServiceError::FunctionLimit(
            "function output exceeds its configured byte limit".into(),
        ));
    }
    let output: serde_json::Value = serde_json::from_slice(&output_bytes)
        .map_err(|error| ServiceError::Function(format!("invalid JSON output: {error}")))?;
    let output = json_to_query_value(output)?;
    rrd_contract::validate_function_value(&output)
        .map_err(|error| ServiceError::Contract(error.to_string()))?;
    Ok(ExecutedFunction {
        output,
        interrupts_consumed,
        fuel_consumed,
    })
}

fn query_value_to_json(value: &QueryValue) -> Result<serde_json::Value> {
    Ok(match value {
        QueryValue::Null => serde_json::Value::Null,
        QueryValue::Bool(value) => serde_json::Value::Bool(*value),
        QueryValue::Integer(value) => {
            let maximum = rrd_contract::MAX_FUNCTION_JSON_SAFE_INTEGER as i64;
            if !(-maximum..=maximum).contains(value) {
                return Err(ServiceError::Contract(
                    "function integer input exceeds the JSON-v1 exact range".into(),
                ));
            }
            serde_json::Value::Number((*value).into())
        }
        QueryValue::Unsigned(value) => {
            if *value > rrd_contract::MAX_FUNCTION_JSON_SAFE_INTEGER {
                return Err(ServiceError::Contract(
                    "function unsigned input exceeds the JSON-v1 exact range".into(),
                ));
            }
            serde_json::Value::Number((*value).into())
        }
        QueryValue::Decimal(value) => {
            let number: serde_json::Number = value.parse().map_err(|_| {
                ServiceError::Contract("function decimal input is not a JSON number".into())
            })?;
            if number.to_string() != *value {
                return Err(ServiceError::Contract(
                    "function decimal input is not canonical in the JSON-v1 numeric domain".into(),
                ));
            }
            serde_json::Value::Number(number)
        }
        QueryValue::String(value) | QueryValue::Digest(value) => {
            serde_json::Value::String(value.clone())
        }
        QueryValue::List(values) => serde_json::Value::Array(
            values
                .iter()
                .map(query_value_to_json)
                .collect::<Result<_>>()?,
        ),
        QueryValue::Map(values) => serde_json::Value::Object(
            values
                .iter()
                .map(|(key, value)| Ok((key.clone(), query_value_to_json(value)?)))
                .collect::<Result<_>>()?,
        ),
    })
}

pub(super) fn json_to_query_value(value: serde_json::Value) -> Result<QueryValue> {
    Ok(match value {
        serde_json::Value::Null => QueryValue::Null,
        serde_json::Value::Bool(value) => QueryValue::Bool(value),
        serde_json::Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                let maximum = rrd_contract::MAX_FUNCTION_JSON_SAFE_INTEGER as i64;
                if !(-maximum..=maximum).contains(&value) {
                    return Err(ServiceError::Contract(
                        "function integer output exceeds the JSON-v1 exact range".into(),
                    ));
                }
                QueryValue::Integer(value)
            } else if let Some(value) = value.as_u64() {
                if value > rrd_contract::MAX_FUNCTION_JSON_SAFE_INTEGER {
                    return Err(ServiceError::Contract(
                        "function unsigned output exceeds the JSON-v1 exact range".into(),
                    ));
                }
                QueryValue::Unsigned(value)
            } else {
                QueryValue::Decimal(value.to_string())
            }
        }
        serde_json::Value::String(value) => QueryValue::String(value),
        serde_json::Value::Array(values) => QueryValue::List(
            values
                .into_iter()
                .map(json_to_query_value)
                .collect::<Result<_>>()?,
        ),
        serde_json::Value::Object(values) => QueryValue::Map(
            values
                .into_iter()
                .map(|(key, value)| Ok((key, json_to_query_value(value)?)))
                .collect::<Result<_>>()?,
        ),
    })
}
