use rrd_attunement::git::*;

const V2: &[u8] = include_bytes!("fixtures/git-index/sha1-v2.index");
const V3: &[u8] = include_bytes!("fixtures/git-index/sha1-v3.index");
const V4: &[u8] = include_bytes!("fixtures/git-index/sha1-v4.index");
const SHA256: &[u8] = include_bytes!("fixtures/git-index/sha256-v2.index");
const SPLIT: &[u8] = include_bytes!("fixtures/git-index/sha1-split.index");

fn limits() -> ParseLimits {
    ParseLimits {
        input_bytes: 8192,
        entries: 32,
        path_bytes: 256,
        depth: 12,
        allocation_bytes: 65536,
    }
}
fn ignore_limits() -> IgnoreLimits {
    IgnoreLimits {
        parse: limits(),
        sources: 8,
        patterns: 64,
    }
}
fn worktree(directory: &str, bytes: &'static [u8]) -> IgnoreInput<'static> {
    IgnoreInput {
        origin: IgnoreOrigin::Worktree {
            directory: directory.into(),
        },
        bytes,
    }
}

#[test]
fn reads_actual_git_v2_v3_v4_and_sha256_indexes() {
    for (bytes, format) in [
        (V2, ObjectFormat::Sha1),
        (V3, ObjectFormat::Sha1),
        (V4, ObjectFormat::Sha1),
        (SHA256, ObjectFormat::Sha256),
    ] {
        let index = TrackedPaths::from_bytes(bytes, format, limits()).unwrap();
        assert_eq!(index.len(), 4);
        assert!(index.checksum_present());
        assert_eq!(index.kind("src/main.rs", false), Some(TrackedKind::File));
        assert_eq!(index.kind("link", false), Some(TrackedKind::Symlink));
        assert_eq!(
            index.kind(".github/workflows/check.yml", false),
            Some(TrackedKind::File)
        );
        assert!(index.has_descendants("ignored", false));
        assert!(!index.has_descendants("ignore", false));
        assert!(index.has_descendants("", false));
        assert!(index.has_descendants("IGNORED", true));
        assert_eq!(index.kind("SRC/MAIN.RS", true), Some(TrackedKind::File));
        assert_eq!(index.kind("SRC/MAIN.RS", false), None);
        assert_eq!(index.kind("missing", false), None);
    }
}

#[test]
fn rejects_checksum_corruption_and_incorrect_object_format() {
    let mut corrupted = V2.to_vec();
    corrupted[15] ^= 1;
    assert_eq!(
        TrackedPaths::from_bytes(&corrupted, ObjectFormat::Sha1, limits()).unwrap_err(),
        Error::ChecksumMismatch
    );
    assert!(TrackedPaths::from_bytes(SHA256, ObjectFormat::Sha1, limits()).is_err());
    assert!(TrackedPaths::from_bytes(V2, ObjectFormat::Sha256, limits()).is_err());
}

#[test]
fn zero_checksum_is_explicit_and_still_binds_all_input_bytes() {
    let original = TrackedPaths::from_bytes(V2, ObjectFormat::Sha1, limits()).unwrap();
    let mut bytes = V2.to_vec();
    let end = bytes.len();
    bytes[end - 20..].fill(0);
    let skipped = TrackedPaths::from_bytes(&bytes, ObjectFormat::Sha1, limits()).unwrap();
    assert!(!skipped.checksum_present());
    assert_ne!(skipped.input_sha256(), original.input_sha256());
    assert_eq!(skipped.len(), original.len());
}

#[test]
fn split_and_sparse_indexes_cannot_silently_lose_tracked_paths() {
    assert_eq!(
        TrackedPaths::from_bytes(SPLIT, ObjectFormat::Sha1, limits()).unwrap_err(),
        Error::SplitIndex
    );
    // Git-format sdir extension marks sparse even without a directory entry.
    let mut sparse = V2[..V2.len() - 20].to_vec();
    sparse.extend_from_slice(b"sdir\0\0\0\0");
    sparse.extend_from_slice(&[0; 20]);
    assert_eq!(
        TrackedPaths::from_bytes(&sparse, ObjectFormat::Sha1, limits()).unwrap_err(),
        Error::SparseIndex
    );
}

#[test]
fn index_limits_apply_before_and_after_decode() {
    let mut cap = limits();
    cap.input_bytes = V2.len() - 1;
    assert_eq!(
        TrackedPaths::from_bytes(V2, ObjectFormat::Sha1, cap).unwrap_err(),
        Error::InputLimit
    );
    cap = limits();
    cap.entries = 3;
    assert_eq!(
        TrackedPaths::from_bytes(V2, ObjectFormat::Sha1, cap).unwrap_err(),
        Error::EntryLimit
    );
    cap = limits();
    cap.path_bytes = 5;
    assert_eq!(
        TrackedPaths::from_bytes(V2, ObjectFormat::Sha1, cap).unwrap_err(),
        Error::PathLimit
    );
    cap = limits();
    cap.allocation_bytes = 1;
    assert_eq!(
        TrackedPaths::from_bytes(V2, ObjectFormat::Sha1, cap).unwrap_err(),
        Error::AllocationLimit
    );
    for end in [0, 4, 11, 24, V2.len() - 1] {
        assert!(TrackedPaths::from_bytes(&V2[..end], ObjectFormat::Sha1, limits()).is_err());
    }
}

#[test]
fn compressed_paths_are_bounded_before_decoder_expansion() {
    let bytes = include_bytes!("fixtures/git-index/sha1-compressed-v4.index");
    let mut cap = limits();
    cap.input_bytes = 32768;
    cap.entries = 128;
    cap.allocation_bytes = 16384;
    assert!(bytes.len() < cap.allocation_bytes);
    assert_eq!(
        TrackedPaths::from_bytes(bytes, ObjectFormat::Sha1, cap).unwrap_err(),
        Error::AllocationLimit
    );
    cap.allocation_bytes = 65536;
    assert_eq!(
        TrackedPaths::from_bytes(bytes, ObjectFormat::Sha1, cap)
            .unwrap()
            .len(),
        128
    );
}

#[test]
fn unsafe_index_paths_are_rejected_without_returning_a_partial_index() {
    let mut bytes = V2.to_vec();
    let needle = b"src/main.rs";
    let at = bytes
        .windows(needle.len())
        .position(|p| p == needle)
        .unwrap();
    // Preserve byte ordering so path validation, rather than index ordering,
    // is the first rejecting boundary.
    bytes[at..at + needle.len()].copy_from_slice(b"src/../file");
    let end = bytes.len();
    bytes[end - 20..].fill(0);
    assert_eq!(
        TrackedPaths::from_bytes(&bytes, ObjectFormat::Sha1, limits()).unwrap_err(),
        Error::InvalidPath
    );
}

#[test]
fn precedence_is_global_then_repository_then_shallow_to_deep_worktree() {
    let inputs = [
        worktree("src", b"!keep.log\n"),
        worktree("", b"*.log\n!root.log\n"),
        IgnoreInput {
            origin: IgnoreOrigin::Repository,
            bytes: b"!*.log\n",
        },
        IgnoreInput {
            origin: IgnoreOrigin::Global,
            bytes: b"*.log\n*.tmp\n",
        },
    ];
    let rules = IgnoreRules::from_inputs(&inputs, ignore_limits(), false).unwrap();
    assert!(rules.ignored("src/keep.log", false).unwrap().is_none());
    assert!(rules.ignored("root.log", false).unwrap().is_none());
    assert_eq!(
        rules
            .ignored("src/other.log", false)
            .unwrap()
            .unwrap()
            .origin,
        IgnoreOrigin::Worktree {
            directory: "".into()
        }
    );
    assert_eq!(
        rules.ignored("root.tmp", false).unwrap().unwrap().origin,
        IgnoreOrigin::Global
    );
    let reverse: Vec<_> = inputs.into_iter().rev().collect();
    let reordered = IgnoreRules::from_inputs(&reverse, ignore_limits(), false).unwrap();
    assert_eq!(rules.input_sha256(), reordered.input_sha256());
}

#[test]
fn an_ignored_parent_cannot_be_reopened_by_a_child_negation() {
    let rules = IgnoreRules::from_inputs(
        &[
            worktree("", b"ignored/\n!ignored/keep.txt\n"),
            worktree("ignored", b"!keep.txt\n"),
        ],
        ignore_limits(),
        false,
    )
    .unwrap();
    let m = rules.ignored("ignored/keep.txt", false).unwrap().unwrap();
    assert_eq!(
        m.origin,
        IgnoreOrigin::Worktree {
            directory: "".into()
        }
    );
    assert_eq!(m.line, 1);
    let index = TrackedPaths::from_bytes(V2, ObjectFormat::Sha1, limits()).unwrap();
    assert!(index.has_descendants("ignored", false));
    assert_eq!(
        index.kind("ignored/tracked.txt", false),
        Some(TrackedKind::File)
    );
    assert!(rules
        .ignored("ignored/untracked.txt", false)
        .unwrap()
        .is_some());
}

#[test]
fn reinclude_directory_before_reinclude_child_and_preserve_hidden_files() {
    let rules = IgnoreRules::from_inputs(
        &[worktree(
            "",
            b"out/*\n!out/keep/\nout/keep/*\n!out/keep/file\n",
        )],
        ignore_limits(),
        false,
    )
    .unwrap();
    assert!(rules.ignored("out/drop/file", false).unwrap().is_some());
    assert!(rules.ignored("out/keep/file", false).unwrap().is_none());
    assert!(rules.ignored("out/keep/other", false).unwrap().is_some());
    assert!(rules
        .ignored(".github/workflows/check.yml", false)
        .unwrap()
        .is_none());
}

#[test]
fn git_wildcards_escaped_prefixes_and_directory_only_patterns() {
    let rules = IgnoreRules::from_inputs(
        &[worktree(
            "",
            b"**/cache/*.tmp\n\\#literal\n\\!literal\nfolder/\n/root-only\n*.log\n!keep.log\n",
        )],
        ignore_limits(),
        false,
    )
    .unwrap();
    for path in [
        "cache/a.tmp",
        "a/b/cache/a.tmp",
        "#literal",
        "!literal",
        "folder/child",
        "root-only",
        "a/no.log",
    ] {
        assert!(rules.ignored(path, false).unwrap().is_some(), "{path}");
    }
    for path in [
        "folder",
        "a/root-only",
        "keep.log",
        "src/keep.log",
        "cache/a.txt",
    ] {
        assert!(rules.ignored(path, false).unwrap().is_none(), "{path}");
    }
    assert!(rules.ignored("folder", true).unwrap().is_some());
}

#[test]
fn case_mode_and_nested_source_scope_are_explicit() {
    let inputs = [worktree("src", b"*.LOG\n")];
    let sensitive = IgnoreRules::from_inputs(&inputs, ignore_limits(), false).unwrap();
    let folded = IgnoreRules::from_inputs(&inputs, ignore_limits(), true).unwrap();
    assert!(sensitive.ignored("src/a.log", false).unwrap().is_none());
    assert!(folded.ignored("src/a.log", false).unwrap().is_some());
    assert!(folded.ignored("SRC/a.log", false).unwrap().is_some());
    assert!(folded.ignored("other/a.LOG", false).unwrap().is_none());
    assert_ne!(sensitive.input_sha256(), folded.input_sha256());
    let nested = IgnoreRules::from_inputs(
        &[worktree("a", b"*.log\n"), worktree("A/b", b"!keep.log\n")],
        ignore_limits(),
        true,
    )
    .unwrap();
    assert!(nested.ignored("a/b/keep.log", false).unwrap().is_none());
    assert!(matches!(
        IgnoreRules::from_inputs(
            &[worktree("src", b"a"), worktree("SRC", b"b")],
            ignore_limits(),
            true
        ),
        Err(Error::DuplicateSource)
    ));
}

#[test]
fn malformed_or_excessive_ignore_inputs_fail_explicitly() {
    let inputs = [worktree("", b"a\nb\n")];
    let mut cap = ignore_limits();
    cap.patterns = 1;
    assert!(matches!(
        IgnoreRules::from_inputs(&inputs, cap, false),
        Err(Error::PatternLimit)
    ));
    cap = ignore_limits();
    cap.parse.input_bytes = 1;
    assert!(matches!(
        IgnoreRules::from_inputs(&inputs, cap, false),
        Err(Error::InputLimit)
    ));
    assert!(matches!(
        IgnoreRules::from_inputs(
            &[worktree("", b"a"), worktree("", b"b")],
            ignore_limits(),
            false
        ),
        Err(Error::DuplicateSource)
    ));
    assert!(matches!(
        IgnoreRules::from_inputs(&[worktree("../escape", b"a")], ignore_limits(), false),
        Err(Error::InvalidPath)
    ));
    let rules = IgnoreRules::from_inputs(&[], ignore_limits(), false).unwrap();
    for path in [
        "../escape",
        "/absolute",
        "a//b",
        "a\\b",
        "C:/file",
        "a/./b",
        "",
    ] {
        assert_eq!(rules.ignored(path, false), Err(Error::InvalidPath));
    }
}

#[test]
fn ignore_decisions_match_recorded_git_check_ignore_results() {
    let oracle: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/git-index/ignore-oracle.json")).unwrap();
    let inputs: Vec<_> = oracle["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|source| {
            let origin = match source["kind"].as_str().unwrap() {
                "global" => IgnoreOrigin::Global,
                "repository" => IgnoreOrigin::Repository,
                "worktree" => IgnoreOrigin::Worktree {
                    directory: source["directory"].as_str().unwrap().into(),
                },
                _ => panic!("unknown fixture source"),
            };
            IgnoreInput {
                origin,
                bytes: source["patterns"].as_str().unwrap().as_bytes(),
            }
        })
        .collect();
    for ignore_case in [false, true] {
        let rules = IgnoreRules::from_inputs(&inputs, ignore_limits(), ignore_case).unwrap();
        for case in oracle["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|case| case["ignore_case"] == ignore_case)
        {
            let path = case["path"].as_str().unwrap();
            let actual = rules
                .ignored(path, case["is_directory"].as_bool().unwrap())
                .unwrap()
                .is_some();
            assert_eq!(
                actual,
                case["ignored"].as_bool().unwrap(),
                "{path}, ignore_case={ignore_case}"
            );
        }
    }
}
