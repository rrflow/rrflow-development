#!/usr/bin/env python3
"""Regressions for the proportional candidate CI lane selection."""

import tempfile
import unittest
from pathlib import Path

from select_ci_lanes import LANES, select, with_dependents, workspace_graph

METADATA = {
    "workspace_members": ["core", "lsm", "engine", "cli", "eval", "lab"],
    "packages": [
        {"id": "core", "name": "rrd-core", "dependencies": []},
        {"id": "lsm", "name": "rrd-lsm", "dependencies": [{"name": "rrd-core"}]},
        {"id": "engine", "name": "rrd-engine", "dependencies": [{"name": "rrd-core"}, {"name": "rrd-lsm"}]},
        {"id": "cli", "name": "rrflow-cli", "dependencies": [{"name": "rrd-engine"}, {"name": "serde"}]},
        {"id": "eval", "name": "rrflow-eval", "dependencies": [{"name": "rrd-core"}]},
        {"id": "lab", "name": "rrflow-lab-core", "dependencies": []},
    ],
}


class LaneSelectionTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        members = [("kernel", "rrd-core"), ("persistence", "rrd-lsm"), ("authority", "rrd-engine"),
                   ("adapters", "rrflow-cli"), ("evaluation", "rrflow-eval"), ("transport", "rrd-contract"),
                   ("evaluation/lab", "rrflow-lab-core")]
        for group, crate in members:
            manifest = self.root / "crates" / group / crate / "Cargo.toml"
            manifest.parent.mkdir(parents=True)
            manifest.write_text(f'[package]\nname = "{crate}"\nversion = "1.0.0"\n', encoding="utf-8")
        # A fuzz package nested inside a member is not itself a member.
        fuzz = self.root / "crates" / "evaluation" / "rrflow-eval" / "fuzz" / "Cargo.toml"
        fuzz.parent.mkdir(parents=True)
        fuzz.write_text('[package]\nname = "rrflow-eval-fuzz"\nversion = "0.0.0"\n', encoding="utf-8")
        listed = ", ".join(f'"crates/{group}/{crate}"' for group, crate in members)
        (self.root / "Cargo.toml").write_text(f"[workspace]\nmembers = [{listed}]\n", encoding="utf-8")

    def lanes(self, *paths, **options):
        result = select(list(paths), metadata=METADATA, root=self.root, **options)
        return {lane for lane in LANES if result[lane]}, result["packages"]

    def dependents(self, *paths):
        return select(list(paths), metadata=METADATA, root=self.root)["dependents"]

    def test_documentation_only_runs_the_documentation_lane(self):
        selected, packages = self.lanes("docs/roadmap/rrflow-1.0.md", "README.md", ".github/ISSUE_TEMPLATE/bug.yml")
        self.assertEqual(selected, {"docs"})
        self.assertEqual(packages, "")

    def test_script_edit_runs_the_script_lane(self):
        selected, _ = self.lanes("scripts/inference/replicate_weight_edit.py", "eval/README.md")
        self.assertEqual(selected, {"scripts", "docs"})

    def test_crate_edit_tests_the_crate_and_compiles_its_dependents(self):
        selected, packages = self.lanes("crates/evaluation/rrflow-eval/src/lab/mod.rs")
        self.assertEqual(selected, {"engine"})
        self.assertEqual(packages, "-p rrflow-eval")
        self.assertEqual(self.dependents("crates/evaluation/rrflow-eval/src/lab/mod.rs"), "")
        selected, packages = self.lanes("crates/authority/rrd-engine/src/engine/vector/search.rs")
        self.assertEqual(selected, {"engine"})
        self.assertEqual(packages, "-p rrd-engine")
        self.assertEqual(self.dependents("crates/authority/rrd-engine/src/engine/vector/search.rs"), "-p rrflow-cli")

    def test_root_manifest_or_lock_change_runs_the_full_matrix(self):
        # The hosted runners cannot link the whole workspace in one target
        # graph, so a dependency change takes the partitioned full matrix.
        for path in ("Cargo.toml", "Cargo.lock"):
            with self.subTest(path=path):
                selected, packages = self.lanes(path)
                self.assertEqual(selected, {"full"})
                self.assertEqual(packages, "")

    def test_sdk_edit_runs_conformance_without_an_empty_engine_package_lane(self):
        selected, packages = self.lanes("sdks/python/src/rrd_client/operation.py")
        self.assertEqual(selected, {"sdk"})
        self.assertEqual(packages, "")

    def test_transport_crate_edit_runs_conformance_too(self):
        selected, packages = self.lanes("crates/transport/rrd-contract/src/lib.rs")
        self.assertEqual(selected, {"sdk", "engine"})
        self.assertEqual(packages, "-p rrd-contract")

    def test_repository_policy_files_and_transaction_mutations_run_full_matrix(self):
        for path in (".gitignore", ".gitattributes",
                     "crates/authority/rrd-engine/src/engine/transaction.rs",
                     "crates/authority/rrd-engine/src/engine/query_transaction.rs",
                     "crates/authority/rrd-engine/src/engine/session_transactions.rs",
                     "crates/authority/rrd-engine/src/engine/endpoint_configuration.rs",
                     "crates/authority/rrd-engine/src/engine/estate_control.rs",
                     "crates/authority/rrd-engine/src/engine/attunement.rs",
                     "crates/authority/rrd-engine/src/engine/function/execution.rs",
                     "crates/authority/rrd-engine/src/engine/observation_delivery/persistence.rs",
                     "crates/authority/rrd-engine/src/engine/reasoning_budget/persistence.rs",
                     "crates/authority/rrd-engine/src/engine/tests/recovery.rs"):
            with self.subTest(path=path):
                self.assertEqual(self.lanes(path)[0], {"full"})

    def test_storage_installer_native_harness_and_workflows_run_the_full_matrix(self):
        for path in (
            "crates/persistence/rrd-lsm/src/database.rs",
            "crates/kernel/rrd-core/src/lib.rs",
            "crates/adapters/rrflow-cli/src/main.rs",
            "crates/authority/rrd-engine/src/engine/installation/admission.rs",
            "crates/authority/rrd-engine/src/engine/estate_layout.rs",
            "crates/authority/rrd-engine/src/engine/token_key/private_file/windows.rs",
            "crates/authority/rrd-engine/src/engine/attunement/windows_supervisor.rs",
            ".github/workflows/ci-reusable.yml",
            "scripts/ci/run_native_capacity.py",
            "rust-toolchain.toml",
            "connectome",
            "deploy/compose.yml",
        ):
            with self.subTest(path=path):
                selected, packages = self.lanes(path, "docs/README.md")
                self.assertEqual(selected, {"full"}, path)
                self.assertEqual(packages, "")

    def test_unknown_path_fails_closed_into_the_full_matrix(self):
        selected, _ = self.lanes("tools/new-thing.toml")
        self.assertEqual(selected, {"full"})
        result = select(["tools/new-thing.toml"], metadata=METADATA, root=self.root)
        self.assertEqual(result["unknown"], ["tools/new-thing.toml"])

    def test_a_nested_member_is_its_own_package_not_an_unknown_path(self):
        selected, packages = self.lanes("crates/evaluation/lab/rrflow-lab-core/src/lib.rs")
        self.assertEqual(selected, {"engine"})
        self.assertEqual(packages, "-p rrflow-lab-core")

    def test_a_nested_non_member_belongs_to_the_member_around_it(self):
        selected, packages = self.lanes("crates/evaluation/rrflow-eval/fuzz/fuzz_targets/manifest.rs")
        self.assertEqual(selected, {"engine"})
        self.assertEqual(packages, "-p rrflow-eval")
        self.assertNotIn("fuzz", packages)

    def test_unknown_crate_directory_fails_closed(self):
        selected, _ = self.lanes("crates/compute/not-a-crate/src/lib.rs")
        self.assertEqual(selected, {"full"})

    def test_forced_or_empty_selection_is_the_full_matrix(self):
        self.assertEqual(self.lanes()[0], {"full"})
        self.assertEqual(self.lanes("README.md", force_full=True)[0], {"full"})

    def test_full_lane_absorbs_the_light_lanes(self):
        selected, _ = self.lanes("README.md", "scripts/x.py", "crates/evaluation/rrflow-eval/src/lib.rs",
                                 "crates/persistence/rrd-lsm/src/lib.rs")
        self.assertEqual(selected, {"full"})

    def test_dependents_close_over_the_workspace_only(self):
        graph = workspace_graph(METADATA)
        self.assertEqual(with_dependents({"rrd-core"}, graph),
                         {"rrd-core", "rrd-lsm", "rrd-engine", "rrflow-cli", "rrflow-eval"})
        self.assertEqual(with_dependents({"rrflow-cli"}, graph), {"rrflow-cli"})
        self.assertNotIn("serde", graph)

    def test_windows_separators_are_normalized(self):
        selected, packages = self.lanes("crates\\evaluation\\rrflow-eval\\src\\lib.rs")
        self.assertEqual(selected, {"engine"})
        self.assertEqual(packages, "-p rrflow-eval")


if __name__ == "__main__":
    unittest.main()
