use rrd_kubernetes::{
    applied_contract_sha256, desired_resources, DesiredInstanceInput, KubernetesResourceKind,
    RrdInstanceSpec, RrdStorageSpec, RRD_KUBERNETES_CONTRACT_VERSION,
};

fn spec() -> RrdInstanceSpec {
    RrdInstanceSpec {
        contract_version: RRD_KUBERNETES_CONTRACT_VERSION,
        image: format!("registry.example/rrd@sha256:{}", "a".repeat(64)),
        storage: RrdStorageSpec {
            size: "100Gi".into(),
            storage_class_name: Some("fast-rwo".into()),
            retain_on_delete: true,
        },
        tls_secret: "project-a-tls".into(),
        installation_plan_config_map: "project-a-install-plan".into(),
        installation_plan_sha256: "b".repeat(64),
    }
}

fn input<'a>(spec: &'a RrdInstanceSpec) -> DesiredInstanceInput<'a> {
    DesiredInstanceInput {
        namespace: "rrflow-system",
        name: "project-a",
        uid: "7c44c935-7d72-4f46-b34d-198c860c8152",
        generation: 1,
        spec,
    }
}

#[test]
fn one_secured_durable_instance_is_rendered_deterministically() {
    let spec = spec();
    let first = desired_resources(&input(&spec)).unwrap();
    let second = desired_resources(&input(&spec)).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.len(), 5);
    assert_eq!(applied_contract_sha256(&first).len(), 64);

    let stateful_set = first
        .iter()
        .find(|resource| resource.kind == KubernetesResourceKind::StatefulSet)
        .unwrap();
    assert_eq!(stateful_set.body["spec"]["replicas"], 1);
    assert_eq!(
        stateful_set.body["spec"]["persistentVolumeClaimRetentionPolicy"]["whenDeleted"],
        "Retain"
    );
    assert_eq!(
        stateful_set.body["spec"]["updateStrategy"]["type"],
        "OnDelete"
    );
    let pod = &stateful_set.body["spec"]["template"]["spec"];
    assert_eq!(pod["automountServiceAccountToken"], false);
    assert_eq!(pod["securityContext"]["runAsNonRoot"], true);
    assert_eq!(pod["initContainers"].as_array().unwrap().len(), 1);
    assert_eq!(pod["initContainers"][0]["command"][0], "rrflow");
    assert_eq!(pod["initContainers"][0]["args"][0], "install");
    assert_eq!(pod["initContainers"][0]["args"][1], "apply");
    assert!(pod["initContainers"][0]["args"]
        .as_array()
        .unwrap()
        .windows(2)
        .any(|arguments| arguments == ["--expect", spec.installation_plan_sha256.as_str()]));
    assert_eq!(pod["containers"][0]["command"][0], "rrd-server");
    assert_eq!(pod["containers"][0]["args"][0], "--project");
    assert!(pod["containers"][0]["args"]
        .as_array()
        .unwrap()
        .windows(2)
        .any(|arguments| { arguments == ["--distribution-executable", "/usr/local/bin/rrflow"] }));
    assert!(pod["containers"][0]["args"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "--tls-client-ca"));
    assert_eq!(pod["containers"][0]["image"], spec.image);
    let encoded_pod = serde_json::to_string(pod).unwrap();
    assert!(!encoded_pod.contains("initialize"));
    assert!(!encoded_pod.contains("rrd-security-bootstrap"));
    assert!(!encoded_pod.contains("bootstrapAtUnixMs"));
    assert!(!encoded_pod.contains("bootstrap-credentials"));

    let budget = first
        .iter()
        .find(|resource| resource.kind == KubernetesResourceKind::PodDisruptionBudget)
        .unwrap();
    assert_eq!(budget.body["spec"]["maxUnavailable"], 0);
    let network = first
        .iter()
        .find(|resource| resource.kind == KubernetesResourceKind::NetworkPolicy)
        .unwrap();
    assert_eq!(network.body["spec"]["egress"], serde_json::json!([]));
    assert_eq!(
        network.body["spec"]["ingress"][0]["from"][0]["namespaceSelector"]["matchLabels"]
            ["rrflow.io/rrd-client"],
        "true"
    );
}

#[test]
fn unsafe_or_fake_distributed_specs_fail_before_kubernetes_io() {
    let mut value = spec();
    value.image = "registry.example/rrd:latest".into();
    assert!(desired_resources(&input(&value)).is_err());

    let mut value = spec();
    value.storage.retain_on_delete = false;
    assert!(desired_resources(&input(&value)).is_err());

    let mut value = spec();
    value.tls_secret = "Bad_Secret".into();
    assert!(desired_resources(&input(&value)).is_err());

    let mut value = spec();
    value.installation_plan_sha256 = "ABC".into();
    assert!(desired_resources(&input(&value)).is_err());

    let value = spec();
    let unsafe_input = DesiredInstanceInput {
        uid: "",
        ..input(&value)
    };
    assert!(desired_resources(&unsafe_input).is_err());
}
