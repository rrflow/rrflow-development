# macOS cache candidate reconciliation with landed main

**Status:** active draft reconciliation; final candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/ci-macos-cache-main-reconciliation`
**Owner:** OBJ-02 / C-07 CI prerequisite / [PR213](https://github.com/rrflow/rrflow-development/pull/213)

The [PR brief](https://github.com/rrflow/rrflow-development/pull/213) authorized
an ordinary merge after PR210 landed. The cache candidate started at
`268fd9518440c673376737ce119118cc15f7035c`; public main was
`6fe8b7590d913beda9b009551e5d2d2644e406d9`. Before merging,
`git merge-tree --write-tree` returned clean tree
`7e6fad49d943a4e1dc7cbea886a6113155e78c31`. The no-fast-forward, no-commit
merge had the same staged tree. Relative to landed main, the candidate retains
only its two cache workflows, workflow-policy and cache-policy tests, and its
linked evidence/index; the temporary benchmark workflow is byte-for-byte equal
to landed main. No existing runtime test, candidate gate, compiler pin, cache
key safety input or native workload was intentionally removed.

`python scripts/ci/test_cache_warm_policy.py` passed five tests;
`python scripts/ci/check_workflow.py` and `python scripts/ci/check_documentation.py`
passed. Reviewed the original
[cache-key investigation](ci-macos-rustup-cache-identity.md),
[hosted probe](ci-macos-rustup-cache-probe.md), PR brief, exact parent revisions,
staged merge tree, relative-main diff and benchmark restoration. The small
probe showed a same-branch restore, not a full-build or main-to-candidate
speedup. PR216 overlaps the candidate workflow and remains in CI; this package
stays draft pending coordination with its actual landing state and fresh
candidate/protected CI. No duplicate full candidate was started here.

Checklist: recorded the merge scope and stop conditions before mutation;
compared staged tree with the independent merge preview; ran focused cache and
workflow-policy tests; regenerated navigation and execution inventory; checked
documentation and the final diff. No upstream source adapted, product source
rewritten, coverage weakened or roadmap/POA&M status changed. Any later main
movement requires its own reconciliation and exact-head evidence.
