#!/usr/bin/env python3
"""Regressions for the session-end check of the pull-request-per-package process."""

import subprocess
import tempfile
import unittest
from pathlib import Path

from check_session_end import problems


def git(repo: Path, *arguments: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@example.invalid", *arguments],
        capture_output=True, text=True, check=True,
    )
    return result.stdout.strip()


class SessionEndTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        base = Path(self.temp.name)
        self.remote = base / "remote.git"
        self.repo = base / "work"
        subprocess.run(["git", "init", "-q", "--bare", "-b", "main", str(self.remote)], check=True)
        subprocess.run(["git", "init", "-q", "-b", "main", str(self.repo)], check=True)
        (self.repo / "a.txt").write_text("a\n")
        git(self.repo, "add", "-A")
        git(self.repo, "commit", "-q", "-m", "first")
        git(self.repo, "remote", "add", "development", str(self.remote))
        git(self.repo, "push", "-q", "-u", "development", "main")

    def tearDown(self):
        self.temp.cleanup()

    def package_branch(self, name="package", push=True):
        git(self.repo, "switch", "-q", "-c", name)
        (self.repo / f"{name}.txt").write_text(name)
        git(self.repo, "add", "-A")
        git(self.repo, "commit", "-q", "-m", name)
        if push:
            git(self.repo, "push", "-q", "-u", "development", name)
        return name

    def test_clean_main_passes(self):
        self.assertEqual(problems(self.repo, set()), [])

    def test_a_pushed_package_branch_with_an_open_pull_request_passes(self):
        name = self.package_branch()
        self.assertEqual(problems(self.repo, {name}), [])

    def test_uncommitted_change_fails(self):
        (self.repo / "a.txt").write_text("changed\n")
        self.assertTrue(any("uncommitted" in p for p in problems(self.repo, set())))

    def test_a_commit_on_main_fails(self):
        (self.repo / "b.txt").write_text("b\n")
        git(self.repo, "add", "-A")
        git(self.repo, "commit", "-q", "-m", "direct")
        self.assertTrue(any("not on main" in p for p in problems(self.repo, set())))

    def test_an_unpushed_package_branch_fails(self):
        git(self.repo, "branch", "feature")
        self.assertTrue(any("no upstream" in p for p in problems(self.repo, {"feature"})))

    def test_unpushed_commits_on_a_package_branch_fail(self):
        name = self.package_branch()
        (self.repo / "more.txt").write_text("more")
        git(self.repo, "add", "-A")
        git(self.repo, "commit", "-q", "-m", "more")
        self.assertTrue(any("lacks; push it" in p for p in problems(self.repo, {name})))

    def test_a_package_branch_without_an_open_pull_request_fails(self):
        self.package_branch()
        self.assertTrue(any("no open pull request" in p for p in problems(self.repo, set())))

    def test_a_merged_package_branch_must_be_deleted(self):
        name = self.package_branch()
        git(self.repo, "switch", "-q", "main")
        git(self.repo, "merge", "-q", "--ff-only", name)
        git(self.repo, "push", "-q", "development", "main")
        self.assertTrue(any("already merged" in p for p in problems(self.repo, {name})))

    def test_without_pull_request_data_only_those_tests_are_skipped(self):
        self.package_branch()
        self.assertEqual(problems(self.repo, None), [])
        (self.repo / "package.txt").write_text("dirty")
        self.assertTrue(any("uncommitted" in p for p in problems(self.repo, None)))

    def test_a_second_worktree_fails(self):
        git(self.repo, "worktree", "add", "-q", "--detach", str(self.repo.parent / "second"))
        self.assertTrue(any("worktrees" in p for p in problems(self.repo, set())))

    def test_an_archive_ref_fails(self):
        git(self.repo, "update-ref", "refs/archive/x/y", "HEAD")
        self.assertTrue(any("outside a branch" in p for p in problems(self.repo, set())))

    def test_a_stash_fails(self):
        (self.repo / "a.txt").write_text("changed\n")
        git(self.repo, "stash")
        self.assertTrue(any("outside a branch" in p for p in problems(self.repo, set())))

    def test_a_remote_branch_without_a_pull_request_fails(self):
        git(self.repo, "push", "-q", "development", "main:refs/heads/side")
        git(self.repo, "fetch", "-q", "development")
        self.assertTrue(any("holds 1 branch" in p for p in problems(self.repo, set())))

    def test_hosted_merge_queue_refs_are_ignored(self):
        git(self.repo, "push", "-q", "development", "main:refs/heads/gh-readonly-queue/main/pr-1-abc")
        git(self.repo, "fetch", "-q", "development")
        self.assertEqual(problems(self.repo, set()), [])


if __name__ == "__main__":
    unittest.main()
