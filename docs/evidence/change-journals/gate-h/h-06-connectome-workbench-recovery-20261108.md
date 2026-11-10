# H-06 Connectome Workbench recovery

**Status:** active draft integration; merge and qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-06-connectome-workbench-recovery-20261108`
**Owner:** H-06 Connectome integration, with POAM-011 and POAM-012 remaining open

## Baseline and change brief

The RRFlow starting revision is `16c7318a76e04a4d219a40c741cbbe2504ec5144`.
The separate Connectome candidate is
`bd0223be62de3bfee724327df158000b8478986f`, and the mainline merge parent is
`4da67e27f52ef20861fd9ee8c8351ce48920f49a`. The existing
[Connectome PR 45](https://github.com/rrflow/connectome-development/pull/45)
contains the integration brief, conflict trace, and stop conditions. The
working merge is not a commit and still has an unresolved Workbench file.

The candidate has dataset-authoring and source-management UI around an older
workspace. Mainline has a newer workspace shell, visual Workbench, engine
flight readback, and session-bound diagnostic capture. The target is one
workspace that carries useful candidate behavior forward without duplicating
engine authority, losing saved profile data, enabling unauthorized effects, or
presenting plans as executed results. This is a prerequisite to the real
project indexing and retrieval journey, not evidence that journey works.

The parent `connectome/` gitlink is locally modified and is not changed by
this journal. Other existing worktrees and their contents remain untouched.
Before removing any recovery folder, identify its exact path and demonstrate
that its unique changes are preserved in the integrated result.

Bulk reconciliation compared 1,039 unstaged or untracked paths, excluding
staged and conflicted paths, against the mainline tree with Git-filtered
hashes. It staged 989 existing files whose hashes and staged modes exactly
match mainline, retaining the already staged integration work. Forty-eight
old paths absent in both the working tree and mainline remain unstaged for
traceability. Two differing files were reviewed and staged: `src/types.tsx`
combines mainline types with candidate Workbench contracts; ExplorerView keeps
the current mainline import and differs only in whitespace. The full
`git diff --cached --check` against the older HEAD fails on pre-existing
mainline whitespace in historical Markdown and SCSS; the targeted check for
those two files against mainline passed. These mechanical steps do not prove
feature preservation or resolve the workspace conflict.
After the bulk staging, the dataset, state, template-interchange and
selected-file checks all passed again with `node --experimental-strip-types`
and their respective `scripts\check-*.mjs` files. The actual
`.\node_modules\.bin\tsc.cmd --noEmit --pretty false` failed with six TS1185
conflict-marker diagnostics in `workspace.tsx`; the candidate remains
unbuildable until its behavior is reconciled, not merely staged.

The owner clarified that this is a single pre-release implementation, not a
released legacy product. Earlier historical-export compatibility work in
this draft is not an independent product requirement; its eventual retention
or removal must be decided during the current-schema consolidation without
discarding actual saved user data. No compatibility cleanup was made in the
subsequent read-only planning step.
Following that plan, the historical-export-only defaults for missing skills
and references were removed from the draft interchange code. Current-schema
imports require both fields; incomplete inputs fail rather than being
silently filled. Existing stored profiles were not modified by this change.

The candidate source panel is now composed in the newer Workbench through
`workbenchSources.tsx`. Saved linked paths are visible and removable without
erasing template references; saved cloud sources can be added, opened
explicitly and removed while re-reading the current profile. Newly added
sources are marked `needs-auth`, and recorded path/source statuses are not
represented as current verification. The local document editor can copy text
or request a user-selected file save. Native path linking/probing,
reconnection, remote document reads, and folder synchronization remain
unavailable; no arbitrary-path native command was registered and no remote
fetch was added. This restores profile metadata management, not the complete
candidate source integration. The newer UI and unresolved workspace file were
read along with `workbenchLinkedCloudPanel.tsx`, `workbenchState.ts`,
`workbenchTemplateTransfer.tsx`, `format.ts`, the browser and desktop file
adapters, `src/types.tsx`, and `scripts/check-workbench-template-transfer.mjs`.

## Trace and changes so far

The PR records the parent comparisons for the native service projection,
graph and client documentation, session request types, record editor and
project controls, runtime shell, historical sprint records, retired
unmounted domain notice, and candidate CI. The retained CI uses the mainline
merge-queue gate, Node version, provenance checks, frontend, native, and real
RRD jobs. Historical records use the complete mainline versions; the
candidate's versions remain in Git. The removed domain notice had no source
caller in either parent, and the candidate changed only its React hook.

The candidate's dataset artifact builder marked unexecuted controller,
cold-trigger, and comparison checks as passed and labelled components
enabled. Its generated plan now labels those components planned and the
unexecuted checks unverified. A configured remote source is not treated as
proof that it is available. The six artifact names and planned workload data
remain; the checklist status shape intentionally changes. The focused
`scripts/check-workbench-dataset.mjs` check is wired into `pnpm check`.

The newer Workbench now normalizes absent candidate collections when creating
or selecting a local Workbench and when saving a diagnostic note. It reads
the latest connection before each of those writes and retains existing
profile fields. The pure normalization check covers absent, legacy, and
populated examples; it does not yet prove browser persistence, concurrency,
or authoring integration.

The candidate workspace additionally normalized nested legacy template
fields. The shared normalizer now carries that behavior forward: missing
skills, research references, learning events, signoff, triggers, and actions
receive the candidate's defaults, while existing values and unrelated
profile fields are retained. The trigger ID comes from the existing caller's
`newId`. The candidate workspace itself remains intact and unmerged.

The canonical Workbench now contains an in-progress dataset-plan panel using
the candidate artifact builder, saved templates, workload profiles, run
label and telemetry settings. It previews each generated JSON file and asks
the existing adapter to save that file through the user-selected file
boundary. Its output is explicitly a snapshot and not an execution or
comparison; configured source URLs are disclosed before export. The panel
does not yet restore template authoring, linked-folder synchronization, cloud
or scripted-event capabilities. It does not resolve the workspace conflict.
The candidate builder also omitted learning-event `body` from its manifest;
the field is now preserved without removing the other fields.

The candidate's version-1 template importer silently discarded unresolved
linked-path IDs, truncated steps, and skipped invalid entries. A new
Valibot-validated interchange module keeps authored template, nested event and
path-reference data, regenerates imported object IDs, and rejects malformed or
unsupported files without a partial import. Unknown fields on validated
objects are retained. The canonical Workbench now exposes explicit
profile-scoped import and export using the existing adapter dialogs; it
re-reads the profile after asynchronous imports, and discards a pending
result after the component unmounts. This does not yet restore template
authoring or library controls. Invalid legacy files the previous permissive
importer accepted now fail explicitly; compatibility with all historical
export variants remains to be characterized before resolving the conflict.

The candidate saved-template library is now composed inside the newer
Workbench. It displays template and learning-event data, identifies unresolved
linked-path references, resolves only known current or mapped legacy routes,
and removes a template from the latest profile without replacing notes or
remote-source data. Scripted execution and trigger evaluation remain visibly
unavailable; this is not replacement evidence for those capabilities. The
library, transfer and plan panels now use distinct React keys; a browser test
exposed that duplicate keys could let a pending import cross a profile switch.

The candidate template authoring panel is now composed in the same Workbench
with connection-scoped drafts. It saves schema, skills, references,
requirements, patterns, routed steps and learning-event definitions into the
latest profile. The existing separate transfer control owns import/export;
authoring definitions does not activate event triggers or actions. This
remains partial until the full old workspace feature map, historical import
compatibility, profile reload and real engine behavior have been verified.
Git history establishes one concrete legacy export shape: the initial
version-1 exporter and template contract at `1391a5e4` predate `skills` and
`researchReferences`. The interchange validator now accepts those two fields
being absent and fills empty collections, while still rejecting invalid
present values and unsupported effects. This does not establish compatibility
with every file the old permissive importer accepted.

The shared browser file-picker helper did not close its writable stream,
which could leave a selected file uncommitted. Its stream lifecycle is now
extracted to `write-selected-file.ts`: successful writes close, response
bodies stream without implicit close, and failures attempt abort while
preserving an abort failure alongside the original error. Picker fallback and
user selection remain in the existing adapter. This changes a shared export
boundary and requires the complete application and native verification
listed below, not only the isolated checks.

The remaining integration map is recorded in the PR. It keeps mainline's
session-bound flight and diagnostic controls in the single Workbench; carries
the candidate's template, dataset, linked-source, and learning authoring into
that surface; and preserves existing profile collections against stale
updates. The existing browser and desktop `ConnectomeAdapter.saveFile` and
`openFile` implementations are candidate boundaries for explicit file
selection. Their suitability and cancellation behavior must be tested before
use; the candidate's arbitrary-path commands remain unregistered. Cloud
authentication, unattended folder synchronization, and scripted engine
effects cannot be described as available without a supporting contract.

Changed Connectome paths and the detailed conflict choices are in the PR and
the current merge index. The parent evidence paths are this journal and its
generated navigation index. No new runtime capability, benchmark, engine
receipt, or roadmap completion is claimed.

## Reading and decisions

Completely read for this increment: Connectome `workbenchDataset.ts`,
`workbenchLaneHelpers.ts`, `workbenchDatasetPanel.tsx`,
`RuntimeConnection/areas.ts`, the new dataset and profile-normalization
checks, `workbenchState.ts`, and the template interchange and transfer
implementations and tests. Selected ranges and
diffs, rather than complete files, were inspected for the parent `README.md`,
change-authoring and traceability records, H-06 and POA&M owners, native
projection, CI workflow, Workbench, runtime page, types, candidate workspace,
template authoring and library panels, browser and desktop file adapters,
native external-path commands, file-system save helper, Vite configuration,
and browser smoke tests. Full-file reread and complete diff review are
pending.

No external source adaptation was needed. Browser inspection is needed for
the integrated shell, and real-process traces and resource measurements are
needed for the project journey and comparison; none can be inferred from this
plan-generation change. Stop for unauthorized external file effects, loss of
profile data, a duplicate lifecycle authority, unsupported success claims, or
an unreviewed worktree deletion.

## Verification and remaining work

- Before correction, `node --experimental-strip-types
  scripts\check-workbench-dataset.mjs` failed: `classifier` was `enabled`
  rather than `planned`. After correction the same command passed for an
  optional-source profile and a required-source profile with both no source
  and a configured, unauthenticated source.
- The mainline-preview Biome executable, run with `check` on the dataset
  builder, test, and `package.json`, passed. It also passed checks on the
  reconciled session, record editor, project controls, runtime page, and
  Workbench initialization.
- `node --experimental-strip-types scripts\check-workbench-state.mjs`
  initially failed because the normalization module did not yet exist; it
  passed after implementation. `node --experimental-strip-types
  scripts\check-workbench-dataset.mjs` also passed afterward.
- `pnpm exec biome check` on the normalization files initially reported
  formatting errors and unexpectedly restored dependencies in the merge
  worktree despite no explicit install request. The local Biome executable's
  `check --write` followed by `check` on `workbench.tsx`,
  `workbenchState.ts`, `scripts\check-workbench-state.mjs`, and
  `package.json` passed. The Node version warning remains.
- `git diff --cached --check` passed after staging the normalization files;
  the Workbench conflict remains unmerged.
- A nested legacy-template case added to `node --experimental-strip-types
  scripts\check-workbench-state.mjs` failed before the candidate's nested
  normalization was carried across (`skills` was undefined), then passed.
  The check also covers passive and interactive signoff defaults, existing
  empty triggers/actions, and preservation of the input object. The dataset
  plan check passed again. Browser/profile reload remains untested.
- The expanded `node --experimental-strip-types
  scripts\check-workbench-dataset.mjs` first failed because exported event
  body was undefined; it passed after the builder was corrected. The same
  command checks all six artifact names and unverified plan statuses.
  `node --experimental-strip-types scripts\check-workbench-state.mjs` also
  passed. The local Biome executable's `check` passed on the builder, panel,
  Workbench and dataset check.
- `.\node_modules\.bin\tsc.cmd --noEmit --pretty false` failed on the
  unresolved `workspace.tsx` conflict markers. A diagnostic TypeScript run
  using an in-memory `ProjectOverview` stub reported zero diagnostics, but
  this is not a successful integrated typecheck or proof of the workspace
  resolution.
- An inline `node -` Vite/Playwright component harness with a virtual test
  entry and a simulated browser save picker passed template selection, plan
  generation, the six save buttons, saving the manifest, and preservation of
  event body in the captured JSON. It did not load the application shell or
  real engine, and did not verify native file dialogs or cancellation. The
  first harness attempt failed because `playwright` was not directly
  installed; the second could not parse JSX in its virtual entry; and the
  third exposed a missing Vite React preamble followed by a non-unique
  selector. The final harness used the installed `@playwright/test`, a
  JSX-free virtual entry, the Vite HTML transform and an exact textbox
  selector. It shut down its temporary server and browser on completion.
- `node --experimental-strip-types scripts\check-file-write.mjs` initially
  failed because the stream helper had not yet been created; after the fix it
  passed string, blob, streamed response, empty content, provider/write/close
  failures and abort-failure checks. It is included in `pnpm check`.
  The local Biome executable checked the eight affected implementation,
  test and manifest files successfully. The diagnostic in-memory workspace
  stub typecheck again reported zero errors; the normal typecheck remains
  blocked by the real conflict. An additional exact abort-call check passed;
  its first Biome check reported formatting, corrected by `check --write`,
  after which the test and `git diff --cached --check` passed.
- A second inline Vite/Playwright component harness simulated a picker whose
  file commits only on stream close. It passed generation, six file controls,
  committed manifest content, picker cancellation and picker error reporting.
  An earlier attempt failed because its text selector matched the status
  message as well as the filename; an exact selector fixed the test. This is
  not a real filesystem or native-dialog test.
- `node --experimental-strip-types
  scripts\check-workbench-template-interchange.mjs` initially failed because
  the module did not yet exist; after implementation it passed complete v1
  data preservation, fresh IDs, unresolved linked-path retention, unknown
  object fields and rejection of invalid, partial, future-version and
  over-limit inputs. The command is included in `pnpm check`.
- `node scripts\check-workbench-template-transfer.mjs` passed in an isolated
  Vite/Playwright component harness: export, import, invalid input without
  partial writes, cancellation, a concurrent profile note update and a
  profile switch while import was pending. Picker results were simulated; no
  actual browser/native file chooser or persistence after reload was tested.
  The script is now part of `pnpm test:smoke`; the complete production smoke
  suite was not run. Targeted Biome formatting passed, and the diagnostic
  in-memory workspace-stub TypeScript run reported zero errors; the normal
  typecheck and full application build remain blocked by conflict markers.
- The expanded `node scripts\check-workbench-template-transfer.mjs` browser
  component check initially failed twice: a pending import was committed to
  the old profile after the profile switch. Distinct React keys for sibling
  panels fixed the failure; the check then passed library display, unresolved
  references, known/unknown route handling, disabled scripted execution,
  removal preserving notes and remote sources, transfer and profile switching.
  Targeted Biome passed on the library, Workbench, runtime page and test.
  The diagnostic stub typecheck had zero diagnostics before the final
  key-only fix. The component harness is simulated and does not qualify the
  integrated desktop or real-engine behavior.
- `node scripts\check-workbench-template-transfer.mjs` passed again with
  added creation checks for template skills and requirements, three default
  steps, an authored event body, action and manual trigger, alongside library
  removal and preservation of remote-source state. The same test includes
  the previously reported transfer and switching cases. Targeted Biome
  `check --write` passed for the authoring, library, Workbench, runtime page
  and test files. The temporary workspace-stub TypeScript diagnostic reported
  zero errors after authoring was added; the unresolved conflict still blocks
  a normal typecheck.
- `node --experimental-strip-types scripts\check-workbench-template-interchange.mjs`
  passed with a version-1 fixture matching the initial committed contract,
  round-trip preservation, and invalid collection rejection. The browser
  transfer check passed again after the compatibility change, including an
  additional concurrent-note preservation assertion when saving an authored
  template. Targeted Biome passed on the interchange code and test.
- After removal of that historical-export fallback,
  `node --experimental-strip-types scripts\check-workbench-template-interchange.mjs`
  passed current-schema round-trip, missing-field and invalid-value checks;
  `node scripts\check-workbench-template-transfer.mjs` and targeted Biome
  also passed. The earlier legacy-fixture result above describes the draft
  at that time, not the current behavior.
- `python scripts\knowledge\render_navigation.py` passed in the parent
  checkout, reporting 18 generated indexes, 986 nodes, 3451 edges and four
  updated index files. `git diff --check` passed. The generated index changes
  outside Gate H remain in the worktree and require review before inclusion.
- The candidate restores a visible manual trigger after each authored event;
  the integrated authoring form initially failed that check. The focused
  browser test then exposed two ambiguous Remove-button selectors after the
  default trigger was restored; both selectors were narrowed to the intended
  template. `node scripts\check-workbench-template-transfer.mjs` and targeted
  Biome passed after the fixes. This is component evidence only, not a merge
  or a real-engine qualification.
- `cargo test -p rrd-engine the_project_map_describes_the_project_and_keeps_the_owners_notes --lib --locked`
  passed in the parent worktree (one test; 358 filtered out), with existing
  naming warnings. It does not cover oversized or malformed maps, which may
  overwrite owner notes and remain a verified source-level risk.
- `node scripts\check-workbench-template-transfer.mjs` passed with source
  add/remove, HTTPS and Google Docs URL validation, explicit disabled native
  and document-load actions, saved-link removal, profile separation and
  concurrent note preservation, plus selected-file save success, simulated
  cancellation and simulated failure. Two intermediate attempts failed on
  Mantine option selectors; the final isolated harness dispatches the option
  click, so it does not prove real dropdown pointer interaction. Targeted
  Biome passed on the source panel, container, Workbench and harness. The
  in-memory workspace-stub TypeScript diagnostic reported zero errors; this
  is not an integrated typecheck. Real native dialogs, clipboard permissions,
  remote access and persisted reload were not tested.
- `rustfmt --check --edition 2021 --config skip_children=true` passed for
  `src-tauri\src\rrd\view.rs`, `cluster.rs`, and `client.rs`.
- The existing candidate CI failed frontend source checks and native
  formatting on the three OS jobs; its SDK/RRD process integration job
  passed. These results belong to the old candidate, not the current
  uncommitted merge.
- The mainline overview rendered in a browser preview at revision
  `4da67e27`; the desktop, merged Workbench, and real engine journey were
  not demonstrated. Node available locally is 22.23.2; mainline requests
  Node 24.22.0 or later.
- `python scripts\knowledge\render_navigation.py --check` initially reported
  stale Gate C, D, H, KB-05, and repository journal indexes. The Gate H index
  was regenerated using the renderer's `generated_index_body`, `render_index`,
  and `_atomic_write` functions without changing the other four indexes. A
  subsequent `--check` still reported those four pre-existing stale indexes.

Not yet run for the integrated result: full typecheck, frontend build and
smoke, native build and tests, cross-platform candidate CI, installed
project indexing/retrieval, benchmark comparisons, and folder cleanup. The
last unresolved Workbench conflict includes template, linked-source,
learning-event, flight, diagnostic, and local-note behavior; each must be
mapped to the canonical destination and tested before it is resolved.

## Explicit learning-event actions

The existing Connectome draft PR was updated with the action integration
brief before these edits. The newer Workbench now offers explicit operator
execution of route navigation, storage and governed-flight refresh reads,
and diagnostic-note capture. All actions are checked before execution;
unknown actions and unavailable routes or prerequisites fail visibly.
Navigation must be last. Execution stops when the session, workbench, or
originating template event changes. A diagnostic capture also checks that
identity immediately before writing the local note. If an action fails, the
UI warns that earlier actions may already have completed; sequences are not
atomic. Captured notes are local diagnostics, not engine audit receipts.
The automatic passive-event loop is not restored, trigger signals are not
wired, and this is not an engine-owned learning workflow.

The implementation and tests read for this step were `workbench.tsx`,
`workbenchEventActions.ts`, `workbenchTemplateLibrary.tsx`,
`workbenchTemplateLibraryPanel.tsx`, `visuals.tsx`, `package.json`,
`scripts/check-workbench-event-actions.mjs`, and
`scripts/check-workbench-template-transfer.mjs`. The new browser harness is
`scripts/check-workbench-event-browser.mjs`. `package.json` includes the
action preflight test in `check` and the browser harness in `test:smoke`.

- `node --experimental-strip-types scripts\check-workbench-event-actions.mjs`
  passed preflight, ordering, unsupported-action, disconnected, error and
  context-change cases.
- `node scripts\check-workbench-event-browser.mjs` passed against the actual
  Workbench component with a simulated engine: operator dispatch, read calls,
  local diagnostic note, visible failure, a changed event during a pending
  capture without another note, and session switching during a pending read
  without continuing the action sequence.
- `node scripts\check-workbench-template-transfer.mjs` passed the existing
  isolated template and source component checks.
- Targeted `.\node_modules\.bin\biome.cmd check --write` passed for the
  changed event implementation and test files and `package.json`.
- `pnpm check` failed at the initial Biome step with 136 diagnostics, including
  the unresolved conflict markers in `workspace.tsx`; subsequent checks in
  that chained command did not run. It also warned that local Node 22.23.2
  does not satisfy the required Node >=24.22.0.

These are component-level results with simulated engine replies. The full
smoke suite, native UI, persistence after restart, real engine, trigger
evaluation, and complete migration of the candidate's Workbench behavior
remain unverified. No merge, roadmap-status change, or folder deletion has
occurred.

## Grouped runtime actions

The grouped-action brief was appended to Connectome PR 45 before this change.
The candidate's `workspace.tsx` grouped action definitions and handler,
`workbench.tsx`, `workbenchDatasetPlans.tsx`, `workbenchTemplateLibrary.tsx`,
`queries.ts`, `tabs/models.tsx`, the existing event browser harness, and
the H-06 owner were read. The candidate's preset hop, action, and tool counts
were not measurements. The new `workbenchRuntimeActions.tsx`, composed in
`workbench.tsx`, instead exposes operator-initiated schema, graph snapshot,
storage, and flight reads, navigation to the corresponding lenses, and
local diagnostic capture. It shows returned counts/cursors and measured UI
elapsed time, updates the session-scoped query caches for those reads, and
does not represent a local note as an engine audit receipt. A disconnected
session cannot issue reads; a session or workbench change unmounts the
control, preventing pending results and notes from entering its replacement.
The existing observe-flight prompt remains explicit; no invented cold-path
flight is launched. The candidate's telemetry profile and hop/action/tool
tracking controls have not been reproduced as measured engine telemetry.
Guided routes, workskills, routines, automatic triggers, linked-path probing,
remote reads, and folder synchronization remain separate unresolved traces.

- `node scripts\check-workbench-event-browser.mjs` passed the expanded
  simulated Workbench harness: schema, database, storage and flight reads,
  local capture, navigation, a read failure, and pending read/capture across
  session changes. It also checked that a stale schema read did not populate
  its query cache and a stale capture did not write a local note.
- `node --experimental-strip-types scripts\check-workbench-event-actions.mjs`
  and `node scripts\check-workbench-template-transfer.mjs` passed.
- Targeted `.\node_modules\.bin\biome.cmd check --write` passed for the new
  action component, `workbench.tsx`, and the browser harness.
- An in-memory TypeScript diagnostic using `MERGE_HEAD`'s workspace source
  in place of the conflicted file returned zero diagnostics. This is not a
  normal typecheck of the merged workspace.
- The owning `pnpm check` previously failed in its first Biome stage on the
  unresolved workspace conflict; the full owning suite, native validation,
  real-engine journey and candidate CI have not passed. Local Node remains
  below the required version. No roadmap status changed.

## Mainline-retired paths

The 48 unstaged deleted paths were compared against candidate changes since
common ancestor `5542691e28605a4a14841b1215935468d06fc0f3`; none were
candidate-modified. `git cat-file -e MERGE_HEAD:<path>` confirmed all 48 are
absent from the mainline merge parent. Searches of mainline for the retired
TS/SCSS paths and installer image names found only historical sprint prose;
mainline's `.agents/sprints/009-connectome-ownership.md` explicitly records
the unused installer images' retirement. Their canonical destination is the
existing mainline UI and assets. The exact path set is the Git deletion diff,
not a second manually maintained inventory. The corresponding trace and stop
condition were added to PR 45 before staging the already-absent paths.
Comparison and searches do not establish native installer or visual
equivalence; normal build, visual/native checks and candidate CI remain
required, and no worktree folder was removed.

## Guided route carry-forward

The PR brief was extended before carrying the candidate's two `routePlans`,
three `workskills`, and two `routines` from the conflicted `workspace.tsx`
into `workbenchGuidedRoutes.tsx`, composed in `workbench.tsx`. Their routes
resolve through the existing current-shell route mapping. The component shows
the candidate route steps and routine destinations and exposes the workskill
lenses; it explicitly identifies them as navigation, not executed or verified
engine routines. It does not replace the candidate's live storage signal,
automatic learning triggers, external path and document operations, or
dataset multi-file output.

- `node scripts\check-workbench-event-browser.mjs` initially failed because
  the routine label matched both a textbox and listbox. After narrowing the
  locator, it passed both route plans, three skill buttons, both routine
  selections, mapped storage/workbench routes and the existing runtime-action
  and event tests. Selection uses dispatched option clicks, so real pointer
  interaction is not proven. The engine is simulated.
- Targeted `.\node_modules\.bin\biome.cmd check` on the component,
  `workbench.tsx` and browser script passed.
- `node --experimental-strip-types scripts\check-workbench-event-actions.mjs`
  and `node scripts\check-workbench-template-transfer.mjs` passed.
- An in-memory TypeScript diagnostic substituting the mainline workspace
  reported zero diagnostics. The real workspace conflict remains; this is
  not an integrated typecheck, application build, or real-engine evidence.
  The full owning `pnpm check` remains blocked as previously recorded. No
  roadmap status changed and no folders were deleted.

The supported Node runtime was subsequently found through `fnm list`;
Node `v24.22.0` was already installed but not on the default PATH. With
`fnm env --shell powershell | Out-String | Invoke-Expression` and
`fnm use 24.22.0`, the event browser, event-action, and template-transfer
commands above passed again. `pnpm check` under Node 24.22.0 still failed
at Biome with 136 errors, including the unresolved `workspace.tsx` conflict
markers; later chained checks did not run. Earlier statements that no
supported local Node was found describe the prior investigation, not the
current situation. The browser test was then expanded to exercise every
guided-plan step, workskill destination, and routine step; it passed under
Node 24.22.0 with the same simulated-session and option-dispatch limitations.

## Candidate event-note carry-forward and workspace characterization

The candidate's `ProjectSection.runLearningEvent` writes a local testing
note after successful action execution when a Workbench is selected. That
behavior was absent from the newer integration. The PR brief was extended
before modifying `workbench.tsx` to record the original event, template,
action, trigger and catamorphism details through its existing latest-profile
note writer. The write only follows a completed event while the originating
template, event, selected Workbench and session remain current. A failed
action does not get a success note. If navigation unmounts the Workbench,
there is no guarantee of a note; these are local notes, not engine receipts.

The candidate and mainline versions of `workspace.tsx`, current
`workbench.tsx`, `workbenchTemplateLibraryPanel.tsx`, and the event browser
test were read. An in-memory compiler experiment concatenated the exact Git
stage-2 candidate with the stage-3 overview, dropping only the latter's
duplicate import. Under Node 24.22.0 it produced 23 TypeScript diagnostics:
the candidate refers to removed `flight_count`, `task_count`, `latest_*` and
`recent_flights` fields, calls `workbenchSnapshot` without its required
cursor, and passes incompatible props to the changed panels. This establishes
concrete API mismatches, not that candidate behavior is useless. It did not
modify either conflict stage or prove runtime behavior.

- Under `fnm use 24.22.0`, `node scripts\check-workbench-event-browser.mjs`
  passed successful event-note content, failed actions with no success note,
  concurrent note preservation, no selected Workbench, and pending operations
  across event, Workbench and session changes. It uses simulated engine reads.
- `node --experimental-strip-types scripts\check-workbench-event-actions.mjs`,
  `node scripts\check-workbench-template-transfer.mjs`, and targeted Biome
  passed under Node 24.22.0.
- A TypeScript diagnostic with the stage-3 workspace substituted in memory
  returned zero diagnostics after the final Workbench-identity guard. It is
  not an actual integrated build. The owning package check remains
  blocked by the unresolved conflict. Full desktop, real engine, triggering,
  and merge validation were not run. No status changed.

Checklist: baseline and owning records identified; PR brief and trace
recorded; first failing plan oracle reproduced and corrected; Gate H
navigation regenerated; no unrelated worktree or parent gitlink reset.
Full-file reread, final diff, remaining navigation drift, candidate CI and
merge remain pending. No roadmap or POA&M status changed.
