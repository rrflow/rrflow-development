#!/usr/bin/env python3
"""Regressions for the architecture checker: each rule fails on the violation it names."""

import copy
import unittest

from check_architecture import Member, check_crates, check_module_names, load_manifest, read_members

RULES = {
    "component_prefix": "rrflow-",
    "daemon_crates": ["rrd-engine"],
    "banned_module_names": ["utils", "common", "helpers", "misc"],
}
TIERS = [{"id": 0, "folders": ["crates/foundation"]}, {"id": 1, "folders": ["crates/storage"]}, {"id": 4, "folders": ["crates/engine"]}]


def manifest(*crates: dict) -> dict:
    return {"rules": RULES, "tier": TIERS, "crate": [copy.deepcopy(crate) for crate in crates]}


def member(directory: str, normal=(), build=(), dev=()) -> Member:
    return Member(directory, {"normal": set(normal), "build": set(build), "dev": set(dev)})


KERNEL = {"name": "rrflow-kernel", "tier": 0, "folder": "crates/foundation", "standalone": True}
LSM = {"name": "rrflow-lsm", "tier": 1, "folder": "crates/storage", "standalone": True}
KV = {"name": "rrflow-kv", "tier": 1, "folder": "crates/storage"}
ENGINE = {"name": "rrd-engine", "tier": 4, "folder": "crates/engine"}
LAB = {"name": "rrflow-lab", "tier": "evaluation", "folder": "crates/evaluation/lab"}
MEMBERS = {
    "rrflow-kernel": member("crates/foundation/rrflow-kernel"),
    "rrflow-lsm": member("crates/storage/rrflow-lsm", normal=["rrflow-kernel"]),
    "rrflow-kv": member("crates/storage/rrflow-kv", normal=["rrflow-lsm", "rrflow-kernel"]),
    "rrd-engine": member("crates/engine/rrd-engine", normal=["rrflow-kv"], dev=["rrflow-lab"]),
    "rrflow-lab": member("crates/evaluation/lab/rrflow-lab", normal=["rrd-engine"]),
}
ALL = (KERNEL, LSM, KV, ENGINE, LAB)


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
        self.assert_one(self.problems(crates=(KERNEL, LSM, KV, ENGINE)), "rrflow-lab: a workspace member")
        members = {name: value for name, value in MEMBERS.items() if name != "rrflow-lab"}
        members["rrd-engine"] = member("crates/engine/rrd-engine", normal=["rrflow-kv"])
        self.assert_one(self.problems(members=members), "rrflow-lab: listed in")

    def test_a_crate_outside_its_recorded_folder_fails(self):
        moved = member("crates/compute/rrflow-kv", normal=["rrflow-lsm", "rrflow-kernel"])
        self.assert_one(self.problems(rrflow_kv=moved), "is at crates/compute/rrflow-kv")

    def test_a_dependency_on_a_higher_tier_fails(self):
        upward = member("crates/storage/rrflow-kv", normal=["rrflow-lsm", "rrd-engine"])
        self.assert_one(self.problems(rrflow_kv=upward), "higher tier 4")

    def test_a_build_dependency_obeys_the_tiers_and_a_dev_dependency_does_not(self):
        build = member("crates/storage/rrflow-kv", normal=["rrflow-lsm"], build=["rrd-engine"])
        self.assert_one(self.problems(rrflow_kv=build), "higher tier 4")
        dev = member("crates/storage/rrflow-kv", normal=["rrflow-lsm"], dev=["rrd-engine"])
        self.assertEqual(self.problems(rrflow_kv=dev), [])

    def test_a_same_tier_dependency_must_be_listed_above(self):
        backwards = member("crates/storage/rrflow-lsm", normal=["rrflow-kernel", "rrflow-kv"])
        problems = self.problems(rrflow_lsm=backwards)
        self.assertTrue(any("lists below it" in problem for problem in problems), problems)

    def test_a_product_crate_uses_an_evaluation_crate_only_in_tests(self):
        engine = member("crates/engine/rrd-engine", normal=["rrflow-kv", "rrflow-lab"])
        self.assert_one(self.problems(rrd_engine=engine), "only in its tests")

    def test_a_name_must_conform_or_record_its_target(self):
        store = {"name": "rrd-store", "tier": 1, "folder": "crates/storage"}
        members = {**MEMBERS, "rrd-store": member("crates/storage/rrd-store")}
        self.assert_one(self.problems(crates=(*ALL, store), members=members), "rename it or record `target_name`")
        self.assertEqual(self.problems(crates=(*ALL, {**store, "target_name": "rrflow-db"}), members=members), [])
        self.assert_one(self.problems(crates=(*ALL, {**store, "target_name": "rrd-db"}), members=members), "target name rrd-db")
        self.assert_one(self.problems(crates=(*ALL, {**store, "target_name": "rrflow-kv"}), members=members), "already a workspace member")

    def test_a_folder_must_belong_to_the_tier_or_record_its_target(self):
        kv = {**KV, "folder": "crates/elsewhere"}
        members = {**MEMBERS, "rrflow-kv": member("crates/elsewhere/rrflow-kv", normal=["rrflow-lsm"])}
        self.assert_one(self.problems(crates=(KERNEL, LSM, kv, ENGINE, LAB), members=members), "move it or record `target_folder`")
        recorded = {**kv, "target_folder": "crates/storage"}
        self.assertEqual(self.problems(crates=(KERNEL, LSM, recorded, ENGINE, LAB), members=members), [])
        wrong = {**kv, "target_folder": "crates/engine"}
        self.assert_one(self.problems(crates=(KERNEL, LSM, wrong, ENGINE, LAB), members=members), "target folder crates/engine")

    def test_a_reached_target_must_be_removed(self):
        self.assert_one(self.problems(crates=(KERNEL, LSM, {**KV, "target_folder": "crates/storage"}, ENGINE, LAB)), "remove `target_folder`")
        self.assert_one(self.problems(crates=(KERNEL, LSM, {**KV, "target_name": "rrflow-kv"}, ENGINE, LAB)), "remove `target_name`")

    def test_a_standalone_crate_depends_only_on_standalone_crates_in_every_kind(self):
        lsm = member("crates/storage/rrflow-lsm", normal=["rrflow-kernel"], dev=["rrflow-kv"])
        self.assert_one(self.problems(rrflow_lsm=lsm), "dev dependency on rrflow-kv")
        excepted = {**LSM, "standalone_exceptions": ["dev:rrflow-kv"]}
        self.assertEqual(self.problems(crates=(KERNEL, excepted, KV, ENGINE, LAB), rrflow_lsm=lsm), [])
        self.assert_one(self.problems(crates=(KERNEL, excepted, KV, ENGINE, LAB)), "no longer applies")

    def test_nothing_depends_on_a_retiring_crate(self):
        old = {"name": "rrd-maintenance", "tier": 1, "folder": "crates/operations", "retire": True}
        members = {**MEMBERS, "rrd-maintenance": member("crates/operations/rrd-maintenance")}
        self.assertEqual(self.problems(crates=(*ALL, old), members=members), [])
        members["rrd-engine"] = member("crates/engine/rrd-engine", normal=["rrflow-kv", "rrd-maintenance"], dev=["rrflow-lab"])
        self.assert_one(self.problems(crates=(*ALL, old), members=members), "marked retire")


class ModuleNameTests(unittest.TestCase):
    def test_banned_file_and_folder_modules_fail_and_harness_trees_do_not(self):
        banned = RULES["banned_module_names"]
        problems = check_module_names([
            "crates/storage/rrflow-kv/src/utils.rs",
            "crates/storage/rrflow-kv/src/common/keys.rs",
            "crates/storage/rrflow-kv/src/keys.rs",
            "crates/storage/rrflow-kv/src/common_keys.rs",
            "crates/storage/rrflow-kv/tests/common/mod.rs",
            "crates/storage/rrflow-kv/src/read/tests/helpers.rs",
        ], banned)
        self.assertEqual(len(problems), 2, problems)
        self.assertIn("`common`", problems[0])
        self.assertIn("`utils`", problems[1])


class RepositoryTests(unittest.TestCase):
    def test_the_recorded_manifest_matches_this_workspace(self):
        self.assertEqual(check_crates(load_manifest(), read_members()), [])


if __name__ == "__main__":
    unittest.main()
