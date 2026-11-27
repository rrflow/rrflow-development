//! Package grouping, dependency edges and outward boundaries.

use super::*;

/// RRFlow Lab's crates by dependency tier. A lab crate may depend only on
/// crates in a lower tier; the `rrflow-lab-runner` tool alone composes every domain.
pub(super) const LAB_TIERS: [&[&str]; 4] = [
    &["rrflow-lab-systems", "rrflow-lab-datasets"],
    &[
        "rrflow-lab-benchmarks",
        "rrflow-lab-experiments",
        "rrflow-lab-evals",
        "rrflow-lab-probes",
        "rrflow-lab-tuning",
    ],
    // A definition pack carries tests, systems and references, so it reads the
    // benchmark catalog and references below it.
    &["rrflow-lab-definitions"],
    &["rrflow-lab-runner"],
];

/// Where an evaluation crate sits: the shared layer, RRFlow Lab, or release
/// qualification. Product crates are outside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EvaluationArea {
    Shared,
    Lab,
    Qualification,
}

pub(super) fn evaluation_area(manifest: &Path) -> Option<EvaluationArea> {
    let parts = manifest
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    match parts
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["lab", "core", ..] => Some(EvaluationArea::Shared),
        ["lab", "qualification-storage", ..] => Some(EvaluationArea::Qualification),
        ["lab", ..] => Some(EvaluationArea::Lab),
        _ => None,
    }
}

/// The evaluation layer measures RRFlow from outside: a product crate may use
/// it only in its own tests. RRFlow Lab and release qualification share
/// `rrflow-lab-core` and never depend on each other, a lab crate depends
/// only on lower tiers, and a crate's name follows where it sits.
#[test]
pub(super) fn evaluation_layer_is_measured_from_outside_and_keeps_its_parts_apart() {
    let metadata = workspace_metadata();
    let area = |name: &str| evaluation_area(&metadata.packages[name].manifest);
    let lab_tier = |name: &str| LAB_TIERS.iter().position(|tier| tier.contains(&name));
    for name in metadata.packages.keys() {
        match area(name) {
            Some(EvaluationArea::Lab) => assert!(
                lab_tier(name).is_some(),
                "lab crate {name} must be named rrflow-lab-runner or rrflow-lab-<domain> and placed in LAB_TIERS"
            ),
            Some(EvaluationArea::Qualification) => assert!(
                name.starts_with("rrflow-lab-qualification-"),
                "qualification crate {name} must be named rrflow-lab-qualification-<gate>"
            ),
            Some(EvaluationArea::Shared) | None => {}
        }
    }
    for dependency in &metadata.dependency_inputs {
        if !metadata.packages.contains_key(&dependency.name) {
            continue;
        }
        let (owner, used) = (dependency.owner.as_str(), dependency.name.as_str());
        let (from, to) = (area(owner), area(used));
        if to.is_some() && from.is_none() {
            assert_eq!(
                dependency.kind, "dev",
                "{owner} may use evaluation crate {used} only in its tests"
            );
        }
        assert!(
            !matches!(
                (from, to),
                (
                    Some(EvaluationArea::Lab),
                    Some(EvaluationArea::Qualification)
                ) | (
                    Some(EvaluationArea::Qualification),
                    Some(EvaluationArea::Lab)
                )
            ),
            "{owner} and {used} cross between RRFlow Lab and release qualification"
        );
        if owner == "rrflow-lab-core" {
            assert!(to.is_none(), "rrflow-lab-core depends on {used}");
        }
        if let (Some(EvaluationArea::Lab), Some(EvaluationArea::Lab)) = (from, to) {
            assert!(
                lab_tier(used) < lab_tier(owner),
                "lab crate {owner} depends on {used}, which is not in a lower tier"
            );
        }
    }
}

#[test]
pub(super) fn attunement_proposals_have_no_runtime_or_store_dependency() {
    let metadata = workspace_metadata();
    assert_exact(
        &metadata.packages["rrflow-attunement"].all,
        names(&[
            "rrflow-core",
            "serde",
            "serde_json",
            "gix-index",
            "gix-ignore",
            "gix-config",
            "filetime",
            "tree-sitter",
            "tree-sitter-rust",
            "tree-sitter-python",
            "tree-sitter-go",
            // The trait tree-sitter's query matches implement; it does not
            // re-export it.
            "streaming-iterator",
        ]),
        "attunement uses pure buffer parsing, kernel hashes and serialized observations",
    );
}

#[test]
pub(super) fn engine_has_exact_composition() {
    let metadata = workspace_metadata();
    assert_exact(
        production_workspace_dependencies(&metadata, "rrd-engine"),
        names(&[
            "rrflow-attunement",
            "rrd-contract",
            "rrflow-engram",
            "rrflow-security",
            "rrflow-cx",
            "rrflow-core",
            "rrflow-inference",
            "rrflow-observability",
            "rrflow-knowledge",
            "rrflow-cld",
            "rrflow-ql",
            "rrflow-db",
            "rrflow-vector",
        ]),
        "rrd-engine production workspace dependencies",
    );
}

#[test]
pub(super) fn daemon_depends_only_on_engine_and_contract() {
    let metadata = workspace_metadata();
    assert_exact(
        production_workspace_dependencies(&metadata, "rrd-server"),
        names(&["rrd-contract", "rrd-engine"]),
        "rrd-server production workspace dependencies",
    );
}

#[test]
pub(super) fn observation_transport_has_no_authority_or_storage_dependency() {
    let metadata = workspace_metadata();
    assert_exact(
        &metadata.packages["rrflow-observability"].all,
        names(&[
            "rrd-contract",
            "rrflow-core",
            "serde",
            "serde_json",
            "chrono",
            "ureq",
        ]),
        "observation transport is a stateless codec and bounded HTTP adapter",
    );
    let consumers = metadata
        .dependency_inputs
        .iter()
        .filter(|dependency| dependency.name == "rrflow-observability")
        .map(|dependency| {
            (
                dependency.owner.as_str(),
                dependency.kind.as_str(),
                dependency.optional,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(consumers, vec![("rrd-engine", "normal", true)]);
    let engine = &metadata.packages["rrd-engine"];
    assert!(engine.features["full"].contains("dep:rrflow-observability"));
    assert!(!engine.features["edge"].contains("dep:rrflow-observability"));
}

#[test]
pub(super) fn mcp_adapter_uses_only_engine_and_public_protocol_boundaries() {
    let metadata = workspace_metadata();
    assert_exact(
        production_workspace_dependencies(&metadata, "rrflow-mcp"),
        names(&["rrd-client", "rrd-contract", "rrd-engine"]),
        "rrflow-mcp embedded/daemon production boundaries",
    );
}

#[test]
pub(super) fn public_contract_and_client_stay_implementation_free() {
    let metadata = workspace_metadata();
    assert_exact(
        production_workspace_dependencies(&metadata, "rrd-contract"),
        BTreeSet::new(),
        "rrd-contract production workspace dependencies",
    );
    assert_exact(
        production_workspace_dependencies(&metadata, "rrd-client"),
        names(&["rrd-contract"]),
        "rrd-client production workspace dependencies",
    );
}

#[test]
pub(super) fn outward_consumers_cannot_bypass_the_engine_boundary() {
    let metadata = workspace_metadata();
    let internal_components = metadata.components_below_the_engine();

    assert_exact(
        &forbidden_edges(&metadata, "rrflow-cli", &internal_components),
        BTreeSet::new(),
        "CLI must use only the public embedded engine or daemon client boundary",
    );
    assert_exact(
        &forbidden_edges(&metadata, "rrflow-mcp", &internal_components),
        BTreeSet::new(),
        "MCP adapter must use only the engine boundary",
    );
}

#[test]
pub(super) fn outward_product_sources_do_not_import_physical_components() {
    let metadata = workspace_metadata();
    // Each component's crate as source names it, and the graph crate that was removed.
    let mut physical = metadata
        .components_below_the_engine()
        .iter()
        .map(|name| name.replace('-', "_"))
        .collect::<Vec<_>>();
    physical.push("rrd_graph".to_owned());
    let physical = physical.iter().map(String::as_str).collect::<Vec<_>>();
    let mut violations = Vec::new();
    for relative in [
        metadata.in_crate("rrd-client", "src"),
        metadata.in_crate("rrd-server", "src"),
        metadata.in_crate("rrflow-cli", "src"),
        metadata.in_crate("rrflow-cli", "examples"),
        metadata.in_crate("rrflow-mcp", "src"),
    ] {
        let directory = metadata.root.join(relative);
        if directory.is_dir() {
            collect_physical_import_violations(
                &metadata.root,
                &directory,
                &physical,
                &mut violations,
            );
        }
    }
    violations.sort();
    violations.dedup();
    assert!(
        violations.is_empty(),
        "outward product source imports a physical/domain component instead of rrd-engine, rrd-client, or rrd-contract: {violations:#?}"
    );
}

#[test]
pub(super) fn tracked_and_untracked_text_files_have_no_trailing_horizontal_whitespace() {
    let metadata = workspace_metadata();
    let output = Command::new("git")
        .current_dir(&metadata.root)
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .output()
        .expect("git ls-files must start");
    assert!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut violations = Vec::new();
    for encoded in output.stdout.split(|byte| *byte == 0) {
        if encoded.is_empty() {
            continue;
        }
        let relative = PathBuf::from(
            std::str::from_utf8(encoded).expect("repository paths must be valid UTF-8"),
        );
        let path = metadata.root.join(&relative);
        if !path.is_file() {
            continue;
        }
        let bytes = fs::read(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        if bytes.contains(&0) || std::str::from_utf8(&bytes).is_err() {
            continue;
        }
        for (index, encoded_line) in bytes.split(|byte| *byte == b'\n').enumerate() {
            let line = encoded_line.strip_suffix(b"\r").unwrap_or(encoded_line);
            if line.last().is_some_and(|byte| matches!(byte, b' ' | b'\t')) {
                violations.push(format!("{}:{}", relative.display(), index + 1));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "tracked or untracked text contains trailing horizontal whitespace: {violations:#?}"
    );
}

/// A first-party crate is declared once, in the root manifest, with the path
/// of the member that owns it. A member names it with `workspace = true`, so
/// moving a crate is one edit and no member holds a path to a sibling.
#[test]
pub(super) fn first_party_dependencies_are_declared_once_in_the_root_manifest() {
    let metadata = workspace_metadata();
    let mut violations = Vec::new();
    for (owner, package) in &metadata.packages {
        let manifest = metadata.root.join(&package.manifest);
        let source = fs::read_to_string(&manifest).expect("member manifest must be readable");
        for (index, line) in source.lines().enumerate() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let name = key.trim().split('.').next().unwrap_or_default();
            if metadata.packages.contains_key(name)
                && (key.contains(".path") || value.contains("path"))
            {
                violations.push(format!("{owner} line {}: {}", index + 1, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "first-party dependencies must use `workspace = true`: {violations:#?}"
    );

    let root = fs::read_to_string(metadata.root.join("Cargo.toml"))
        .expect("root manifest must be readable");
    let used = metadata
        .dependency_inputs
        .iter()
        .filter(|dependency| metadata.packages.contains_key(&dependency.name))
        .map(|dependency| dependency.name.as_str())
        .collect::<BTreeSet<_>>();
    for name in used {
        let directory = metadata.packages[name]
            .manifest
            .parent()
            .expect("a member manifest has a directory")
            .to_string_lossy()
            .replace('\\', "/");
        let declaration = format!("{name} = {{ path = \"{directory}\"");
        assert_eq!(
            root.lines()
                .filter(|line| line.starts_with(&declaration))
                .count(),
            1,
            "the root manifest must declare {name} once at {directory}"
        );
    }
}

/// The two outward-boundary rules above take their component list from the architecture
/// manifest. An empty or wrong list would let them pass without checking anything.
#[test]
pub(super) fn components_below_the_engine_come_from_the_architecture_manifest() {
    let metadata = workspace_metadata();
    let components = metadata.components_below_the_engine();
    for expected in ["rrflow-core", "rrflow-lsm", "rrflow-cld", "rrflow-db"] {
        assert!(
            components.contains(expected),
            "{expected} must be a component below the engine"
        );
    }
    for outward in ["rrd-contract", "rrd-engine", "rrd-client", "rrflow-cli"] {
        assert!(
            !components.contains(outward),
            "{outward} is not a component below the engine"
        );
    }
    for component in &components {
        assert!(
            metadata.packages.contains_key(component),
            "{component} is in the architecture manifest but is not a workspace package"
        );
    }
}
