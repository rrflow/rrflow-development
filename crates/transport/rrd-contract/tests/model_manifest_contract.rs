use rrd_contract::{
    route_step_decision_schema_sha256, router_backend_descriptor_sha256,
    router_model_handshake_sha256, router_model_manifest_sha256, CanonicalId, RouteDecisionKind,
    RouterArtifactDescriptor, RouterBackendDescriptor, RouterBackendLimits, RouterGrammarBinding,
    RouterModelHandshake, RouterModelLimits, RouterModelManifest, RouterQuantizationBinding,
    RouterRuntimeBinding, ROUTER_CONTRACT_VERSION, ROUTER_MODEL_MANIFEST_CONTRACT_VERSION,
};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const MODEL_BYTES: &[u8] = b"golden router model bytes";
const TOKENIZER_BYTES: &[u8] = b"golden tokenizer bytes";
const RUNTIME_BYTES: &[u8] = b"golden runtime bytes";
const GRAMMAR_BYTES: &[u8] = b"golden deterministic grammar bytes";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct GoldenModelHandshake {
    manifest: RouterModelManifest,
    backend: RouterBackendDescriptor,
    handshake: RouterModelHandshake,
}

fn canonical_id(value: &str) -> CanonicalId {
    CanonicalId::new(value).unwrap()
}

fn digest(value: u64) -> String {
    format!("{value:064x}")
}

fn artifact(
    bytes: &[u8],
    media_type: &str,
    format: &str,
    format_revision: u64,
) -> RouterArtifactDescriptor {
    RouterArtifactDescriptor {
        media_type: media_type.into(),
        format: canonical_id(format),
        format_revision,
        encoded_bytes: bytes.len() as u64,
        sha256: rrd_core::digest::sha256_hex(bytes),
    }
}

fn backend_limits() -> RouterBackendLimits {
    RouterBackendLimits {
        maximum_request_bytes: 64 * 1024,
        maximum_response_bytes: 32 * 1024,
        maximum_intent_bytes: 4 * 1024,
        maximum_signals: 16,
        maximum_recipe_candidates: 16,
        maximum_branch_candidates: 16,
        maximum_context_seeds: 16,
        maximum_parameters: 16,
        maximum_execution_ms: 5_000,
    }
}

fn model_limits() -> RouterModelLimits {
    RouterModelLimits {
        maximum_model_bytes: 512 * 1024 * 1024,
        maximum_tokenizer_bytes: 64 * 1024 * 1024,
        maximum_runtime_bytes: 256 * 1024 * 1024,
        maximum_grammar_bytes: 1024 * 1024,
        maximum_resident_bytes: 1024 * 1024 * 1024,
        maximum_context_tokens: 32_768,
        maximum_input_tokens: 30_720,
        maximum_output_tokens: 2_048,
        maximum_threads: 16,
    }
}

fn decisions() -> BTreeSet<RouteDecisionKind> {
    BTreeSet::from([
        RouteDecisionKind::SelectRecipe,
        RouteDecisionKind::AdvanceBranch,
        RouteDecisionKind::RequestContext,
    ])
}

fn manifest() -> RouterModelManifest {
    let model = artifact(MODEL_BYTES, "application/vnd.ggml.gguf", "gguf", 3);
    let tokenizer = artifact(
        TOKENIZER_BYTES,
        "application/vnd.rrflow.tokenizer",
        "sentencepiece",
        1,
    );
    let routing_schema_sha256 = route_step_decision_schema_sha256().unwrap();
    let mut manifest = RouterModelManifest {
        contract_version: ROUTER_MODEL_MANIFEST_CONTRACT_VERSION,
        id: canonical_id("reference-router-model"),
        revision: 1,
        router_contract_version: ROUTER_CONTRACT_VERSION,
        model: model.clone(),
        tokenizer: tokenizer.clone(),
        routing_schema_sha256: routing_schema_sha256.clone(),
        decisions: decisions(),
        backend_limits: backend_limits(),
        model_limits: model_limits(),
        runtime: RouterRuntimeBinding {
            id: canonical_id("rrflow-cpu-router-runtime"),
            revision: 7,
            abi: canonical_id("rrflow-router-abi"),
            abi_revision: 1,
            model_format: model.format,
            model_format_revision: model.format_revision,
            tokenizer_format: tokenizer.format,
            tokenizer_format_revision: tokenizer.format_revision,
            device_class: canonical_id("cpu"),
            artifact: artifact(
                RUNTIME_BYTES,
                "application/vnd.rrflow.router-runtime",
                "elf-shared-object",
                1,
            ),
            configuration_sha256: digest(40),
        },
        quantization: RouterQuantizationBinding::Quantized {
            scheme: canonical_id("grouped-k-quantization"),
            revision: 2,
            configuration_sha256: digest(41),
        },
        grammar: RouterGrammarBinding {
            revision: 4,
            source_schema_sha256: routing_schema_sha256,
            artifact: artifact(
                GRAMMAR_BYTES,
                "application/vnd.rrflow.router-grammar",
                "gbnf",
                1,
            ),
        },
        manifest_sha256: digest(0),
    };
    manifest.manifest_sha256 = router_model_manifest_sha256(&manifest).unwrap();
    manifest
}

fn backend(manifest: &RouterModelManifest) -> RouterBackendDescriptor {
    let mut backend = RouterBackendDescriptor {
        contract_version: ROUTER_CONTRACT_VERSION,
        id: canonical_id("reference-router"),
        revision: 3,
        model_manifest_id: manifest.id.clone(),
        model_manifest_revision: manifest.revision,
        model_manifest_sha256: manifest.manifest_sha256.clone(),
        decisions: manifest.decisions.clone(),
        limits: manifest.backend_limits.clone(),
        descriptor_sha256: digest(0),
    };
    backend.descriptor_sha256 = router_backend_descriptor_sha256(&backend).unwrap();
    backend
}

fn handshake(
    manifest: &RouterModelManifest,
    backend: &RouterBackendDescriptor,
) -> RouterModelHandshake {
    let mut handshake = RouterModelHandshake {
        contract_version: ROUTER_MODEL_MANIFEST_CONTRACT_VERSION,
        manifest_id: manifest.id.clone(),
        manifest_revision: manifest.revision,
        manifest_sha256: manifest.manifest_sha256.clone(),
        backend_id: backend.id.clone(),
        backend_revision: backend.revision,
        backend_descriptor_sha256: backend.descriptor_sha256.clone(),
        model: manifest.model.clone(),
        tokenizer: manifest.tokenizer.clone(),
        routing_schema_sha256: manifest.routing_schema_sha256.clone(),
        decisions: manifest.decisions.clone(),
        backend_limits: manifest.backend_limits.clone(),
        model_limits: manifest.model_limits.clone(),
        runtime: manifest.runtime.clone(),
        quantization: manifest.quantization.clone(),
        grammar: manifest.grammar.clone(),
        handshake_sha256: digest(0),
    };
    seal_handshake(&mut handshake);
    handshake
}

fn seal_handshake(handshake: &mut RouterModelHandshake) {
    handshake.handshake_sha256 = router_model_handshake_sha256(handshake).unwrap();
}

fn golden() -> GoldenModelHandshake {
    let manifest = manifest();
    let backend = backend(&manifest);
    let handshake = handshake(&manifest, &backend);
    GoldenModelHandshake {
        manifest,
        backend,
        handshake,
    }
}

#[test]
fn manifest_and_handshake_match_the_provider_neutral_golden_contract() {
    let golden = golden();
    golden
        .handshake
        .validate_for(&golden.manifest, &golden.backend)
        .unwrap();

    let expected: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/model-manifest-v1.json")).unwrap();
    let actual = serde_json::to_value(&golden).unwrap();
    assert_eq!(
        actual,
        expected,
        "{}",
        serde_json::to_string_pretty(&actual).unwrap()
    );
    let reopened: GoldenModelHandshake = serde_json::from_value(expected).unwrap();
    assert_eq!(reopened, golden);
}

#[test]
fn generated_manifest_schema_is_closed_and_has_no_locator_or_provider_authority() {
    let schema = serde_json::to_value(schema_for!(GoldenModelHandshake)).unwrap();
    assert_eq!(schema["additionalProperties"], serde_json::json!(false));
    let definitions = schema["$defs"].as_object().unwrap();
    for name in [
        "RouterArtifactDescriptor",
        "RouterBackendDescriptor",
        "RouterBackendLimits",
        "RouterGrammarBinding",
        "RouterModelHandshake",
        "RouterModelLimits",
        "RouterModelManifest",
        "RouterRuntimeBinding",
    ] {
        assert_eq!(
            definitions[name]["additionalProperties"],
            serde_json::json!(false),
            "schema {name} must reject unknown fields"
        );
    }
    let quantization = &definitions["RouterQuantizationBinding"];
    assert_eq!(quantization["oneOf"].as_array().unwrap().len(), 2);
    assert!(quantization["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .all(|variant| variant["additionalProperties"] == serde_json::json!(false)));

    let encoded = serde_json::to_string(&golden()).unwrap();
    for forbidden in [
        "provider",
        "filesystem",
        "file_path",
        "download_url",
        "endpoint",
        "credential",
        "secret",
    ] {
        assert!(!encoded.contains(forbidden), "manifest leaked {forbidden}");
    }

    let mut unknown = serde_json::to_value(golden().manifest).unwrap();
    unknown["model_path"] = serde_json::json!("/tmp/model.gguf");
    assert!(serde_json::from_value::<RouterModelManifest>(unknown).is_err());
}

#[test]
fn manifest_rejects_invalid_artifacts_formats_resources_and_schema_before_binding() {
    let valid = manifest();

    let mut changed = valid.clone();
    changed.model.media_type = "Application/OCTET-STREAM".into();
    changed.manifest_sha256 = router_model_manifest_sha256(&changed).unwrap();
    assert!(changed.validate().is_err());

    let mut changed = valid.clone();
    changed.runtime.model_format_revision += 1;
    changed.manifest_sha256 = router_model_manifest_sha256(&changed).unwrap();
    assert!(changed.validate().is_err());

    let mut changed = valid.clone();
    changed.model_limits.maximum_input_tokens = changed.model_limits.maximum_context_tokens;
    changed.manifest_sha256 = router_model_manifest_sha256(&changed).unwrap();
    assert!(changed.validate().is_err());

    let mut changed = valid.clone();
    changed.routing_schema_sha256 = digest(90);
    changed.grammar.source_schema_sha256 = changed.routing_schema_sha256.clone();
    changed.manifest_sha256 = router_model_manifest_sha256(&changed).unwrap();
    assert!(changed.validate().is_err());

    let mut changed = valid;
    changed.grammar.source_schema_sha256 = digest(91);
    changed.manifest_sha256 = router_model_manifest_sha256(&changed).unwrap();
    assert!(changed.validate().is_err());
}

#[test]
fn each_runtime_declaration_mismatch_is_rejected_independently() {
    let manifest = manifest();
    let backend = backend(&manifest);
    let valid = handshake(&manifest, &backend);

    type HandshakeMutation = (&'static str, fn(&mut RouterModelHandshake));
    let cases: [HandshakeMutation; 22] = [
        ("contract", |value| value.contract_version += 1),
        ("manifest id", |value| {
            value.manifest_id = canonical_id("other-manifest")
        }),
        ("manifest revision", |value| value.manifest_revision += 1),
        ("manifest digest", |value| {
            value.manifest_sha256 = digest(60)
        }),
        ("backend id", |value| {
            value.backend_id = canonical_id("other-backend")
        }),
        ("backend revision", |value| value.backend_revision += 1),
        ("backend digest", |value| {
            value.backend_descriptor_sha256 = digest(61)
        }),
        ("model", |value| value.model.sha256 = digest(62)),
        ("tokenizer", |value| value.tokenizer.sha256 = digest(63)),
        ("routing schema", |value| {
            value.routing_schema_sha256 = digest(64)
        }),
        ("decisions", |value| {
            value.decisions.remove(&RouteDecisionKind::AdvanceBranch);
        }),
        ("backend limits", |value| {
            value.backend_limits.maximum_signals -= 1
        }),
        ("model limits", |value| {
            value.model_limits.maximum_resident_bytes -= 1
        }),
        ("runtime ABI", |value| value.runtime.abi_revision += 1),
        ("device class", |value| {
            value.runtime.device_class = canonical_id("cuda-gpu")
        }),
        ("runtime artifact digest", |value| {
            value.runtime.artifact.sha256 = digest(65)
        }),
        ("runtime configuration digest", |value| {
            value.runtime.configuration_sha256 = digest(66)
        }),
        ("quantization", |value| {
            if let RouterQuantizationBinding::Quantized { revision, .. } = &mut value.quantization {
                *revision += 1;
            }
        }),
        ("quantization configuration digest", |value| {
            if let RouterQuantizationBinding::Quantized {
                configuration_sha256,
                ..
            } = &mut value.quantization
            {
                *configuration_sha256 = digest(67);
            }
        }),
        ("grammar revision", |value| value.grammar.revision += 1),
        ("grammar source digest", |value| {
            value.grammar.source_schema_sha256 = digest(68)
        }),
        ("grammar artifact digest", |value| {
            value.grammar.artifact.sha256 = digest(69)
        }),
    ];

    for (name, mutate) in cases {
        let mut changed = valid.clone();
        mutate(&mut changed);
        seal_handshake(&mut changed);
        assert!(
            changed.validate_for(&manifest, &backend).is_err(),
            "{name} mismatch was accepted"
        );
    }
}

#[test]
fn every_backend_and_model_resource_limit_mismatch_is_rejected_independently() {
    let manifest = manifest();
    let backend = backend(&manifest);
    let valid = handshake(&manifest, &backend);

    type LimitMutation = (&'static str, fn(&mut RouterModelHandshake));
    let cases: [LimitMutation; 18] = [
        ("request bytes", |value| {
            value.backend_limits.maximum_request_bytes += 1
        }),
        ("response bytes", |value| {
            value.backend_limits.maximum_response_bytes += 1
        }),
        ("intent bytes", |value| {
            value.backend_limits.maximum_intent_bytes += 1
        }),
        ("signals", |value| value.backend_limits.maximum_signals += 1),
        ("recipe candidates", |value| {
            value.backend_limits.maximum_recipe_candidates += 1
        }),
        ("branch candidates", |value| {
            value.backend_limits.maximum_branch_candidates += 1
        }),
        ("context seeds", |value| {
            value.backend_limits.maximum_context_seeds += 1
        }),
        ("parameters", |value| {
            value.backend_limits.maximum_parameters += 1
        }),
        ("execution time", |value| {
            value.backend_limits.maximum_execution_ms += 1
        }),
        ("model bytes", |value| {
            value.model_limits.maximum_model_bytes += 1
        }),
        ("tokenizer bytes", |value| {
            value.model_limits.maximum_tokenizer_bytes += 1
        }),
        ("runtime bytes", |value| {
            value.model_limits.maximum_runtime_bytes += 1
        }),
        ("grammar bytes", |value| {
            value.model_limits.maximum_grammar_bytes += 1
        }),
        ("resident bytes", |value| {
            value.model_limits.maximum_resident_bytes += 1
        }),
        ("context tokens", |value| {
            value.model_limits.maximum_context_tokens += 1
        }),
        ("input tokens", |value| {
            value.model_limits.maximum_input_tokens -= 1
        }),
        ("output tokens", |value| {
            value.model_limits.maximum_output_tokens -= 1
        }),
        ("threads", |value| value.model_limits.maximum_threads += 1),
    ];

    for (name, mutate) in cases {
        let mut changed = valid.clone();
        mutate(&mut changed);
        seal_handshake(&mut changed);
        assert!(
            changed.validate_for(&manifest, &backend).is_err(),
            "{name} limit mismatch was accepted"
        );
    }
}

#[test]
fn stale_manifest_and_backend_digests_cannot_be_resealed_by_the_handshake() {
    let manifest = manifest();
    let backend = backend(&manifest);

    let mut changed_manifest = manifest.clone();
    changed_manifest.model.sha256 = digest(70);
    let mut observed = handshake(&changed_manifest, &backend);
    seal_handshake(&mut observed);
    assert!(observed.validate_for(&changed_manifest, &backend).is_err());

    let mut changed_backend = backend.clone();
    changed_backend.limits.maximum_signals -= 1;
    let mut observed = handshake(&manifest, &changed_backend);
    seal_handshake(&mut observed);
    assert!(observed.validate_for(&manifest, &changed_backend).is_err());

    let mut observed = handshake(&manifest, &backend);
    observed.handshake_sha256 = digest(72);
    assert!(observed.validate_for(&manifest, &backend).is_err());
}
