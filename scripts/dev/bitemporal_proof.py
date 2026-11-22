#!/usr/bin/env python3
"""Install an owned project, commit/correct/query via the public SDK, kill/reopen.

Run with the repository Python SDK environment:
  uv run --project sdks/python python scripts/dev/bitemporal_proof.py \
      --engine <rrflow executable> --output <new proof directory>

The output must not exist. The retained project can subsequently be opened in
Connectome using the same engine build. No provider, cloud or user project is
activated. Only public CLI/HTTP operations author the database.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import queue
import shutil
import subprocess
import sys
import threading
import time
import tomllib
from contextlib import contextmanager
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "sdks/python/src"))
from rrd_client import RequestOptions, RrdClient


def save(path: Path, value: object) -> None:
    path.write_bytes((json.dumps(value, indent=2) + "\n").encode("utf-8"))


def cli(engine: Path, *args: str) -> dict:
    result = subprocess.run(
        [str(engine), "--json", *args],
        check=True,
        capture_output=True,
        timeout=120,
    )
    return json.loads(result.stdout)


@contextmanager
def serving(engine: Path, project: Path, output: Path, phase: str):
    """Own this child only; force termination exercises acknowledged WAL recovery."""
    lines: queue.Queue[str] = queue.Queue(maxsize=1)
    with (output / f"{phase}-stderr.log").open("wb") as log:
        process = subprocess.Popen(
            [
                str(engine),
                "--json",
                "serve",
                "--project",
                str(project),
                "--bind",
                "127.0.0.1:0",
            ],
            stdout=subprocess.PIPE,
            stderr=log,
            text=True,
        )
        try:
            threading.Thread(
                target=lambda: lines.put(process.stdout.readline()), daemon=True
            ).start()
            announced = json.loads(lines.get(timeout=60))
            if announced.get("status") != "listening":
                raise RuntimeError("engine did not announce a listening endpoint")
            save(output / f"{phase}-endpoint.json", announced)
            yield announced
        finally:
            if process.poll() is None:
                process.kill()
            process.wait(timeout=30)
            if process.stdout:
                process.stdout.close()


def prove(engine: Path, output: Path) -> None:
    output.mkdir(parents=True, exist_ok=False)
    owned_engine = output / engine.name
    shutil.copy2(engine, owned_engine)
    with owned_engine.open("rb") as executable:
        engine_hash = hashlib.file_digest(executable, "sha256").hexdigest()
    project = output / "project"
    project.mkdir()
    identity = cli(owned_engine, "install", "identity")["installation_identity"]
    preview = cli(
        owned_engine,
        "install",
        "plan",
        "--project",
        str(project),
        "--mode",
        "fresh",
        "--identity",
        identity,
    )
    plan_file = output / "plan.json"
    save(plan_file, preview)
    cli(
        owned_engine,
        "install",
        "apply",
        "--project",
        str(project),
        "--mode",
        "fresh",
        "--plan",
        str(plan_file),
        "--expect",
        preview["installation"]["plan_sha256"],
    )
    locator = tomllib.loads(
        (project / ".rrflow/config.toml").read_text(encoding="utf-8")
    )
    # Credentials stay local to this process and are never included in evidence.
    credential = json.loads(
        (project / locator["operator_credential"]).read_text(encoding="utf-8")
    )
    evidence: dict = {
        "engine_sha256": engine_hash,
        "version": cli(owned_engine, "version"),
        "project": str(project),
        "observations": [],
    }
    correction = None
    old_cursor = new_cursor = 0
    for phase in ("initial", "reopened"):
        with serving(owned_engine, project, output, phase) as endpoint:
            instance = endpoint["instance_id"]
            scope = f"instance:{instance}"
            with RrdClient(endpoint["url"], instance, max_attempts=1) as client:
                session = None

                def options(
                    name: str, mutation: bool = False, *, phase=phase, session=None
                ) -> RequestOptions:
                    return RequestOptions(
                        request_id=f"proof-{phase}-{name}",
                        operation_id=f"proof-{phase}-{name}",
                        idempotency_key=f"proof-{name}" if mutation else None,
                        deadline_unix_ms=int(time.time() * 1000) + 60_000,
                        session=session,
                    )

                session = client.create_session(
                    credential["principal_id"],
                    credential["credential"],
                    {
                        "limits": {
                            "idle_timeout_ms": 60000,
                            "absolute_timeout_ms": 300000,
                            "max_open_transactions": 8,
                        }
                    },
                    options("session", True),
                )

                def call(
                    operation: str,
                    payload: dict,
                    name: str,
                    mutation: bool = False,
                    *,
                    client=client,
                    session=session,
                ):
                    return client.call(
                        operation, payload, options(name, mutation, session=session)
                    )

                def program(
                    bindings: dict, disposition: str = "COMMIT", *, scope=scope
                ) -> dict:
                    return {
                        "scope": scope,
                        "program": "BEGIN; "
                        + " ".join(f"MUTATE ${name};" for name in bindings)
                        + f" {disposition};",
                        "mutation_bindings": bindings,
                        "timeout_ms": 30000,
                    }

                def document(identifier: str, title: str) -> dict:
                    return {
                        "mutation": "put_record",
                        "reference": {"kind": "proof_document", "id": identifier},
                        "valid_from": 100,
                        "properties": {"title": {"type": "string", "value": title}},
                    }

                if phase == "initial":
                    catalogue = call(
                        "data-schema-read",
                        {"scope": scope, "max_storage_keys": 1024},
                        "schema",
                    )
                    schema = catalogue["schema"] or {
                        "revision": 0,
                        "tables": {},
                        "records": {},
                        "relations": {},
                        "events": {},
                    }
                    schema["revision"] += 1
                    schema["migration"] = "installed bitemporal proof schema"
                    schema["tables"].update(
                        {
                            "proof_document": {
                                "model": "document",
                                "mode": "schemaless",
                            },
                            "proof_link": {"model": "graph_relation", "mode": "strict"},
                        }
                    )
                    schema["relations"]["proof_link"] = {
                        "from": ["proof_document"],
                        "to": ["proof_document"],
                        "properties": {},
                        "allow_additional_properties": True,
                        "unique_pair": True,
                    }
                    original = program(
                        {
                            "schema": {"mutation": "put_schema", "registry": schema},
                            "first": document("alpha", "Original"),
                            "second": document("beta", "Evidence"),
                            "edge": {
                                "mutation": "put_relation",
                                "reference": {"kind": "proof_link", "id": "evidence"},
                                "from": {"kind": "proof_document", "id": "alpha"},
                                "to": {"kind": "proof_document", "id": "beta"},
                                "valid_from": 100,
                                "properties": {},
                            },
                        }
                    )
                    committed = call(
                        "query-transaction-execute", original, "original", True
                    )
                    old_cursor = committed["commit"]["last_runtime_cursor"]
                    correction = program({"correction": document("alpha", "Corrected")})
                    corrected = call(
                        "query-transaction-execute", correction, "correction", True
                    )
                    new_cursor = corrected["commit"]["last_runtime_cursor"]
                    canceled = call(
                        "query-transaction-execute",
                        program({"cancel": document("alpha", "Wrong")}, "CANCEL"),
                        "cancel",
                        True,
                    )
                    assert (
                        canceled["state"] == "aborted"
                        and canceled.get("commit") is None
                    )
                    evidence.update(
                        {
                            "original": committed,
                            "correction": corrected,
                            "cancel": canceled,
                            "old_cursor": old_cursor,
                            "new_cursor": new_cursor,
                        }
                    )
                    save(
                        output / "requests.json",
                        {"original": original, "correction": correction},
                    )
                else:
                    replay = call(
                        "query-transaction-execute", correction, "correction", True
                    )
                    assert replay["commit"]["idempotent_replay"]
                    assert replay["commit"]["last_runtime_cursor"] == new_cursor
                    evidence["replay_after_termination"] = replay

                for label, valid, cursor, expected in (
                    ("old", 100, old_cursor, "Original"),
                    ("new", 100, new_cursor, "Corrected"),
                    ("before_valid", 99, new_cursor, None),
                ):
                    query = f'FROM record:proof_document AT VALID {valid} KNOWN {cursor} WHERE id = "alpha" PROJECT title EXPLAIN CONTRACT'
                    result = call(
                        "query-execute",
                        {
                            "scope": scope,
                            "query": query,
                            "parameters": {},
                            "budget": {
                                "max_storage_keys": 1024,
                                "max_rows": 10,
                                "max_output_bytes": 16384,
                                "max_batch_rows": 10,
                            },
                        },
                        label,
                    )
                    actual = (
                        result["rows"][0]["values"]["title"]["value"]
                        if result["rows"]
                        else None
                    )
                    assert actual == expected, (phase, label, result)
                    assert result["known_at_cursor"] == cursor
                    evidence["observations"].append(
                        {
                            "phase": phase,
                            "label": label,
                            "query": query,
                            "result": result,
                        }
                    )
                edges = call(
                    "query-execute",
                    {
                        "scope": scope,
                        "query": f"FROM relation:proof_link AT VALID 100 KNOWN {new_cursor} EXPLAIN CONTRACT",
                        "parameters": {},
                        "budget": {
                            "max_storage_keys": 1024,
                            "max_rows": 10,
                            "max_output_bytes": 16384,
                            "max_batch_rows": 10,
                        },
                    },
                    "edges",
                )
                assert len(edges["rows"]) == 1
                evidence["observations"].append(
                    {"phase": phase, "label": "edge", "result": edges}
                )
    evidence["passed"] = True
    save(output / "evidence.json", evidence)
    print(
        json.dumps(
            {
                "passed": True,
                "project": str(project),
                "old_cursor": old_cursor,
                "new_cursor": new_cursor,
                "evidence": str(output / "evidence.json"),
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    prove(args.engine.resolve(strict=True), args.output.resolve())
