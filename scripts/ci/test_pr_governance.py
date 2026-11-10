#!/usr/bin/env python3
"""Regressions for the pull-request work-in-progress limits."""

import unittest

from check_pr_governance import (
    GOVERNANCE_SINCE,
    INTEGRATION_LABEL,
    MAX_AGE_DAYS,
    MAX_LINES,
    MAX_OPEN,
    MAX_PER_AUTHOR,
    Facts,
    evaluate,
)


def facts(**overrides) -> Facts:
    base = dict(number=10, author="agent", base_ref="main", labels=[], changed_files=3, changed_lines=120,
                merge_base_age_days=0.5, created_at="2026-11-11T00:00:00Z",
                open_pull_requests=[{"number": 10, "author": "agent", "created_at": "2026-11-11T00:00:00Z"}])
    base.update(overrides)
    return Facts(**base)


def opened(count: int, author: str = "other") -> list[dict]:
    return [{"number": 10, "author": "agent", "created_at": "2026-11-11T00:00:00Z"}] + [
        {"number": 100 + i, "author": author, "created_at": "2026-11-11T01:00:00Z"} for i in range(count)
    ]


class GovernanceTests(unittest.TestCase):
    def test_a_small_fresh_single_pull_request_is_admitted(self):
        self.assertEqual(evaluate(facts()), [])

    def test_an_empty_diff_is_refused(self):
        self.assertTrue(any("changes no files" in v for v in evaluate(facts(changed_files=0, changed_lines=0))))

    def test_a_stacked_pull_request_is_refused(self):
        self.assertTrue(any("no stacks" in v for v in evaluate(facts(base_ref="agent/other"))))

    def test_size_limit_applies_unless_the_owner_labels_an_integration(self):
        big = facts(changed_lines=MAX_LINES + 1)
        self.assertTrue(any("changed lines exceed" in v for v in evaluate(big)))
        self.assertEqual(evaluate(facts(changed_lines=MAX_LINES + 1, labels=[INTEGRATION_LABEL])), [])
        self.assertEqual(evaluate(facts(changed_lines=MAX_LINES)), [])

    def test_open_pull_request_cap_counts_every_open_pull_request(self):
        at_limit = facts(open_pull_requests=opened(MAX_OPEN - 1))
        self.assertEqual(evaluate(at_limit), [])
        over = facts(open_pull_requests=opened(MAX_OPEN))
        self.assertTrue(any(f"the limit is {MAX_OPEN}" in v for v in evaluate(over)))
        self.assertEqual(evaluate(facts(open_pull_requests=opened(MAX_OPEN), labels=[INTEGRATION_LABEL])), [])

    def test_per_author_cap(self):
        mine = facts(open_pull_requests=opened(MAX_PER_AUTHOR, author="agent"))
        self.assertTrue(any(f"the limit is {MAX_PER_AUTHOR}" in v for v in evaluate(mine)))
        theirs = facts(open_pull_requests=opened(MAX_PER_AUTHOR, author="someone-else"))
        self.assertFalse(any("agent would have" in v for v in evaluate(theirs)))

    def test_a_stale_branch_is_refused_even_when_small(self):
        stale = facts(merge_base_age_days=MAX_AGE_DAYS + 0.1)
        self.assertTrue(any("Merge main into the branch first" in v for v in evaluate(stale)))
        self.assertEqual(evaluate(facts(merge_base_age_days=MAX_AGE_DAYS)), [])

    def test_the_integration_label_does_not_excuse_emptiness_stacking_or_staleness(self):
        violations = evaluate(facts(changed_files=0, changed_lines=0, base_ref="agent/x",
                                    merge_base_age_days=30, labels=[INTEGRATION_LABEL]))
        self.assertEqual(len(violations), 3)

    def test_the_backlog_opened_before_the_limits_is_not_counted_and_not_capped(self):
        backlog = [{"number": 200 + i, "author": "agent", "created_at": "2026-11-03T00:00:00Z"} for i in range(20)]
        new = facts(open_pull_requests=opened(0) + backlog)
        self.assertEqual(evaluate(new), [], "old open pull requests do not block a new small one")
        old = facts(created_at="2026-11-03T00:00:00Z", changed_lines=40_000, open_pull_requests=opened(0) + backlog)
        self.assertEqual(evaluate(old), [], "a backlog pull request keeps its size while it is absorbed")
        stale_old = facts(created_at="2026-11-03T00:00:00Z", merge_base_age_days=40, changed_files=0, changed_lines=0)
        self.assertEqual(len(evaluate(stale_old)), 2, "but it must still be non-empty and merged with main")
        self.assertLess("2026-11-03T00:00:00Z", GOVERNANCE_SINCE)

    def test_every_violation_is_reported_at_once(self):
        violations = evaluate(facts(changed_files=0, changed_lines=MAX_LINES + 50, base_ref="dev",
                                    merge_base_age_days=10, open_pull_requests=opened(MAX_OPEN, author="agent")))
        self.assertEqual(len(violations), 6)


if __name__ == "__main__":
    unittest.main()
