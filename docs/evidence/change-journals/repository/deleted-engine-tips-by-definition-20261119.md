# Every deleted engine tip, read by the definitions its source added

**Status:** active change-journal evidence; one missing contract test restored, nothing else claimed

**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/repository/deleted-engine-tips-by-definition-20261119`

**Owner:** repository absorption; continues [the reading of unmerged work](unmerged-pull-requests-by-intent-20261119.md)

**Starting revision:** main `fc5767b4c`.

## Change brief

- **Current behaviour.** Earlier passes read deleted work by the tests it added. A behaviour that shipped without a test could not be found that way.
- **Target behaviour.** Every commit this checkout still holds that `main` cannot reach is read by the functions, types, constants and modules its source added, one branch at a time, and anything `main` lacks is ported.
- **Scope.** `crates/transport/rrd-contract/tests/endpoint_worker.rs` and this record.
- **Failure oracle.** A definition a tip added whose name, and whose behaviour under another name, `main` does not have.
- **Research and trace decisions.** None needed.
- **Stop conditions.** Stop and port before going on whenever a branch's behaviour is absent from `main`.

## Method

`git fsck --dangling --no-reflogs` lists 135 commits `main` cannot reach. For each, its tree is compared with its merge base on `main`, and every added `fn`, `struct`, `enum`, `trait`, `const`, `static`, `type`, `mod`, `def` and `class` in a source file is looked up on `main` as a whole word. 98 tips added no name `main` lacks. 37 added 127 distinct names `main` lacks; those were read by hand, grouped by the branch they came from.

57 of the 127 are the earlier word for engram and exist on `main` under the engram name.

## Result, branch by branch

| Branch (tips) | What it set out to do | On `main` |
|---|---|---|
| `agent/existing-project-inherited-access` (`d76fee096`) | Install into a project whose `.rrflow` folder already exists, keeping it | Yes. `with_retained_engram`, nine `retained_engram_*` tests, `inherited_engram_preview_retains_the_folder_without_effects` |
| `agent/adopt-existing-rrflow` (`420f35e66`) | Move a leftover `.rrflow` into `previous/` | Replaced on purpose by retention above; `a_leftover_rrflow_folder_is_retained_untouched…` holds it |
| project map draft (`0976c4ed1`) | Inventory honours `.gitignore`; write a project map | Yes. `installation/inventory/git_inputs.rs` with tests; `project_map.rs`, called after install and on serve |
| Parse executor stack (`0fe2d60fe`, `7fba8ea5a`, `b56f23af1`) | Parse runs in bounded batches with oracles | Yes, as the Parse Step. Its six tests map to `parse_step.rs`. Where the draft rejected a file whose facts exceed one commit, `main` publishes them across Steps |
| `agent/d04-source-spans` (`a105804f8`, `c53cf2104`, `c5491c061`) | Parser with exact source ranges and limits | Yes. Every function it added exists by name; all seven parser tests are in `rrd-attunement/tests/parse.rs` |
| macOS capacity runner (`81059c9df`, `bcac69311`, `5936e895e`) | Stop a create failing "Resource busy" right after a detach | Yes, differently: one image per suite, copied per case, so no create follows a detach; the busy-create tests are there under the template names |
| `agent/comparison-lab` (`f9b4957a1`, `cb07c1515`, `d0e6c7f26`) | Lab adapter for RRFlow: payload index, TurboQuant, native HNSW | Yes. `payload_index_request`, the TurboQuant build and activation, and the native parameters from [the lab binding](../gate-j/lab-explicit-native-hnsw-20261119.md) |
| F-06 cold storage (`9dc7d8ac6`, `2be5aa805`, `a2b9021a1`) | Cold history with deadline handling and an oracle | Yes. `engine/cold_history/`; a deadline is typed by its cause, not by the clock afterwards |
| `agent/engram-build-repair-20261107` (`aa169c31f`, `d7ef0a13b`) | The rename, plus routing, flight binding, adapter fail-closed and selector identity | Code yes: all five items [the rename review](engram-rename-review-20261114.md) recorded as missing have since landed. **One test was absent** and is restored here |
| contract split (`cf4cefc3a`) | Split the contract's `lib.rs` into domain modules | Yes. `lib.rs` is 326 lines over 68 source entries |
| single helpers (`04182088e`, `c54cbc4b2`, `edd3c3e28`, `2d09e42e3`, `e305e80f7`, `eddb6e264`, `73c9cecd2`) | Test helpers, a measurement example, a probe constant, a misspelt rename | Nothing to carry. `only_an_advanced_cursor_gets_the_stale_transaction_action` and the generated client operations are on `main` |

## The one that was absent

`worker_adapter_profiles_are_typed_and_extensible`. `EndpointWorkerAdapter` on `main` names the SWAG and LFG chat adapters and an `Other` adapter bound by two schema digests, and the engine refuses to execute an adapter it does not implement. No contract test held the type. The restored test asserts that both named adapters validate and are native, that an `Other` adapter with two digests validates and is not native, and that an `Other` adapter with a malformed digest is refused.

## Noted, not ported

The Parse executor draft retried a physical write conflict three times inside one Step. `main` has no such retry and no test shows it is needed; a conflict fails the Step and the job runs it again.

## Evidence

Windows 11, one run each.

| Command | Result |
|---|---|
| `cargo test -p rrd-contract --test endpoint_worker --locked` | 6 passed |

## Changed paths

- `crates/transport/rrd-contract/tests/endpoint_worker.rs`
- this record and its index

## Not proven

- A definition is looked up by name, then by reading. A behaviour inside a function whose name survived, and that no test holds, is not found by this pass.
- Linux and macOS are for the candidate gate.

## Roadmap and POA&M

No status change.
