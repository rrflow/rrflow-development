# Clean controller primary: installed SDK replan evidence

**Status:** active engineering-binary evidence; native, worker and release qualification pending
**Coordinate:** `rrflow://rrflow-instance/data/evidence/change-journal/gate-g/controller-publication-installed-sdk`
**Owner:** Gate G/I controller composition; driver: active SWAG session; PR206

The [integrated source validation](controller-publication-integration.md) is now
committed as `9e167aad55323c964d436a030745900e9f81c0ab`, tree
`90a7e59ce053e8038fe27753237d91c9a3076cbf`. Its ordinary parents are controller
`c5bd4d99` and exact publication repair `a40a4f8c`. Only one documentation
conflict required resolution; native source-observation exports remain intact.

From that clean source, `cargo build -p rrflow-cli --bin rrflow --locked`
completed with two build workers and dev/test debug information disabled.
The executable was independently copied, checked for one hard link and hashed:

- `controller-publication-rrflow.exe`: 153,362,944 bytes.
- SHA-256: `f0f35eacf4e59fc847aa7016906b54359ca73fb3c18a8dc5eb261f7779ad4f92`.
- Source-envelope SHA-256:
  `1cfe73d61757dbb2b8abc32737ade0b70f21a8b71037e5299795fd74188ccced`.

The actual copied executable passed both Python SDK installed attunement cases:
`RRFLOW_TEST_BINARY=<frozen executable> uv --directory sdks/python run --frozen
pytest -o addopts= --color=no -q tests/test_installed_attunement.py` reports
**two passed in 7.73 seconds**, with no skips. These use disposable estates and
exercise stale-source replan/start and selection after prior progress. The test
verifies its executable copy's digest. No model, worker or training operation
was invoked; credentials stay within private test estates and process memory.
SDK log SHA-256:
`9cc0020d79762aa5fa806774b5c1980d649686fd377ed7fb1a735636771c23c5`.
The primary's compact-JSON contract digest `12fd7107` matches independently
compacted exporter output; the formatted export retains `f1856602`. These are
different serializations of the same verified schema, not interchangeable hashes.

Raw build log, source envelope, validation logs and executable are retained
under `D:/projects/swag/.swag/evidence/20261030/engine-source-inventory/`.
The builder is `build-controller-publication-primary.py`; it requires the exact
clean revision and refuses existing output artifacts. This evidence-only record
and generated index do not change source behavior or reassign that binary to a
later documentation commit. Documentation/navigation, the 2,160-record execution
inventory and whitespace checks pass before publication.

This is an unoptimized, unqualified Windows engineering build. The prior
1,870-path installed Inventory trial belongs to PR227's original binary and has
not been repeated against this executable. Real endpoint/worker completion,
RROFLOW delivery, independently reviewed model quality, upgrades, sustained
workload, native candidate/protected CI and public binary/container release
remain separate acceptance. No roadmap gate or dependency is declared landed.
