#!/usr/bin/env python3
"""Refuse to let an agent session end with work that no pull request carries.

On 2026-11-09 five worktrees held between 14 and 514 staged, uncommitted files
each, and the implementations behind two merged pull requests existed only on
a laptop. This check passes when the worktree is clean and its branch has no
commits the remote lacks, or when every change is committed and pushed. Run it
before stopping; a pull request description should name the branch it left.
"""

from __future__ import annotations

import subprocess
import sys


def git(*arguments: str) -> str:
    return subprocess.run(["git", *arguments], check=True, capture_output=True, text=True).stdout.strip()


def problems() -> list[str]:
    found: list[str] = []
    status = [line for line in git("status", "--porcelain").splitlines() if line]
    if status:
        found.append(f"{len(status)} uncommitted change(s); commit them to this branch or a *-wip-* branch")
    branch = git("rev-parse", "--abbrev-ref", "HEAD")
    if branch == "HEAD":
        found.append("detached HEAD; put the work on a named branch before stopping")
        return found
    upstream = subprocess.run(["git", "rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{upstream}"],
                              capture_output=True, text=True)
    if upstream.returncode != 0:
        found.append(f"branch {branch} has no upstream; push it (git push -u <remote> {branch})")
        return found
    ahead = int(git("rev-list", "--count", f"{upstream.stdout.strip()}..HEAD"))
    if ahead:
        found.append(f"branch {branch} has {ahead} commit(s) the remote lacks; push before stopping")
    return found


def main() -> int:
    found = problems()
    for problem in found:
        print(f"session-end: {problem}")
    if found:
        return 1
    print("session-end: OK: clean worktree, nothing unpushed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
