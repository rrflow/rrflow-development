use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SensitiveReadUse {
    CausalLogPage,
    AlternateCausalLogPage,
    DataSnapshotReducer,
    GraphSnapshotReducer,
}

impl SensitiveReadUse {
    fn needle(self) -> String {
        match self {
            Self::CausalLogPage => ["read", "_changes("].concat(),
            Self::AlternateCausalLogPage => ["runtime_read", "_changes("].concat(),
            Self::DataSnapshotReducer => ["RuntimeDataSnapshot::from", "_changes("].concat(),
            Self::GraphSnapshotReducer => ["RuntimeGraphSnapshot::from", "_changes("].concat(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct AllowedUse {
    path: &'static str,
    kind: SensitiveReadUse,
    count: usize,
    operation: &'static str,
}

const ALLOWED_USES: &[AllowedUse] = &[
    AllowedUse {
        path: "crates/operations/rrd-cluster/src/artifact_transfer.rs",
        kind: SensitiveReadUse::CausalLogPage,
        count: 1,
        operation: "bounded cluster artifact-transfer replay",
    },
    AllowedUse {
        path: "crates/compute/rrd-query/tests/query.rs",
        kind: SensitiveReadUse::CausalLogPage,
        count: 1,
        operation: "exact rrflowQL result oracle",
    },
    AllowedUse {
        path: "crates/compute/rrd-query/tests/query.rs",
        kind: SensitiveReadUse::GraphSnapshotReducer,
        count: 1,
        operation: "exact rrflowQL graph reducer oracle",
    },
    AllowedUse {
        path: "crates/authority/rrd-engine/src/engine/rollback.rs",
        kind: SensitiveReadUse::CausalLogPage,
        count: 1,
        operation: "explicit forward-rollback planning replay",
    },
    AllowedUse {
        path: "crates/authority/rrd-engine/src/engine/rollback.rs",
        kind: SensitiveReadUse::GraphSnapshotReducer,
        count: 2,
        operation: "explicit rollback current/target graph comparison",
    },
    AllowedUse {
        path: "crates/authority/rrd-engine/src/runtime/vector_catalog.rs",
        kind: SensitiveReadUse::CausalLogPage,
        count: 1,
        operation: "explicit quantization projection-catalogue rebuild",
    },
    AllowedUse {
        path: "crates/authority/rrd-engine/src/runtime/data_plane.rs",
        kind: SensitiveReadUse::CausalLogPage,
        count: 1,
        operation: "embedding trace-only conflict recovery verification",
    },
    AllowedUse {
        path: "crates/compute/rrd-vector/tests/engine_differential.rs",
        kind: SensitiveReadUse::CausalLogPage,
        count: 1,
        operation: "exact vector candidate oracle",
    },
    AllowedUse {
        path: "crates/kernel/rrd-core/src/runtime.rs",
        kind: SensitiveReadUse::DataSnapshotReducer,
        count: 1,
        operation: "kernel data reducer missing-table rejection self-test",
    },
    AllowedUse {
        path: "crates/kernel/rrd-core/src/runtime.rs",
        kind: SensitiveReadUse::GraphSnapshotReducer,
        count: 1,
        operation: "kernel graph reducer self-test",
    },
    AllowedUse {
        path: "crates/authority/rrd-engine/src/engine/diagnostic.rs",
        kind: SensitiveReadUse::CausalLogPage,
        count: 1,
        operation: "explicit bounded diagnostic replay",
    },
    AllowedUse {
        path: "crates/authority/rrd-engine/src/engine/diagnostic.rs",
        kind: SensitiveReadUse::GraphSnapshotReducer,
        count: 2,
        operation: "explicit diagnostic graph comparison",
    },
    AllowedUse {
        path: "crates/persistence/rrd-store/src/access/semantic_commit.rs",
        kind: SensitiveReadUse::DataSnapshotReducer,
        count: 1,
        operation: "retirement validation over directly selected versions",
    },
    AllowedUse {
        path: "crates/persistence/rrd-store/src/repository/runtime.rs",
        kind: SensitiveReadUse::DataSnapshotReducer,
        count: 2,
        operation: "data snapshot and preview over directly selected versions",
    },
    AllowedUse {
        path: "crates/persistence/rrd-store/src/repository/runtime.rs",
        kind: SensitiveReadUse::GraphSnapshotReducer,
        count: 1,
        operation: "transaction graph preview over directly selected versions",
    },
    AllowedUse {
        path: "crates/persistence/rrd-store/tests/direct_read_paths.rs",
        kind: SensitiveReadUse::CausalLogPage,
        count: 1,
        operation: "exact all-model change-log oracle",
    },
    AllowedUse {
        path: "crates/persistence/rrd-store/tests/direct_read_paths.rs",
        kind: SensitiveReadUse::DataSnapshotReducer,
        count: 1,
        operation: "exact all-model data reducer oracle",
    },
    AllowedUse {
        path: "crates/persistence/rrd-store/tests/catalog_revision.rs",
        kind: SensitiveReadUse::CausalLogPage,
        count: 3,
        operation: "stale and fresh catalogue-revision log API characterization",
    },
    AllowedUse {
        path: "crates/persistence/rrd-store/tests/snapshot.rs",
        kind: SensitiveReadUse::CausalLogPage,
        count: 5,
        operation: "bounded authenticated change-page and stamp characterization",
    },
    AllowedUse {
        path: "crates/persistence/rrd-store/tests/snapshot.rs",
        kind: SensitiveReadUse::GraphSnapshotReducer,
        count: 1,
        operation: "exact graph reducer oracle",
    },
];

#[test]
fn causal_log_and_snapshot_reducer_uses_match_the_explicit_allowlist() {
    let root = workspace_root();
    let mut rust_sources = Vec::new();
    collect_rust_sources(&root.join("crates"), &mut rust_sources);
    rust_sources.sort();

    let mut actual = BTreeMap::new();
    for path in rust_sources {
        let relative = path
            .strip_prefix(&root)
            .expect("crate source must be workspace-relative")
            .to_string_lossy()
            .replace('\\', "/");
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        let compact = source
            .chars()
            .filter(|character| !character.is_ascii_whitespace())
            .collect::<String>();
        for kind in [
            SensitiveReadUse::CausalLogPage,
            SensitiveReadUse::AlternateCausalLogPage,
            SensitiveReadUse::DataSnapshotReducer,
            SensitiveReadUse::GraphSnapshotReducer,
        ] {
            let count = count_identifier_bounded(&compact, &kind.needle());
            if count > 0 {
                actual.insert((relative.clone(), kind), count);
            }
        }
    }

    let mut expected = BTreeMap::new();
    for allowed in ALLOWED_USES {
        assert!(
            !allowed.operation.trim().is_empty() && allowed.count > 0,
            "sensitive read allowance must name a real operation and nonzero count: {allowed:?}"
        );
        assert!(
            expected
                .insert((allowed.path.to_owned(), allowed.kind), allowed.count)
                .is_none(),
            "sensitive read allowance duplicates path/kind: {allowed:?}"
        );
    }

    assert_eq!(
        actual, expected,
        "causal-log or kernel snapshot-reducer source closure changed; normal query, vector, retrieval, context, memory, and inference reads may not be allowlisted"
    );
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|candidate| {
            candidate.join("Cargo.toml").is_file() && candidate.join("crates").is_dir()
        })
        .expect("rrd-engine must live below the workspace root")
        .to_path_buf()
}

fn collect_rust_sources(directory: &Path, sources: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("cannot inspect {}: {error}", directory.display()))
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|error| panic!("cannot enumerate {}: {error}", directory.display()));
    entries.sort_by_key(fs::DirEntry::path);
    for entry in entries {
        let path = entry.path();
        let kind = entry
            .file_type()
            .unwrap_or_else(|error| panic!("cannot inspect {}: {error}", path.display()));
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            collect_rust_sources(&path, sources);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            sources.push(path);
        }
    }
}

fn count_identifier_bounded(source: &str, needle: &str) -> usize {
    source
        .match_indices(needle)
        .filter(|(offset, _)| {
            *offset == 0
                || !source.as_bytes()[offset - 1].is_ascii_alphanumeric()
                    && source.as_bytes()[offset - 1] != b'_'
        })
        .count()
}
