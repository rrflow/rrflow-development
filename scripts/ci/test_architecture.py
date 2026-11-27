#!/usr/bin/env python3
"""Regressions for the architecture checker: each rule fails on the violation it names."""

import copy
import unittest

from check_architecture import Member, check_core, check_crates, check_module_names, load_manifest, read_members

RULES = {"banned_names": ["utils", "common", "helpers", "misc", "kernel", "foundation", "shared", "base"]}
GROUPS = [
    {"folder": "rrflow", "prefix": "rrflow-"},
    {"folder": "rrd", "prefix": "rrd-"},
    {"folder": "lab", "prefix": "rrflow-lab-", "outside": True},
]


def manifest(*crates: dict) -> dict:
    return {"rules": RULES, "group": GROUPS, "crate": [copy.deepcopy(crate) for crate in crates]}


def member(directory: str, normal=(), build=(), dev=()) -> Member:
    return Member(directory, {"normal": set(normal), "build": set(build), "dev": set(dev)})


CORE = {"name": "rrflow-core", "uses": [], "standalone": True}
LSM = {"name": "rrflow-lsm", "uses": ["rrflow-core"], "standalone": True}
KV = {"name": "rrflow-kv", "uses": ["rrflow-core", "rrflow-lsm"]}
ENGINE = {"name": "rrd-engine", "uses": ["rrflow-kv"]}
RUNNER = {"name": "rrflow-lab-runner", "uses": ["rrd-engine"]}
MEMBERS = {
    "rrflow-core": member("rrflow/core"),
    "rrflow-lsm": member("rrflow/lsm", normal=["rrflow-core"]),
    "rrflow-kv": member("rrflow/kv", normal=["rrflow-lsm", "rrflow-core"]),
    "rrd-engine": member("rrd/engine", normal=["rrflow-kv"], dev=["rrflow-lab-runner"]),
    "rrflow-lab-runner": member("lab/runner", normal=["rrd-engine"]),
}
ALL = (CORE, LSM, KV, ENGINE, RUNNER)


class CrateRuleTests(unittest.TestCase):
    def problems(self, crates=ALL, members=None, **changes) -> list[str]:
        members = copy.deepcopy(MEMBERS if members is None else members)
        for name, replacement in changes.items():
            members[name.replace("_", "-")] = replacement
        return check_crates(manifest(*crates), members)

    def assert_one(self, problems: list[str], fragment: str) -> None:
        self.assertEqual(len(problems), 1, problems)
        self.assertIn(fragment, problems[0])

    def test_a_conforming_workspace_passes(self):
        self.assertEqual(self.problems(), [])

    def test_an_unlisted_member_and_a_stale_entry_fail(self):
        self.assert_one(self.problems(crates=(CORE, LSM, KV, ENGINE)), "rrflow-lab-runner: a workspace member")
        members = {name: value for name, value in MEMBERS.items() if name != "rrflow-lab-runner"}
        members["rrd-engine"] = member("rrd/engine", normal=["rrflow-kv"])
        self.assert_one(self.problems(members=members), "rrflow-lab-runner: listed in")

    def test_a_crate_sits_one_folder_below_a_group(self):
        nested = member("rrflow/storage/kv", normal=["rrflow-lsm", "rrflow-core"])
        self.assert_one(self.problems(rrflow_kv=nested), "one folder below")
        elsewhere = member("crates/kv", normal=["rrflow-lsm", "rrflow-core"])
        self.assert_one(self.problems(rrflow_kv=elsewhere), "one folder below")

    def test_a_package_is_named_for_its_group_and_folder(self):
        moved = member("rrflow/keyvalue", normal=["rrflow-lsm", "rrflow-core"])
        self.assert_one(self.problems(rrflow_kv=moved), "its package name is rrflow-keyvalue")
        daemon = member("rrd/kv", normal=["rrflow-lsm", "rrflow-core"])
        self.assert_one(self.problems(rrflow_kv=daemon), "its package name is rrd-kv")

    def test_a_crate_folder_cannot_take_a_banned_name(self):
        crates = (CORE, LSM, {"name": "rrflow-kernel", "uses": ["rrflow-core", "rrflow-lsm"]})
        members = {name: MEMBERS[name] for name in ("rrflow-core", "rrflow-lsm")}
        members["rrflow-kernel"] = member("rrflow/kernel", normal=["rrflow-lsm", "rrflow-core"])
        self.assert_one(check_crates(manifest(*crates), members), "a folder named `kernel`")

    def test_dependencies_are_exactly_the_recorded_uses(self):
        extra = member("rrflow/lsm", normal=["rrflow-core", "rrflow-kv"])
        problems = self.problems(rrflow_lsm=extra)
        self.assertTrue(any("which its `uses` does not allow" in problem for problem in problems), problems)
        fewer = member("rrflow/kv", normal=["rrflow-core"])
        self.assert_one(self.problems(rrflow_kv=fewer), "which it does not depend on")

    def test_a_build_dependency_counts_and_a_dev_dependency_does_not(self):
        build = member("rrflow/kv", normal=["rrflow-lsm", "rrflow-core"], build=["rrd-engine"])
        self.assert_one(self.problems(rrflow_kv=build), "depends on rrd-engine")
        dev = member("rrflow/kv", normal=["rrflow-lsm", "rrflow-core"], dev=["rrd-engine"])
        self.assertEqual(self.problems(rrflow_kv=dev), [])

    def test_a_used_crate_must_be_listed_above(self):
        self.assert_one(self.problems(crates=(CORE, KV, LSM, ENGINE, RUNNER)), "lists below it")

    def test_a_product_crate_uses_a_lab_crate_only_in_tests(self):
        engine = member("rrd/engine", normal=["rrflow-kv", "rrflow-lab-runner"])
        crates = (CORE, LSM, KV, {"name": "rrflow-lab-runner", "uses": []}, {"name": "rrd-engine", "uses": ["rrflow-kv", "rrflow-lab-runner"]})
        members = {**MEMBERS, "rrd-engine": engine, "rrflow-lab-runner": member("lab/runner")}
        self.assert_one(check_crates(manifest(*crates), members), "only in its tests")

    def test_a_pending_dependency_that_is_gone_must_be_removed(self):
        kept = {**KV, "pending": ["rrflow-lsm"]}
        self.assertEqual(self.problems(crates=(CORE, LSM, kept, ENGINE, RUNNER)), [])
        gone = {"name": "rrflow-kv", "uses": ["rrflow-core"], "pending": ["rrflow-lsm"]}
        cut = member("rrflow/kv", normal=["rrflow-core"])
        self.assert_one(self.problems(crates=(CORE, LSM, gone, ENGINE, RUNNER), rrflow_kv=cut), "remove it from `pending`")

    def test_a_standalone_crate_depends_only_on_standalone_crates_in_every_kind(self):
        lsm = member("rrflow/lsm", normal=["rrflow-core"], dev=["rrflow-kv"])
        self.assert_one(self.problems(rrflow_lsm=lsm), "dev dependency on rrflow-kv")
        excepted = {**LSM, "standalone_exceptions": ["dev:rrflow-kv"]}
        self.assertEqual(self.problems(crates=(CORE, excepted, KV, ENGINE, RUNNER), rrflow_lsm=lsm), [])
        self.assert_one(self.problems(crates=(CORE, excepted, KV, ENGINE, RUNNER)), "no longer applies")


class ModuleNameTests(unittest.TestCase):
    def test_banned_file_and_folder_modules_fail_and_harness_trees_do_not(self):
        problems = check_module_names([
            "rrflow/kv/src/utils.rs",
            "rrflow/kv/src/common/keys.rs",
            "rrflow/kv/src/keys.rs",
            "rrflow/kv/src/common_keys.rs",
            "rrflow/kv/tests/common/mod.rs",
            "rrflow/kv/src/read/tests/helpers.rs",
        ], RULES["banned_names"])
        self.assertEqual(len(problems), 2, problems)
        self.assertIn("`common`", problems[0])
        self.assertIn("`utils`", problems[1])


class CoreTests(unittest.TestCase):
    CORE = {"crate": "rrflow-core", "modules": ["digest", "record"], "max_lines": 30}
    SOURCES = {"rrflow/core/src/lib.rs": 5, "rrflow/core/src/digest.rs": 10, "rrflow/core/src/record/model.rs": 15,
               "rrflow/core/tests/digest.rs": 500, "rrflow/kv/src/extra.rs": 500}

    def problems(self, core=None, **sources) -> list[str]:
        return check_core(core or self.CORE, "rrflow/core", {**self.SOURCES, **sources})

    def test_the_recorded_shape_passes(self):
        self.assertEqual(self.problems(), [])

    def test_a_new_module_fails(self):
        problems = self.problems(**{"rrflow/core/src/extra.rs": 0})
        self.assertEqual(len(problems), 1, problems)
        self.assertIn("module `extra`", problems[0])

    def test_a_removed_module_must_leave_the_list(self):
        problems = self.problems(core={**self.CORE, "modules": ["digest", "record", "trace"]})
        self.assertEqual(len(problems), 1, problems)
        self.assertIn("which is gone", problems[0])

    def test_core_does_not_grow(self):
        problems = self.problems(**{"rrflow/core/src/digest.rs": 11})
        self.assertEqual(len(problems), 1, problems)
        self.assertIn("core does not grow", problems[0])


class RepositoryTests(unittest.TestCase):
    def test_the_recorded_manifest_matches_this_workspace(self):
        self.assertEqual(check_crates(load_manifest(), read_members()), [])


if __name__ == "__main__":
    unittest.main()
