# CI inspection integrated with the Inventory candidate

**Status:** active draft evidence; C-07 and the alpha remain open.
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-c/c-07-ci-inventory-integration`
**Owner:** PR #216; depends on protected landing of PR #224.

The starting local revision was `5954fa5618096927653e221347f7bec0da94b467`.
The PR brief authorized a normal merge of #224 head
`ffe3af9f73177a0af3d594a637f78368b6b93c16`, which contains public main
`f256c8e1337d85fa01f2a0c61e895a4d213163fa`. The read-only merge-tree and the
actual merge were clean. The integrated tree before this journal was
`572de84092e817eb82cd0123c3b9275425536114`. The CI workflow still invokes all
existing partitions and adds the inspector's own test; the offline guide refers
to the runner's actual `REQUIRED_CASES` rather than a duplicate count. No
native case, source-identity check, required status or branch rule was relaxed.

Read the complete inspection script and tests, the affected CI workflow and
guide diff, the dependency's offline runner/policy sections and linked prior
journals. The combined diff preserves the source-bound diagnostic and the
seven-case offline harness admission. This record and the generated parent
index are the only integration-specific evidence paths. Full changed-file
reread is still required before push. No upstream algorithm, new runtime trace,
credential logging, capacity measurement or product capability is introduced.

On Windows, `python scripts/ci/test_inspect_candidate_runs.py` passed nine;
`python scripts/ci/test_offline_lifecycle.py` passed 18; and
`python scripts/ci/check_workflow.py` passed. The read-only inspector compared
protected failure 41156697702 to candidate 41161519177, reported the optional
engine and aggregate failures, ranked their completed build steps, and identified
different tested trees and nonmatching current #224 head. It did not infer a
cause or replace the retained failure. Navigation regeneration passed;
`python scripts/ci/check_documentation.py` passed (832 document statuses,
830 classified coordinates); execution inventory regeneration and `--check`
passed for 2,104 records; and staged whitespace checks passed.

The #224 candidate was still running at this record's creation. Its protected
landing, this package's main synchronization and full-file review, and its own
candidate and protected merge-group checks remain required. The earlier #216
macOS candidate failure remains recorded and unresolved. No roadmap or POA&M
completion status changed.
