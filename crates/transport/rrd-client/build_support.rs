//! Deterministic Rust type projection of the executable HTTP operation catalogue.
use std::collections::BTreeSet;

fn rust_type(name: &str) -> Result<String, String> {
    match name {
        "Empty" => Ok("()".into()),
        "OpenApiDocument" => Ok("serde_json::Value".into()),
        value
            if !value.is_empty()
                && value.as_bytes()[0].is_ascii_alphabetic()
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') =>
        {
            Ok(format!("rrd_contract::{value}"))
        }
        _ => Err(format!("invalid catalogue wire type {name:?}")),
    }
}

fn marker_name(operation: &str) -> Result<String, String> {
    let mut result = String::new();
    for part in operation.split('-') {
        let mut chars = part.chars();
        let first = chars.next().ok_or("empty operation name component")?;
        if !first.is_ascii_lowercase()
            || !chars
                .clone()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        {
            return Err(format!("unsupported operation name {operation:?}"));
        }
        result.push(first.to_ascii_uppercase());
        result.extend(chars);
    }
    Ok(result)
}

pub fn render(catalogue: &rrd_contract::EndpointCatalogue) -> Result<String, String> {
    catalogue.validate().map_err(|error| error.to_string())?;
    let mut operations = catalogue.endpoints.iter().collect::<Vec<_>>();
    operations.sort_by(|a, b| a.operation.as_str().cmp(b.operation.as_str()));
    let mut names = BTreeSet::new();
    let mut text =
        String::from("// Generated from rrd_contract::endpoint_catalogue; do not edit.\n");
    for endpoint in &operations {
        let name = marker_name(endpoint.operation.as_str())?;
        if !names.insert(name.clone()) {
            return Err(format!("duplicate Rust marker {name}"));
        }
        let request = rust_type(&endpoint.request_type)?;
        let response = rust_type(&endpoint.response_type)?;
        text.push_str(&format!("#[derive(Debug, Clone, Copy)]\npub struct {name};\nimpl GeneratedOperation for {name} {{\n    type Request = {request};\n    type Response = {response};\n    const ID: &'static str = {:?};\n}}\n", endpoint.operation.as_str()));
        if endpoint.authentication == rrd_contract::EndpointAuthentication::SessionBearer
            && endpoint.method == rrd_contract::HttpMethod::Post
            && !endpoint.path.contains('{')
        {
            if !endpoint.path.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'/' | b'_' | b'-')
            }) {
                return Err(format!(
                    "unsupported fixed Rust operation path {:?}",
                    endpoint.path
                ));
            }
            text.push_str(&format!("impl crate::catalogue_call::sealed::Sealed for {name} {{}}\nimpl crate::OperationMetadata for {name} {{\n    type Request = {request};\n    type Response = {response};\n    const OPERATION: &'static str = <Self as GeneratedOperation>::ID;\n    const PATH: &'static str = {:?};\n    const MUTATION: bool = {};\n}}\n", endpoint.path, endpoint.mutation));
        }
    }
    text.push_str("pub(crate) const GENERATED_IDS: &[&str] = &[\n");
    for endpoint in operations {
        text.push_str(&format!("    {:?},\n", endpoint.operation.as_str()));
    }
    text.push_str("];\n");
    Ok(text)
}
