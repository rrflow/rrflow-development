# Try RRFlow from source

**Status:** active guide; pre-alpha; verified on Windows x86_64 in Git Bash and on Linux aarch64 at `1fd7f163`, not yet on macOS
**Coordinate:** `rrflow://rrflow-instance/data/guide/getting-started`
**Owner:** the first-run path for someone trying RRFlow: build it, install it into a project folder, use it as a local database, and clean up

This guide takes you from a fresh clone to a running RRFlow engine holding real
data. It needs a few commands and one first build of a few minutes. What
RRFlow can and cannot do today is in the [README](../../README.md#try-rrflow).

The commands are bash. On Windows, run them in Git Bash. They were checked from
a clean tree on Windows x86_64 and Linux aarch64. macOS has not been checked
with this guide yet, though candidate CI runs the installed lifecycle on macOS.
If you prefer PowerShell 7, the
[local rrflowDB guide](../reference/sdk/rrflowdb-local.md) has the same steps.

## Before you start

- **Git** and **Rust through [rustup](https://rustup.rs/).** The repository pins
  its Rust version in `rust-toolchain.toml`, and rustup installs that version
  the first time you build.
- **A linker.** On Windows, the Visual Studio Build Tools with "Desktop
  development with C++". On Linux, gcc or clang. On macOS, the Xcode Command
  Line Tools.
- **About 15 GB of free disk.** A debug build took 11 GB here.

## 1. Get the source

```bash
git clone https://github.com/rrflow/rrflow-development.git rrflow
cd rrflow
```

Connectome, RRFlow's UI, is an optional submodule at `connectome/`. This guide
does not need it.

## 2. Build

```bash
cargo build --locked -p rrflow-cli --bin rrflow -p rrd-client --example rrflowdb
```

This builds `rrflow`, the one command you install and run RRFlow with, and
`rrflowdb`, a small example application that uses it as a database.

## 3. Give the binary a home of its own

A project you install records the exact `rrflow` binary that installed it, and
RRFlow refuses an executable with more than one hard link, which Cargo can
create in `target/`. So copy it to its own folder and use that copy from now on:

```bash
ext=; [ "${OS:-}" = Windows_NT ] && ext=.exe
mkdir -p "$HOME/rrflow-bin"
cp "target/debug/rrflow$ext" "$HOME/rrflow-bin/rrflow$ext"
rrflow="$HOME/rrflow-bin/rrflow$ext"
"$rrflow" version
```

You should see `RRFlow 1.0.0 (pre-release; rrd protocol v1)`. Keep this copy:
replacing it is not an upgrade, and the project stays bound to it.

## 4. Install RRFlow into a project folder

Start with an empty folder. Installing writes only `.rrflow/` and
`.rrflow-install.lock` into it.

```bash
project="$HOME/rrflow-try"
mkdir -p "$project"
identity=$("$rrflow" install identity --json | sed -n 's/.*"installation_identity": *"\([0-9a-f]*\)".*/\1/p')
"$rrflow" install plan --project "$project" --identity "$identity" --mode fresh --json > rrflow-plan.json
```

`install plan` is a preview. It writes nothing to the project, and
`rrflow-plan.json` lists exactly what installing will create, with its
configuration and permissions. Read it, then apply that exact plan:

```bash
plan_sha=$(sed -n 's/.*"plan_sha256": *"\([0-9a-f]*\)".*/\1/p' rrflow-plan.json | head -1)
"$rrflow" install apply --project "$project" --mode fresh --plan rrflow-plan.json --expect "$plan_sha"
```

The first `plan_sha256` in the file is the installation's own. `--expect` makes
`apply` refuse anything but the plan you read, so a wrong value fails safely.
It prints the new instance's name. To add RRFlow to a project that already has
files, use `--mode existing` in both commands. This guide was checked with
`fresh`.

## 5. Start it

```bash
"$rrflow" serve --project "$project" --bind 127.0.0.1:9477
```

`serve` stays in the foreground and prints `RRFlow listening at
http://127.0.0.1:9477`. Open a second terminal in the same `rrflow` folder, set
`ext`, `rrflow` and `project` the same way as above, and check that the engine
lets you in:

```bash
"$rrflow" ready --project "$project" --address 127.0.0.1:9477
```

You should see `HTTP operations: 78`. Any free loopback port works, as long as
every command uses the same one.

## 6. Use it as a database

```bash
example="target/debug/examples/rrflowdb$ext"
"$example" "$project" 127.0.0.1:9477 init
"$example" "$project" 127.0.0.1:9477 verify open
"$example" "$project" 127.0.0.1:9477 read
"$example" "$project" 127.0.0.1:9477 update
"$example" "$project" 127.0.0.1:9477 abort
"$example" "$project" 127.0.0.1:9477 verify done
"$example" "$project" 127.0.0.1:9477 schema
```

In order, these create a project, tasks, a document and their relationships in
one transaction; check what was created; query it with rrflowQL; change a task;
show that an aborted transaction changes nothing; check the change; and print
the schema. Each prints JSON, and a nonzero exit means that step failed. The
[local rrflowDB guide](../reference/sdk/rrflowdb-local.md) explains the data
model and the queries.

## 7. Check that it kept your data

Stop `serve` with Ctrl+C in its terminal. On Windows, if Ctrl+C in Git Bash does
not stop it within a few seconds, close that terminal window instead. Then:

```bash
"$rrflow" verify --project "$project" --level quick
```

You should see `checks: 13/13 passed`. Start `serve` again as in step 5, and in
the second terminal:

```bash
"$example" "$project" 127.0.0.1:9477 verify done
```

Exit status 0 means your data survived the restart.

## 8. Clean up

Stop `serve`, then remove what installing created:

```bash
rm -rf "$project/.rrflow" "$project/.rrflow-install.lock" rrflow-plan.json
```

There is no `uninstall` command yet. `.rrflow/credentials/` holds the local
operator credential, so never share the `.rrflow` folder or attach it to an
issue.

## Next steps

- Explore an installed project in the
  [native Connectome workbench](../reference/client/connectome.md#native-local-workbench),
  a development build of RRFlow's UI.
- Use another language through the [SDK reference](../reference/sdk/README.md).
- Every command, file and limit of an installed project is in the
  [installed lifecycle](../reference/deployment/installed-lifecycle.md).

## If something goes wrong

| You see | What to do |
|---|---|
| `install apply` or `serve` refuses the executable | Use the copy from step 3, not the binary in `target/`. |
| `serve` cannot bind its address | Another program has the port. Pick another and use it in every command. |
| `ready` or the example cannot connect | `serve` must be running in the other terminal. |
| PowerShell rejects `-Encoding utf8NoBOM` | That needs PowerShell 7. Use PowerShell 7, or the bash commands here. |
| The build fails linking, or runs out of space | Check the linker under "Before you start", and free about 15 GB. |
| Anything else | [Open an issue](https://github.com/rrflow/rrflow-development/issues/new/choose) with the output of `"$rrflow" version` and `git rev-parse HEAD`, your OS, the exact command and what it printed. |
