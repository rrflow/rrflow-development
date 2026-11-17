#!/usr/bin/env python3
"""Refuse to let an agent session end with work that only this machine holds.

The owner's pre-release process (AGENTS.md, 2026-11-16) is one short-lived
branch and one pull request per bounded package, merged through the queue on a
green gate. On 2026-11-09 five worktrees held between 14 and 514 staged files
each, and by 2026-11-14 there were 35 worktrees, 172 local branches and 81
archived refs: the owner could not tell which checkout held the real work. This
check passes only when nothing is uncommitted, stashed or archived, the
checkout is the only worktree, every local branch is pushed and has an open pull
request, no branch already merged into `main` is left behind, and the remote
holds no branch without an open pull request. Run it before stopping.

The open-pull-request test needs the GitHub CLI. Without it the check says so
and skips only that test.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

# GitHub creates and removes these merge-queue scratch refs itself.
HOSTED_PREFIXES = ("gh-readonly-queue/",)
# Reference namespaces that hold work outside any branch.
SIDE_NAMESPACES = ("refs/archive/", "refs/stash", "refs/wip/")
# The push target for pre-release work; `origin` is the official repository.
PUSH_REMOTE = "development"


def git(repo: Path, *arguments: str, check: bool = True) -> str:
    result = subprocess.run(["git", "-C", str(repo), *arguments], capture_output=True, text=True)
    if check and result.returncode:
        raise RuntimeError(f"git {' '.join(arguments)}: {result.stderr.strip()}")
    return result.stdout.strip()


def git_ok(repo: Path, *arguments: str) -> bool:
    return subprocess.run(["git", "-C", str(repo), *arguments], capture_output=True).returncode == 0


def problems(repo: Path, open_pull_request_heads: set[str] | None = None) -> list[str]:
    """Everything that stops the session ending.

    `open_pull_request_heads` is the set of head branch names that have an open
    pull request; `None` means it could not be read, and the open-pull-request
    tests are skipped.
    """
    found: list[str] = []
    status = [line for line in git(repo, "status", "--porcelain").splitlines() if line]
    if status:
        found.append(f"{len(status)} uncommitted change(s); commit them to the package branch")
    worktrees = [line for line in git(repo, "worktree", "list", "--porcelain").splitlines() if line.startswith("worktree ")]
    if len(worktrees) > 1:
        found.append(f"{len(worktrees)} worktrees; the repository's own checkout is the only one")
    side = [name for name in git(repo, "for-each-ref", "--format=%(refname)").splitlines()
            if name.startswith(SIDE_NAMESPACES)]
    if side:
        found.append(f"{len(side)} ref(s) hold work outside a branch: {', '.join(sorted(side)[:5])}; absorb each, then delete it")

    for name in sorted(git(repo, "for-each-ref", "--format=%(refname:short)", "refs/heads/").splitlines()):
        upstream = git(repo, "for-each-ref", "--format=%(upstream:short)", f"refs/heads/{name}")
        if not upstream:
            found.append(f"branch {name!r} has no upstream; push it (git push -u {PUSH_REMOTE} {name})")
            continue
        unpushed = int(git(repo, "rev-list", "--count", f"{upstream}..{name}"))
        if name == "main":
            if unpushed:
                found.append(f"main has {unpushed} commit(s) {upstream} lacks; work lands by pull request, not on main")
            continue
        if unpushed:
            found.append(f"branch {name!r} has {unpushed} commit(s) {upstream} lacks; push it")
        default = f"{upstream.split('/', 1)[0]}/main"
        if git_ok(repo, "rev-parse", "--verify", "-q", default) and git_ok(repo, "merge-base", "--is-ancestor", name, default):
            found.append(f"branch {name!r} is already merged into {default}; delete it")
        elif open_pull_request_heads is not None and name not in open_pull_request_heads:
            found.append(f"branch {name!r} has no open pull request; open one or finish and delete it")

    if open_pull_request_heads is not None:
        remote_heads = []
        for name in git(repo, "for-each-ref", "--format=%(refname:short)", f"refs/remotes/{PUSH_REMOTE}/").splitlines():
            short = name[len(PUSH_REMOTE) + 1:]
            if short in ("main", "HEAD") or name == PUSH_REMOTE or short.startswith(HOSTED_PREFIXES):
                continue
            if short not in open_pull_request_heads:
                remote_heads.append(short)
        if remote_heads:
            found.append(f"{PUSH_REMOTE} holds {len(remote_heads)} branch(es) with no open pull request: "
                         f"{', '.join(sorted(remote_heads)[:5])}")
    return found


def open_pull_request_heads(repo: Path) -> set[str] | None:
    try:
        slug = git(repo, "remote", "get-url", PUSH_REMOTE).removesuffix(".git").split("github.com/", 1)[1]
        result = subprocess.run(
            ["gh", "pr", "list", "--repo", slug, "--state", "open", "--limit", "200", "--json", "headRefName"],
            capture_output=True, text=True,
        )
    except (RuntimeError, IndexError, OSError):
        return None
    if result.returncode:
        return None
    return {entry["headRefName"] for entry in json.loads(result.stdout)}


def main() -> int:
    repo = Path(__file__).resolve().parents[2]
    heads = open_pull_request_heads(repo)
    if heads is None:
        print("session-end: notice: open pull requests could not be read (gh missing or not signed in); "
              "the open-pull-request tests were skipped")
    found = problems(repo, heads)
    for problem in found:
        print(f"session-end: {problem}")
    if found:
        return 1
    print("session-end: OK: clean, one worktree, every branch pushed with an open pull request, nothing stashed or archived")
    return 0


if __name__ == "__main__":
    sys.exit(main())
