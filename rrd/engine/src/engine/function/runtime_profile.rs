use super::super::*;

pub(super) const JAVASCRIPT_RUNTIME_PROFILE: &str = "javascript-es2020-json-v1";
pub(super) const WEBASSEMBLY_RUNTIME_PROFILE: &str = "webassembly-json-v1";

const JAVASCRIPT_RUNTIME_BUILD_DESCRIPTOR: &str = concat!(
    "rrflow-function-runtime-build-v1\n",
    "runtime=javascript_es2020\n",
    "abi=canonical_json_v1\n",
    "rquickjs=0.12.2\n",
    "rrd_engine=1.0.0\n",
);
const WEBASSEMBLY_RUNTIME_BUILD_DESCRIPTOR: &str = concat!(
    "rrflow-function-runtime-build-v1\n",
    "runtime=webassembly_v1\n",
    "abi=json_v1\n",
    "wasmi=1.1.0\n",
    "rrd_engine=1.0.0\n",
);

pub(in crate::engine) fn runtime_profile(kind: FunctionRuntimeKind) -> CanonicalId {
    CanonicalId::new(match kind {
        FunctionRuntimeKind::JavaScriptEs2020 => JAVASCRIPT_RUNTIME_PROFILE,
        FunctionRuntimeKind::WebAssemblyV1 => WEBASSEMBLY_RUNTIME_PROFILE,
    })
    .expect("embedded function runtime profile is canonical")
}

pub(in crate::engine) fn runtime_build_sha256(kind: FunctionRuntimeKind) -> String {
    digest::sha256_hex(match kind {
        FunctionRuntimeKind::JavaScriptEs2020 => JAVASCRIPT_RUNTIME_BUILD_DESCRIPTOR.as_bytes(),
        FunctionRuntimeKind::WebAssemblyV1 => WEBASSEMBLY_RUNTIME_BUILD_DESCRIPTOR.as_bytes(),
    })
}

pub(super) fn validate_runtime_identity(runtime: &FunctionRuntime) -> Result<()> {
    let kind = runtime.kind();
    if runtime.runtime_profile() != &runtime_profile(kind)
        || runtime.runtime_build_sha256() != runtime_build_sha256(kind)
    {
        return Err(ServiceError::Contract(format!(
            "function {} runtime profile or build is not available in this RRD",
            runtime.artifact_sha256()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_build_descriptors_are_frozen() {
        assert_eq!(
            runtime_build_sha256(FunctionRuntimeKind::JavaScriptEs2020),
            "7b565bf62da8b17fcb258102bf025eb9fc94e530a20f4d937317de81831a8474"
        );
        assert_eq!(
            runtime_build_sha256(FunctionRuntimeKind::WebAssemblyV1),
            "e704a363b683a872e9d5dfd30a46f879de901b0bdb8bf5b4f2eff2346a8290cc"
        );
    }
}
