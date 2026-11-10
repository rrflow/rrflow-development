# Evaluation inputs: retired identity and trailing whitespace removed

**Status:** active repository correction; no engine, evaluation or release claim

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/eval-inputs-identity-20261110`

**Owner:** repository invariants in `crates/authority/rrd-engine/tests/workspace_architecture.rs`; OBJ-13 evaluation inputs carried by #308

**Starting revision:** main `cc139164` (merge of #308).

**Current behavior.** #308 landed through the docs and scripts lanes, which do not run the engine's repository-wide architecture tests. On main `cc139164`, `retired_pre_release_identity_is_absent_from_the_active_repository` and `tracked_and_untracked_text_files_have_no_trailing_horizontal_whitespace` fail: the staged evaluation prompt captures under `eval/inputs/staging/` contain the retired brand token seven times and the retired split-engine name (the product name joined to `-engine`) six times across four JSON files and one text file, and `rrflow-tech-candidates.txt` (UTF-8 BOM, CRLF) has fourteen lines with trailing spaces before the carriage return. Every engine-lane candidate merged with this main inherits both failures (first seen on #249's merge check).

**Target behavior.** The five files carry the current identity (`rrflow`, `RRFlow` where the capture was upper case) in place of the retired tokens and no trailing horizontal whitespace; line endings, BOM, structure and every other byte are unchanged. The prompts remain the same evaluation inputs: only a brand token inside free text changed.

**Changed paths.** `eval/inputs/staging/claude-rrflow-human-prompts.json` (2 tokens), `claude-rrflow-user-prompts.raw.json` (2), `codex-rrflow-human-prompts.json` (3), `rrflow-real-prompt-pool.json` (5), `rrflow-tech-candidates.txt` (1 token, 14 lines of whitespace).

**Acceptance.** `cargo test -p rrd-engine --locked --test workspace_architecture`: 33 passed on this tree (32 passed, 1 failed before the whitespace correction; the identity test failed on the merged #249 tree before any correction). The candidate run for this change selects the docs and scripts lanes, which do not run this test; the local result above is the evidence, and the lane gap is closed by a separate repository-invariants job change.

**Research and trace decisions:** none; a byte-level data correction.

**Checks not run:** no evaluation rerun on the corrected prompts; the token change does not alter what the workbench measures.

**Roadmap/POA&M change:** none.
