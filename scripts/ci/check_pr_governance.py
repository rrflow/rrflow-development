#!/usr/bin/env python3
"""Fail a candidate pull request that breaks the repository's work-in-progress limits.

The limits are machine-enforced because prose did not hold: by 2026-11-09 the
repository had 27 open pull requests, 20 of them drafts, most 130 to 420
commits behind main, two of them merged with no file changes. The numbers come
from measured practice, not taste:

- Google's reviewed changes have a median of about 24 lines and are approved in
  under four hours (Sadowski et al., "Modern Code Review: A Case Study at
  Google", ICSE-SEIP 2018). The Cisco/SmartBear review study of 2,500 reviews
  found review effectiveness falls off past 200 to 400 lines.
- DORA's trunk-based development capability sets the bar at three or fewer
  active branches and daily merges to trunk.
- Open-source projects that admit AI agents cap them at about three open pull
  requests per contributor; teams running parallel agents size concurrency by
  open pull requests awaiting review, not by agents available.

Checks, each with a test in `test_pr_governance.py`:

- an empty diff fails (a merge with nothing in it is not a landing);
- more than `MAX_LINES` hand-written changed lines is reported as an advisory,
  never a failure. The cited studies measure human line-by-line review; a
  complete package is the owner's landing unit, generated projections are not
  reviewed line by line (GitHub collapses `linguist-generated` files), and
  size labels in practice (Kubernetes `size/*`) inform rather than block;
- a base other than `main` fails (no stacked pull requests);
- more than `MAX_OPEN` open pull requests, or more than `MAX_PER_AUTHOR` by one
  author, fails the newest one unless it carries `integration`;
- a branch whose merge base with main is older than `MAX_AGE_DAYS` fails.

The script reads facts from the GitHub event and the repository; `evaluate`
is pure so the rules are tested without GitHub.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import urllib.request
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path

MAX_LINES = 400
MAX_OPEN = 5
MAX_PER_AUTHOR = 3
MAX_AGE_DAYS = 3
INTEGRATION_LABEL = "integration"
# Paths produced by a generator, a lockfile or a golden refresh. They count as
# changed files but not as hand-written lines.
GENERATED_MARKERS = ("/generated/", "eval/schemas/", "openapi", "signal-catalogue", "signal_catalogue")
GENERATED_SUFFIXES = (".lock", "pnpm-lock.yaml", "package-lock.json", "go.sum")
# Pull requests opened before the limits landed are the backlog being absorbed;
# they keep the empty-diff, stacking and staleness rules but not the caps.
GOVERNANCE_SINCE = "2026-11-10T00:00:00Z"


@dataclass
class Facts:
    number: int
    author: str
    base_ref: str
    labels: list[str]
    changed_files: int
    changed_lines: int
    merge_base_age_days: float
    generated_lines: int = 0
    created_at: str = GOVERNANCE_SINCE
    open_pull_requests: list[dict] = field(default_factory=list)  # [{number, author, created_at}]


def evaluate(facts: Facts) -> list[str]:
    """Every rule the pull request breaks, in a fixed order; empty means admitted."""
    violations: list[str] = []
    legacy = facts.created_at < GOVERNANCE_SINCE
    exempt = INTEGRATION_LABEL in facts.labels or legacy
    if facts.changed_files == 0:
        violations.append("the pull request changes no files; a merge with nothing in it is not a landing")
    if facts.base_ref != "main":
        violations.append(f"the base is {facts.base_ref}; pull requests open against main only, no stacks")
    others = [pr for pr in facts.open_pull_requests
              if pr["number"] != facts.number and pr.get("created_at", GOVERNANCE_SINCE) >= GOVERNANCE_SINCE]
    if len(others) + 1 > MAX_OPEN and not exempt:
        violations.append(
            f"{len(others) + 1} pull requests would be open; the limit is {MAX_OPEN}. Land or close one before opening another"
        )
    same_author = [pr for pr in others if pr["author"] == facts.author]
    if len(same_author) + 1 > MAX_PER_AUTHOR and not exempt:
        violations.append(
            f"{facts.author} would have {len(same_author) + 1} open pull requests; the limit is {MAX_PER_AUTHOR}"
        )
    if facts.merge_base_age_days > MAX_AGE_DAYS:
        violations.append(
            f"the branch's merge base with main is {facts.merge_base_age_days:.1f} days old; the limit is "
            f"{MAX_AGE_DAYS}. Merge main into the branch first"
        )
    return violations


def is_generated(path: str) -> bool:
    lowered = path.lower()
    return any(marker in lowered for marker in GENERATED_MARKERS) or lowered.endswith(GENERATED_SUFFIXES)


def generated_line_count(numstat: str) -> int:
    """Lines changed in generated paths, from `git diff --numstat` output."""
    total = 0
    for line in numstat.splitlines():
        parts = line.split(chr(9))
        if len(parts) == 3 and parts[0].isdigit() and parts[1].isdigit() and is_generated(parts[2]):
            total += int(parts[0]) + int(parts[1])
    return total


def advisories(facts: Facts) -> list[str]:
    """Facts a reviewer should see that never block a landing."""
    hand_written = max(facts.changed_lines - facts.generated_lines, 0)
    if hand_written > MAX_LINES:
        return [f"large package: {hand_written} hand-written changed lines ({facts.generated_lines} generated "
                f"lines excluded) exceed {MAX_LINES}; review from the pull request's change brief and "
                f"traceability map"]
    return []


def github_json(url: str, token: str | None) -> object:
    request = urllib.request.Request(url, headers={"Accept": "application/vnd.github+json", "User-Agent": "rrflow-ci"})
    if token:
        request.add_header("Authorization", f"Bearer {token}")
    with urllib.request.urlopen(request, timeout=60) as response:
        return json.load(response)


def merge_base_age_days(base: str, head: str, root: Path) -> float:
    merge_base = subprocess.run(["git", "merge-base", base, head], cwd=root, check=True,
                                capture_output=True, text=True).stdout.strip()
    committed = subprocess.run(["git", "show", "-s", "--format=%ct", merge_base], cwd=root, check=True,
                               capture_output=True, text=True).stdout.strip()
    return (datetime.now(timezone.utc).timestamp() - int(committed)) / 86400


def diff_numstat(base: str, head: str, root: Path) -> str:
    return subprocess.run(["git", "diff", "--numstat", f"{base}...{head}"], cwd=root, check=True,
                          capture_output=True, text=True).stdout


def facts_from_github(event: dict, repository: str, token: str | None, root: Path) -> Facts:
    # The event payload is frozen when the run is first triggered and is reused
    # by every rerun, so a label the owner applies afterwards never appears in
    # it. The live pull request is the authority for labels, counts and refs.
    api = f"https://api.github.com/repos/{repository}"
    pull = github_json(f"{api}/pulls/{event['pull_request']['number']}", token)
    open_prs = github_json(f"{api}/pulls?state=open&per_page=100", token)
    return Facts(
        number=pull["number"],
        author=pull["user"]["login"],
        base_ref=pull["base"]["ref"],
        labels=[label["name"] for label in pull.get("labels", [])],
        changed_files=int(pull.get("changed_files", 0)),
        changed_lines=int(pull.get("additions", 0)) + int(pull.get("deletions", 0)),
        merge_base_age_days=merge_base_age_days(pull["base"]["sha"], pull["head"]["sha"], root),
        generated_lines=generated_line_count(diff_numstat(pull["base"]["sha"], pull["head"]["sha"], root)),
        created_at=pull["created_at"],
        open_pull_requests=[{"number": pr["number"], "author": pr["user"]["login"], "created_at": pr["created_at"]}
                            for pr in open_prs],
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--event", type=Path, default=Path(os.environ.get("GITHUB_EVENT_PATH", "")))
    parser.add_argument("--repository", default=os.environ.get("GITHUB_REPOSITORY", ""))
    args = parser.parse_args(argv)
    event_name = os.environ.get("GITHUB_EVENT_NAME", "")
    if event_name != "pull_request":
        print(f"pr-governance: {event_name or 'no'} event, nothing to check")
        return 0
    event = json.loads(args.event.read_text(encoding="utf-8"))
    facts = facts_from_github(event, args.repository, os.environ.get("GITHUB_TOKEN"), Path.cwd())
    violations = evaluate(facts)
    for advisory in advisories(facts):
        print(f"::warning::pr-governance: {advisory}")
    for violation in violations:
        print(f"::error::pr-governance: {violation}")
    if violations:
        return 1
    print(f"pr-governance: OK: #{facts.number} by {facts.author}, {facts.changed_lines} lines, "
          f"{len(facts.open_pull_requests)} open, merge base {facts.merge_base_age_days:.1f} days old")
    return 0


if __name__ == "__main__":
    sys.exit(main())
