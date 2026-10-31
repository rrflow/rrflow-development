#!/usr/bin/env python3
"""Identity and reporting tests; no CI acceptance or native qualification."""
import copy
import json
import unittest

from inspect_candidate_runs import REPOSITORY, collect, compare, duration, summarize


def fixture(count=1):
    head, source, tree = "1" * 40, "2" * 40, "3" * 40
    run = {"id": 7, "status": "in_progress", "conclusion": None,
           "path": ".github/workflows/ci.yml", "repository": {"full_name": REPOSITORY},
           "event": "pull_request", "head_sha": head, "run_attempt": 2,
           "referenced_workflows": [{"sha": source, "path": f"{REPOSITORY}/.github/workflows/ci-reusable.yml@{source}"}]}
    step = {"name": "storage recovery", "status": "completed", "conclusion": "failure",
            "started_at": "2026-10-30T03:00:00Z", "completed_at": "2026-10-30T03:05:00Z"}
    jobs = [{"id": 100 + i, "run_id": 7, "run_attempt": 2, "head_sha": head,
             "name": f"windows-{i}", "status": "completed", "conclusion": "failure",
             "steps": [copy.deepcopy(step)]} for i in range(count)]
    return run, jobs, {"sha": source, "tree": {"sha": tree}}


def requester(run, jobs, commit):
    def request(endpoint):
        if endpoint == "actions/runs/7":
            value = run
        elif endpoint.startswith("git/commits/"):
            value = commit
        else:
            page = int(endpoint[-1])
            value = {"total_count": len(jobs), "jobs": jobs[(page - 1) * 100:page * 100]}
        return value, json.dumps(value).encode()
    return request


class InspectionTests(unittest.TestCase):
    def test_failure_and_running_are_reported_with_identity(self):
        run, jobs, commit = fixture(2)
        jobs[1]["status"], jobs[1]["conclusion"] = "in_progress", None
        jobs[1]["steps"][0].update(status="in_progress", conclusion=None, completed_at=None)
        snapshot = collect(7, requester(run, jobs, commit))
        report = summarize(snapshot)
        self.assertEqual(report["tested_source"], "2" * 40)
        self.assertEqual(report["tested_tree"], "3" * 40)
        self.assertEqual(report["attention"][0]["steps"][0]["seconds"], 300)
        self.assertIsNone(report["attention"][1]["steps"][0]["seconds"])

    def test_comparison_reports_pass_to_failure_without_assuming_timeout(self):
        run, jobs, commit = fixture()
        after = collect(7, requester(run, jobs, commit))
        before = copy.deepcopy(after)
        before["jobs"][0]["steps"][0].update(conclusion="success", completed_at="2026-10-30T03:02:11Z")
        row = compare(before, after)[0]
        self.assertEqual((row["first_conclusion"], row["first_seconds"], row["second_conclusion"], row["second_seconds"]),
                         ("success", 131, "failure", 300))
        before["jobs"].append(copy.deepcopy(before["jobs"][0]))
        self.assertEqual(compare(before, after), [])

    def test_pagination_empty_inventory_and_invalid_identity(self):
        run, jobs, commit = fixture(101)
        self.assertEqual(len(collect(7, requester(run, jobs, commit))["jobs"]), 101)
        self.assertEqual(collect(7, requester(run, [], commit))["jobs"], [])
        for change in ("duplicate", "attempt", "head", "source", "commit", "repository"):
            r, j, c = copy.deepcopy((run, jobs, commit))
            if change == "duplicate": j[-1] = copy.deepcopy(j[0])
            if change == "attempt": j[-1]["run_attempt"] = 1
            if change == "head": j[-1]["head_sha"] = "9" * 40
            if change == "source": r["referenced_workflows"] = []
            if change == "commit": c["sha"] = "9" * 40
            if change == "repository": r["repository"]["full_name"] = "rrflow/rrflow"
            with self.subTest(change=change), self.assertRaises(RuntimeError):
                collect(7, requester(r, j, c))

    def test_changed_attempt_or_incomplete_pages_fail(self):
        run, jobs, commit = fixture(101)
        base = requester(run, jobs, commit)
        calls = 0
        def changed(endpoint):
            nonlocal calls
            if endpoint == "actions/runs/7":
                calls += 1
                if calls == 2:
                    other = copy.deepcopy(run)
                    other["run_attempt"] = 3
                    return other, b""
            return base(endpoint)
        with self.assertRaises(RuntimeError): collect(7, changed)
        def missing(endpoint):
            if endpoint.endswith("page=2"): return {"total_count": 101, "jobs": []}, b""
            return base(endpoint)
        with self.assertRaises(RuntimeError): collect(7, missing)

    def test_bad_or_missing_timestamps_remain_unknown(self):
        self.assertIsNone(duration("bad", "bad"))
        self.assertIsNone(duration(None, None))
        self.assertIsNone(duration("2026-10-30T03:00:00Z", "2026-10-30T02:00:00Z"))

    def test_completed_job_without_conclusion_is_not_silently_green(self):
        run, jobs, commit = fixture()
        jobs[0]["conclusion"] = None
        jobs[0]["steps"][0]["conclusion"] = None
        report = summarize(collect(7, requester(run, jobs, commit)))
        self.assertIsNone(report["attention"][0]["conclusion"])
        self.assertIsNone(report["attention"][0]["steps"][0]["conclusion"])

    def test_pending_steps_do_not_hide_active_or_unknown_states(self):
        run, jobs, commit = fixture()
        jobs[0].update(status="in_progress", conclusion=None)
        jobs[0]["steps"] = [
            {"name": "pending", "status": "pending", "conclusion": None},
            {"name": "queued", "status": "queued", "conclusion": None},
            {"name": "active", "status": "in_progress", "conclusion": None},
            {"name": "failed", "status": "completed", "conclusion": "failure"},
            {"name": "missing", "status": "completed", "conclusion": None},
            {"name": "unknown", "status": "future_status", "conclusion": None},
        ]
        report = summarize(collect(7, requester(run, jobs, commit)))
        self.assertEqual([s["name"] for s in report["attention"][0]["steps"]],
                         ["active", "failed", "missing", "unknown"])


if __name__ == "__main__":
    unittest.main()
