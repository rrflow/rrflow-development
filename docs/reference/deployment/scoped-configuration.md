# Scoped query and recall configuration

**Status:** active reference for the bounded numeric implementation; broader policy and UI work remains open
**Coordinate:** `rrflow://rrflow-instance/data/reference/deployment/scoped-configuration`
**Owner:** publication, binding and enforcement of shared user capacity settings

[ADR-0006](../../decisions/0006-scoped-policies-and-preferences.md)
owns the user/project/effective-configuration model. This implementation covers
eleven typed capacities: query storage keys, rows, output bytes, batch rows,
memory bytes, spill bytes and elapsed milliseconds; recall graph depth, items,
output bytes and storage keys. It does not implement arbitrary behavioral
obligations, organization policies, model preferences or Connectome controls.

## One durable source, explicit project bindings

The local OS owner publishes a user source at an explicitly chosen absolute
path. Choose a location outside project directories whose existing parent is
trusted. RRFlow creates a private directory containing `configuration.rrkv`,
using the existing native rrflowKV control repository. This is not another
engram or a daemon. Each project seals the source identity, root and history
read budget in its installation configuration. Discovery does not bind it.

Publication requires a five-minute, digest-bound preview and exact approval.
The native transaction compares the expected head and atomically appends an
immutable revision with the new head. Each revision is a complete settings
snapshot with a half-open valid interval and a recorded revision. Corrections
append history. A repeated apply returns its publication only while that exact
plan remains the head; an expired plan cannot create state.

Source access uses local OS ownership. A project bearer credential cannot
publish global changes through HTTP. A shared policy source grants no access
to another project's knowledge. This is not protection against another process
with the same OS owner's filesystem authority. Remote policy publication and
organization administration remain unimplemented.

Access briefly acquires the native writer lease. Contention, missing source,
wrong identity, unsafe permissions, corrupt history and exhausted history reads
refuse the operation; they cannot turn a required source into empty defaults.
This package makes no throughput claim for concurrent source access. Reads do
not refresh content validity or write a freshness timestamp. Interrupted native
initialization before a valid store exists requires explicit recovery; dedicated
user-source backup, removal, relocation and synchronization commands are not
implemented by this package.

## Publish and bind

Prepare a UTF-8 JSON file, for example `user-policy.json`:

```json
{
  "valid_from_unix_ms": 0,
  "valid_to_unix_ms": null,
  "settings": {
    "query_rows": {"minimum": 2, "maximum": 100, "preference": 20},
    "recall_items": {"minimum": 1, "maximum": 32, "preference": 16}
  }
}
```

These are capacity bounds. A minimum query capacity of two does not promise
two matching rows or a level of answer quality. Unknown setting names and
inconsistent bounds are rejected; additional semantic capabilities require an
explicit typed implementation.

```text
rrflow user-configuration plan --root <absolute-user-root> --source user-policies --expected-revision 0 --input user-policy.json
rrflow user-configuration apply --plan <saved-plan.json> --expect <plan_sha256>
rrflow user-configuration read --root <absolute-user-root> --source user-policies
```

Save the first command's complete JSON output as UTF-8 `saved-plan.json`, review
it, and supply its exact digest to apply. For an update, use the current expected
revision. Read also accepts `--valid-at-unix-ms` and `--recorded-revision` to
distinguish what was valid from what had been recorded. Publication previews
and reads accept `--max-history-reads` (default 256, maximum 4096).

Add the binding to each project's installation input before previewing and
applying installation through the existing installation workflow:

```toml
[scoped.user]
source_id = "user-policies"
root = "/absolute/private/user-policies"
max_history_reads = 256

[scoped.settings.query_rows]
preference = 12
maximum = 80
```

Use the platform's absolute path syntax. The source must already exist with
the specified identity. Project configuration remains sealed at installation;
this package does not add live project-policy editing.

## Resolve and enforce

Hard limits intersect: the greatest minimum and the smallest maximum across
user policy, project policy and installed ceilings apply. Selection is explicit
request, then project preference, then user preference, then installed default.
There is no silent clamping. An impossible intersection or invalid stored
default is a configuration conflict; an out-of-bounds request is a limit error.

Authenticated `POST /v1/configuration/effective/read` returns the installed
identity, project digest, selected user snapshot and read revision, units,
bounds, rule provenance and selected value for every setting. Optional
`valid_at_unix_ms` and `recorded_revision` are diagnostic historical coordinates;
they never authorize a historical policy for a new effect. A `requested` map
can explain specific capacities before use. The operation requires `engram.read`.

Query and recall admission load current policy from the bound source using host
time. Their returned query result or context packet includes a compact
`configuration` admission stamp with the actual project digest, evaluated time
and selected/read user revisions and digest. The context digest binds that
stamp. A separate diagnostic read cannot substitute for this execution evidence.
Already admitted work retains its admission; subsequent admission observes
updates without restarting the project server. This is not a transaction across
the project store and user store, nor mid-flight cancellation on a policy change.

Exact typed resolution does not use embeddings or approximate nearest-neighbor
retrieval. [Memory freshness](../data/knowledge-connectors.md#memory-freshness-and-vectorization) owns the planned derived
retrieval rules. Vectorization cannot override policy admission.

## Evidence and remaining scope

The [package journal](../../evidence/change-journals/gate-d/scoped-configuration-foundation-20261121.md)
records the installed two-project driver, restart, correction, historical read,
isolation and required-source failure checks. Numeric enforcement is a foundation
for the broader policy model, not completion of D-06 or POAM-077. Connectome
editing, behavioral obligations, controller integration and vector freshness
still need their own implementation and installed evidence.
