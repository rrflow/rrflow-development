# H-05: stack audit repair on the SDK privacy candidate

**Status:** active integration evidence; prerequisite and candidate CI pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-05-audit-sdk-dependency`
**Owner:** OBJ-10 / H-05 / [PR144](https://github.com/rrflow/rrflow-development/pull/144)

The PR brief declared the explicit dependency before the merge. Audit head
`00b51009d47cbc7c3b5e69fff8ca9b0705102f21` and SDK head
`d44268dbd191dabcd7ee4a4b991df8e6dd83930a` share public main
`81fce08e14497bbc280790beefe4666716355aec`. Their independently added Gate H
journal indexes and Git inventory conflicted. The SDK head was merged normally;
both generated files were regenerated from the combined tree. No source,
security restriction, prior failure evidence or journal was discarded. This
stack prevents those two known conflicts from recurring when PR177 lands first;
it does not guarantee against unrelated intervening changes to main.

On native Windows x64, with isolated `target/audit-page-tail-refresh-build`,
locked offline dependencies and disabled debug info, the original fixed
`rrd-security` prefix test passed; the real HTTP audit read/export and merged
storage-diagnostics trace tests each passed; all 31 engine workspace architecture
tests passed; `python -m unittest discover -s scripts/ci -p
 test_sdk_private_manifest.py` passed four Windows tests with one POSIX case
skipped. The previous audit and SDK journals retain their full package suites,
strict lint, first failures and limits. No new physical behavior or research
adaptation was introduced. Reviewed the dependency PR, conflict paths and
combined non-generated source diff. Final format, navigation, documentation,
workflow and inventory checks are required before commit.

PR177 must first pass its current exact-head candidate CI and the protected
merge-group, and land; PR144 requires its own new exact-head candidate and
merge-group CI. Earlier CI runs do not transfer to this combined head. No
roadmap or POAM status changes. Checklist: both heads and baseline recorded;
explicit dependency in PR brief; generated-only conflicts resolved; real
cross-boundary and architecture checks rerun; old evidence preserved; no
capacity, memory-receipt or alpha completion claim.
