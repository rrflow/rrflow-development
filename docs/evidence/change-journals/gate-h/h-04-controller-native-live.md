# H-04: live native client controller proposal

**Status:** active installed engineering evidence; UI and protected landing pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-h/h-04-controller-native-live`
**Owner:** H-04 native client; G-01 experimental proposal admission
**Pull request:** [222](https://github.com/rrflow/rrflow-development/pull/222)

## Exact inputs and implementation boundary

The [implementation](h-04-controller-native-binding.md) was committed and published as `dccfc41962d6ff998c9e309ebd550d8f5fadcbdf`, tree `2cc11b77712919b7044870302e72944d228ee50f`. A clean-tree `cargo build --locked -p rrd-client --example controller_proposal` produced the native client SHA256 `7eb1ff25d0c3a60492d81e226f1032eda2da3b5d088a05581d55889178c0a72c`. This is an unqualified development example, not a release binary or canonical primary-build receipt.

The installed primary executable is the existing optimized engineering build from `54fb845dffd702e1504c7f9a0cc2a3c5757d2bf2`, SHA256 `fb9b5727e6ea0de668c723e63d008bfd976a02ca020aad218b122b61142afee0`. Its unchanged build receipt was checked by the proof driver. The client and engine revisions are intentionally distinct: this proves this operation across that exact pair, not full cross-version catalogue or distribution compatibility.

The input task is the actual public PR222 maintenance title/body fetched after implementation publication. Its bounded task-text projection and source digest are retained. The existing operator-supplied endpoint profile, credential reference and 60,000 ms timeout were used. There was no invented routing example, label, review or approval. The endpoint health/model discovery was HTTP 200 and observed idle before dispatch; its reported alias does not attest loaded weight bytes.

## Actual operation and restart evidence

The existing installed proof driver ran in a fresh estate with `--native-client` pointing to the hashed Rust example and an explicit timeout. Operator credentials remained native; provider credentials were supplied only to the owned engine process through the existing explicit mapping. The Rust example read the installed locator/credential, authenticated, called the generated binding once and closed its own session. Its typed output is retained as typed output, never relabelled as raw HTTP capture. The original Python proof path performed public setup/readback/replay checks; it did not dispatch the authorized proposal.

The native call returned an accepted experimental proposal from the actual configured BF16 endpoint: successful HTTP status in the receipt, 1,731 reported prompt tokens, 20 completion tokens and 1,881 ms adapter elapsed time. These are a single observed call's fields, not a latency benchmark or quality evaluation. Attempt `actual-proposal-22e61c8cbefa44bea629fc5ba4af7bc9` retained one spent step. Public attempt/budget readback after restarting the owned engine was identical. Reusing the attempt was denied with 412 and no second accepted proposal. Missing credential and unauthenticated requests were denied before the authorized call. Native exit was zero and session close succeeded. The proof driver terminated its owned server processes in `finally`; the authorized and reopened Windows process receipts both record exit code 1. This proves termination/reopen readback, not graceful server shutdown. A postflight assertion initially expected exit zero; inspecting the actual receipts and existing `process.terminate()` path corrected that reporting assumption without rerunning a provider call.

Sixteen exact public receipts and their [digest manifest](controller-native-client-receipts/manifest.json) are retained. Provider/operator credential bytes and bearer/API-key headers were checked absent before copying. Captured CRLF bytes are preserved with scoped attributes. No earlier trial artifact or uncertain estate was modified. This journal and receipts are the only follow-up changes; navigation/inventory and repository checks accompany publication.

## Scope, review and remaining gates

The PR brief authorized the fresh native proof before implementation. Reviewed the native output, actual attempt/budget receipts before and after restart, replay denial, client build identity, preflight, process exits and manifest hashes. No external research, new model training, renderer implementation, worker execution or project-source mutation occurred. No retry, output repair or invented acceptance label was needed. The provisional model remains unqualified; a one-candidate proposal cannot establish selection quality.

The owner-supplied complete project task and independent outcome review remain pending. Connectome integration still requires its own native/UI changes and pinned-client qualification. Development-task plan/worker execution, ordinary-edit successor planning, RROFLOW outcomes, independently reviewed learning, native/container release and measured quantization remain open. PR206/222 must pass their dependency and protected CI gates; the shared macOS APFS failure is not cleared by this Windows client trial. No roadmap or POAM completion status changes.
