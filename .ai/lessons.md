# Lessons

Each entry is something that cost the owner time, what the repository shows about it, and the rule that replaces it. Newest first.

## 2026-11-26: the first alignment plan was wrong in shape and in method

The plan was section 6 of `docs/architecture/code-structure-and-conventions.md`, written 2026-11-25.

1. **It staged the work like a released product.** Fourteen packages, one pull request each, a nightly between them, renames held back until after splits. After a day, one crate of twelve had its name (`rrd-lsm` → `rrflow-lsm`) and `rrd-store` was still everywhere. This is pre-release. There is no compatibility to protect, so a rename happens in one pass and the errors it surfaces are fixed in the same branch.
2. **It invented a `foundation` tier.** Commit `17914bb92` (2026-11-25) moved the kernel crate to `crates/foundation/rrflow-kernel` and the plan built a tier around it. Nobody asked for a foundation tier. The name described nothing and gave a grab-bag crate a permanent home.
3. **It never asked what the kernel was.** Measured on 2026-11-26, the 12,321-line "kernel" is not a kernel. 6,200 lines are the database's record and schema model, 1,180 lines are used only by the store, 1,451 lines are used only by the contract (which is why the reasoning tree exists twice), and LSM takes only a 249-line digest module from it. The crate existed so that anything could reach anything.
4. **An earlier "absorb" was cosmetic.** Commit `6e4fdbb6b` (2026-11-22) answered "absorb the kernel into the engine" by moving the crate's folder to `crates/authority/rrd-engine/kernel`. It stayed a separate crate. A crate nested in another crate's folder is not part of that crate in Rust; the move satisfied the words and changed nothing.
5. **Its measurement of the store was wrong.** The plan said no store module depended on another upward. A text scan of `crate::` paths missed re-exports at the crate root. In fact the storage port returned the six repositories, and the key-value code called the semantic commit planner, so the key-value layer depended on the layers above it.
6. **It put the layers in a different order from the owner's.** The plan had KV below TX below DB, and made rrflowMX the agent memory layer. The owner's table (in the plan file) has TX below KV, MX as the in-memory leaf, and DB on top.

**Rules that replace it**

- Pre-release: rename and move in one pass; let the compiler and the tests show what broke; fix it there. No alias, re-export shim, forwarding module or second path is left behind, and no step is held back to keep an old name alive.
- A folder or crate name must describe what is in it. If the honest description is "things other crates need", the contents belong to their owners instead.
- Moving a folder is not absorbing. Code is part of a crate when it is a module of that crate.
- Before a crate is moved, renamed or kept, measure who uses each part of it. A plan states the measurement and how it was taken.
- A dependency measurement comes from the compiler (`cargo metadata`, or errors after the move), not from a text scan.
- The owner's component table is the layering. A plan that needs a different order says so in its first line and asks.

## 2026-11-26: reporting

- "Nothing merged" was the owner's view because the local checkout sat on a feature branch with `main` behind, and the official repository never receives pushes. A report says where each change is: the development repository's `main`, the local checkout, the official repository.
- The Connectome alarm hook prints on every message. It is a missing macOS runner on an old run, not a code failure, and the owner ruled that Connectome is not worked on until the engine is reorganized. It is mentioned once, not every reply.
- The macOS runner has still not been made to run reliably and the macOS code is not trusted. That is its own phase, after the restructure.

## Standing

- `origin` is the official repository and is never pushed. The only push target is `development`. No force push, no history rewrite, no stash.
- RRD is the daemon: the running service. `rrd-` names the daemon's own crates and nothing else.
- Proposing a layout the owner did not ask for costs more than asking one question. Asking a question the records already answer costs more than reading them.
