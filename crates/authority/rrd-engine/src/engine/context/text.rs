//! Record text, reciprocal rank and the context digest.

use super::*;

pub(super) fn record_text(properties: &RuntimeProperties) -> String {
    let mut output = String::new();
    for (name, value) in properties {
        append_text(&mut output, name);
        append_runtime_text(&mut output, value);
    }
    output
}

pub(super) fn append_runtime_text(output: &mut String, value: &RuntimeValue) {
    match value {
        RuntimeValue::String(value) | RuntimeValue::Digest(value) => append_text(output, value),
        RuntimeValue::List(values) => {
            for value in values {
                append_runtime_text(output, value);
            }
        }
        RuntimeValue::Map(values) => {
            for (name, value) in values {
                append_text(output, name);
                append_runtime_text(output, value);
            }
        }
        RuntimeValue::Null
        | RuntimeValue::Bool(_)
        | RuntimeValue::Integer(_)
        | RuntimeValue::Unsigned(_)
        | RuntimeValue::Decimal(_) => {}
    }
}

pub(super) fn append_text(output: &mut String, value: &str) {
    if !value.trim().is_empty() {
        if !output.is_empty() {
            output.push(' ');
        }
        output.push_str(value);
    }
}

pub(super) fn reciprocal_rank(weight: f64, rank: u64) -> f64 {
    weight / (RRF_RANK_CONSTANT + rank as f64)
}

pub(super) fn context_digest<T: Serialize>(value: &T) -> Result<String> {
    let bytes =
        serde_json::to_vec(value).map_err(|error| ServiceError::Contract(error.to_string()))?;
    Ok(digest::sha256_hex(&bytes))
}
