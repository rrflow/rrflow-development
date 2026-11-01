#!/usr/bin/env python3
"""Inspect explicit GitHub CI runs; this report is not acceptance evidence."""
from __future__ import annotations

import argparse
from datetime import datetime
import json
import re

from collect_storage_acceptance import REPOSITORY, api
from run_offline_lifecycle import require

SHA = re.compile(r"[0-9a-f]{40}\Z")
MAX_JOBS = 128
MAX_LONGEST_STEPS = 10


def identity(record, run_id):
    require(record.get("id") == run_id, "run ID differs")
    require(record.get("repository", {}).get("full_name") == REPOSITORY, "repository differs")
    require(record.get("path") == ".github/workflows/ci.yml", "workflow differs")
    require(record.get("event") in {"pull_request", "merge_group", "workflow_dispatch"}, "event differs")
    attempt, head = record.get("run_attempt"), record.get("head_sha", "")
    require(type(attempt) is int and 0 < attempt <= 9007199254740991, "invalid attempt")
    require(isinstance(head, str) and SHA.fullmatch(head), "invalid event source")
    refs = record.get("referenced_workflows")
    require(isinstance(refs, list) and len(refs) == 1, "ambiguous reusable workflow")
    source = refs[0].get("sha", "")
    require(isinstance(source, str) and SHA.fullmatch(source), "invalid tested source")
    require(refs[0].get("path") == f"{REPOSITORY}/.github/workflows/ci-reusable.yml@{source}", "workflow source differs")
    return attempt, head, source


def collect(run_id, request=api):
    require(type(run_id) is int and 0 < run_id <= 9007199254740991, "invalid run ID")
    run, _ = request(f"actions/runs/{run_id}")
    attempt, head, source = identity(run, run_id)
    jobs, expected = [], None
    for page in range(1, 3):
        response, _ = request(f"actions/runs/{run_id}/attempts/{attempt}/jobs?per_page=100&page={page}")
        count, rows = response.get("total_count"), response.get("jobs")
        require(type(count) is int and 0 <= count <= MAX_JOBS, "invalid job count")
        require(expected is None or expected == count, "job inventory changed")
        require(isinstance(rows, list) and len(rows) <= 100, "invalid jobs page")
        expected = count
        jobs.extend(rows)
        require(len(jobs) <= count, "too many jobs")
        if len(jobs) == count:
            break
    require(len(jobs) == expected, "incomplete job inventory")
    require(all(isinstance(j, dict) and type(j.get("id")) is int for j in jobs), "invalid job")
    require(len({j["id"] for j in jobs}) == len(jobs), "duplicate jobs")
    for job in jobs:
        require(job.get("run_id") == run_id and job.get("run_attempt") == attempt
                and job.get("head_sha") == head, "job identity differs")
    commit, _ = request(f"git/commits/{source}")
    require(commit.get("sha") == source, "tested commit differs")
    tree = commit.get("tree", {}).get("sha", "")
    require(isinstance(tree, str) and SHA.fullmatch(tree), "invalid tested tree")
    final, _ = request(f"actions/runs/{run_id}")
    require(identity(final, run_id) == (attempt, head, source), "run identity changed during collection")
    return {"run": final, "jobs": jobs, "tested_source": source, "tested_tree": tree}


def duration(start, end):
    if not isinstance(start, str) or not isinstance(end, str):
        return None
    try:
        seconds = (datetime.fromisoformat(end.replace("Z", "+00:00")) -
                   datetime.fromisoformat(start.replace("Z", "+00:00"))).total_seconds()
    except (ValueError, TypeError, OverflowError):
        return None
    return seconds if seconds >= 0 else None


def job_steps(job):
    steps = job.get("steps")
    return steps if isinstance(steps, list) else []


def summarize(snapshot):
    run = snapshot["run"]
    rows = []
    longest = []
    timed_step_count = 0
    for job in snapshot["jobs"]:
        interesting = []
        for position, step in enumerate(job_steps(job)):
            if not isinstance(step, dict):
                continue
            conclusion, status = step.get("conclusion"), step.get("status")
            elapsed = duration(step.get("started_at"), step.get("completed_at"))
            job_name, step_name = job.get("name"), step.get("name")
            if (status == "completed" and elapsed is not None
                    and isinstance(job_name, str) and isinstance(step_name, str)):
                timed_step_count += 1
                row = {"job": job_name, "job_url": job.get("html_url"), "name": step_name,
                       "conclusion": conclusion, "seconds": elapsed}
                longest.append((-elapsed, job_name, step_name, job["id"], position, row))
                longest.sort(key=lambda item: item[:5])
                del longest[MAX_LONGEST_STEPS:]
            if conclusion is None and status in ("pending", "queued", "waiting", "requested"):
                continue
            if conclusion not in ("success", "skipped") or status != "completed":
                interesting.append({"name": step.get("name"), "status": status,
                                    "conclusion": conclusion,
                                    "seconds": elapsed})
        if job.get("conclusion") not in ("success", "skipped") or job.get("status") != "completed" or interesting:
            rows.append({"name": job.get("name"), "status": job.get("status"),
                         "conclusion": job.get("conclusion"), "url": job.get("html_url"), "steps": interesting})
    return {"run_id": run["id"], "attempt": run["run_attempt"], "event": run.get("event"),
            "event_head": run.get("head_sha"), "event_branch": run.get("head_branch"),
            "tested_source": snapshot["tested_source"], "tested_tree": snapshot["tested_tree"],
            "status": run.get("status"), "conclusion": run.get("conclusion"),
            "url": run.get("html_url"), "job_count": len(snapshot["jobs"]), "attention": rows,
            "timed_step_count": timed_step_count,
            "longest_steps": [item[5] for item in longest]}


def compare(first, second):
    """Compare uniquely named steps; matching names do not prove equal execution."""
    def index(snapshot):
        found = {}
        for job in snapshot["jobs"]:
            for step in job_steps(job):
                if not isinstance(step, dict):
                    continue
                key = (job.get("name"), step.get("name"))
                if not all(isinstance(k, str) for k in key):
                    continue
                found.setdefault(key, []).append(step)
        return found
    left, right = index(first), index(second)
    result = []
    for key in sorted(left.keys() & right.keys()):
        if len(left[key]) != 1 or len(right[key]) != 1:
            continue
        a, b = left[key][0], right[key][0]
        if b.get("conclusion") not in (None, "success", "skipped"):
            result.append({"job": key[0], "step": key[1],
                           "first_conclusion": a.get("conclusion"),
                           "first_seconds": duration(a.get("started_at"), a.get("completed_at")),
                           "second_conclusion": b.get("conclusion"),
                           "second_seconds": duration(b.get("started_at"), b.get("completed_at"))})
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("runs", type=int, nargs="+", help="one or two explicit Actions run IDs")
    parser.add_argument("--pr", type=int, help="compare event heads with the current PR head")
    args = parser.parse_args()
    if len(args.runs) > 2 or len(set(args.runs)) != len(args.runs):
        parser.error("provide one or two distinct run IDs")
    snapshots = [collect(run) for run in args.runs]
    report = {"notice": "Informational snapshot; association, equivalence, cause and acceptance are not established.",
              "runs": [summarize(s) for s in snapshots]}
    if len(snapshots) == 2:
        report["matching_failed_steps"] = compare(*snapshots)
        report["tested_trees_match"] = snapshots[0]["tested_tree"] == snapshots[1]["tested_tree"]
    if args.pr is not None:
        require(0 < args.pr <= 9007199254740991, "invalid PR number")
        pr, _ = api(f"pulls/{args.pr}")
        head = pr.get("head", {}).get("sha", "")
        require(pr.get("number") == args.pr and isinstance(head, str) and SHA.fullmatch(head), "invalid PR record")
        report["current_pr"] = {"number": args.pr, "head": head,
                                "event_head_matches": [s["run"].get("head_sha") == head for s in snapshots]}
    print(json.dumps(report, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
