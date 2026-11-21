# Pin merged Connectome recovery history

**Status:** active integration evidence; child merged green; parent gate pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/connectome-recovery-pin-20261121`
**Owner:** [Gate H](../../../roadmap/rrflow-1.0/gate-h.md), H-06 and [POAM-011](../../../poam/rrflow-1.0-alpha/poam-011.md)

Baseline 15ed675d403b30eabfe69882bda39e1a0ce34c6c. Advance the optional
Connectome gitlink from f1dde27ffe858a7f955f86b603a409479e6eec50 to child main
ed53cfdbd216db73b72a523311fdebadfd458c26. This binds the parent checkout to the
merged policy UI and recovered graph/installation stash history. Exact scope:
gitlink, this record and its generated index. No engine code or contract changes.

The [client checkout contract](../../../reference/client/connectome.md#repository-checkout)
owns the boundary. Child [PR110](https://github.com/rrflow/connectome-development/pull/110)
merged at fd7836a880f683ecad5e864724b8421eead5349a after candidate and queue
checks. [PR111](https://github.com/rrflow/connectome-development/pull/111) merged
at ed53cfdbd216db73b72a523311fdebadfd458c26 after its required candidate and
[queue checks](https://github.com/rrflow/connectome-development/actions/runs/44931739250)
passed. Both source branches were deleted remotely and locally after verified
recovery bundles. The child remote currently exposes only main.

Read the client checkout owner, child recovery journal and parent gitlink.
Acceptance: exact child commit is readable and fetched from the declared public
remote, documentation/navigation checks, parent candidate and queue. Stop on a
missing object, changed source URL, unmerged child or failing gate. No upstream
adaptation, new research, trace or resource experiment is needed for this pin.
No new desktop trial, whole-platform parity or completed engine learning journey
is claimed. The child journal records the original local tests and limits.
Other recovered work remains outstanding. Roadmap status is unchanged.

Local validation: declared child remote main resolves to ed53cfdbd216db73b72a523311fdebadfd458c26; documentation-policy and git diff --check passed. Parent hosted candidate and queue remain pending in PR462.
