"""Optional real-primary Python attunement acceptance on a disposable estate."""

from __future__ import annotations

import hashlib
import json
import os
import queue
import shutil
import subprocess
import tempfile
import threading
import time
import tomllib
from collections.abc import Iterator
from pathlib import Path
from typing import Any

import pytest

from rrd_client import RequestOptions, RrdApiError, RrdClient, Session


@pytest.fixture
def owned_root(tmp_path: Path) -> Iterator[Path]:
    with tempfile.TemporaryDirectory(prefix="rrflow-sdk-", dir=tmp_path) as directory:
        yield Path(directory)


@pytest.mark.skipif(not os.environ.get("RRFLOW_TEST_BINARY"), reason="real primary not provided")
def test_installed_python_stale_plan_replans_and_starts(owned_root: Path) -> None:
    built_binary = Path(os.environ["RRFLOW_TEST_BINARY"]).resolve(strict=True)
    binary = owned_root / built_binary.name
    shutil.copy2(built_binary, binary)
    with built_binary.open("rb") as source_file, binary.open("rb") as copied_file:
        assert (
            hashlib.file_digest(source_file, "sha256").digest()
            == hashlib.file_digest(copied_file, "sha256").digest()
        )
    project = owned_root / "project"
    project.mkdir()
    source = project / "source.txt"
    source.write_text("initial source\n", encoding="utf-8")

    def cli(*args: str) -> dict[str, Any]:
        result = subprocess.run(
            [str(binary), *args], check=True, capture_output=True, text=True, timeout=120
        )
        return json.loads(result.stdout)

    identity = cli("install", "identity")["installation_identity"]
    preview = cli(
        "--json",
        "install",
        "plan",
        "--identity",
        identity,
        "--mode",
        "existing",
        "--project",
        str(project),
    )
    preview_file = owned_root / "plan.json"
    preview_file.write_text(json.dumps(preview), encoding="utf-8")
    cli(
        "--json",
        "install",
        "apply",
        "--mode",
        "existing",
        "--project",
        str(project),
        "--plan",
        str(preview_file),
        "--expect",
        preview["installation"]["plan_sha256"],
    )
    plan = preview["attunement"]
    job_id = "job-" + plan["plan_sha256"][:24]
    locator = tomllib.loads((project / ".rrflow" / "config.toml").read_text(encoding="utf-8"))
    credential = json.loads((project / locator["operator_credential"]).read_text(encoding="utf-8"))
    source.write_text("changed source\n", encoding="utf-8")
    lines: queue.Queue[str] = queue.Queue(maxsize=1)
    with (owned_root / "server-stderr.log").open("wb") as stderr:
        process = subprocess.Popen(
            [str(binary), "--json", "serve", "--project", str(project), "--bind", "127.0.0.1:0"],
            stdout=subprocess.PIPE,
            stderr=stderr,
            text=True,
        )
        assert process.stdout is not None
        threading.Thread(target=lambda: lines.put(process.stdout.readline()), daemon=True).start()
        try:
            announcement = json.loads(lines.get(timeout=60))
            assert announcement["status"] == "listening"
            with RrdClient(
                announcement["url"], announcement["instance_id"], max_attempts=1
            ) as client:

                def options(
                    name: str, *, session: Session | None = None, mutation: bool = False
                ) -> RequestOptions:
                    return RequestOptions(
                        request_id=f"python-live-{name}",
                        operation_id=f"python-live-{name}",
                        idempotency_key=f"python-live-{name}" if mutation else None,
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
                            "max_open_transactions": 4,
                        }
                    },
                    options("session", mutation=True),
                )
                start = {
                    "contract_version": 1,
                    "job_id": job_id,
                    "plan_sha256": plan["plan_sha256"],
                    "expected_revision": 1,
                    "lease_ms": 30000,
                }
                with pytest.raises(RrdApiError) as captured:
                    client.call(
                        "attunement-start", start, options("stale", session=session, mutation=True)
                    )
                assert captured.value.status == 412
                assert captured.value.code == "failed_precondition"
                assert captured.value.retryable is False
                assert captured.value.details == {
                    "reason": "attunement-source-changed",
                    "next-action": "preview-attunement-replan",
                }
                before = client.call(
                    "attunement-read",
                    {"contract_version": 1, "job_id": job_id},
                    options("read", session=session),
                )
                assert before["status"]["job"]["revision"] == 1
                replan = client.call(
                    "attunement-replan-preview",
                    {"contract_version": 1, "job_id": job_id, "expected_revision": 1},
                    options("preview", session=session),
                )
                accepted = client.call(
                    "attunement-replan-accept",
                    {
                        "contract_version": 1,
                        "preview": replan,
                        "proposal_sha256": replan["proposal_sha256"],
                    },
                    options("accept", session=session, mutation=True),
                )
                assert accepted["predecessor"]["job"]["state"] == "cancel_requested"
                assert accepted["plan"]["source_sha256"] != plan["source_sha256"]
                successor = accepted["successor"]["job"]
                new_start = {
                    "contract_version": 1,
                    "job_id": successor["id"],
                    "plan_sha256": accepted["plan"]["plan_sha256"],
                    "expected_revision": successor["revision"],
                    "lease_ms": 30000,
                }
                started = client.call(
                    "attunement-start",
                    new_start,
                    options("new-start", session=session, mutation=True),
                )
                assert started["status"]["job"]["state"] == "running"

            process.terminate()
            process.wait(timeout=30)
            process.stdout.close()
            lines = queue.Queue(maxsize=1)
            process = subprocess.Popen(
                [
                    str(binary),
                    "--json",
                    "serve",
                    "--project",
                    str(project),
                    "--bind",
                    "127.0.0.1:0",
                ],
                stdout=subprocess.PIPE,
                stderr=stderr,
                text=True,
            )
            assert process.stdout is not None
            threading.Thread(
                target=lambda: lines.put(process.stdout.readline()), daemon=True
            ).start()
            reopened = json.loads(lines.get(timeout=60))
            assert reopened["status"] == "listening"
            assert reopened["instance_id"] == announcement["instance_id"]
            with RrdClient(reopened["url"], reopened["instance_id"], max_attempts=1) as client:
                receipt = client.call(
                    "attunement-replan-receipt-read",
                    {
                        "contract_version": 1,
                        "job_id": job_id,
                        "idempotency_key": "python-live-accept",
                    },
                    options("receipt", session=session),
                )
                assert receipt["receipt"]["result"]["successor"]["job"]["id"] == successor["id"]
                replay = client.call(
                    "attunement-replan-accept",
                    {
                        "contract_version": 1,
                        "preview": replan,
                        "proposal_sha256": replan["proposal_sha256"],
                    },
                    options("accept", session=session, mutation=True),
                )
                assert replay["idempotent_replay"] is True
                assert replay["successor"]["job"]["id"] == successor["id"]
                audit = client.call(
                    "audit-read",
                    {"after_sequence": 0, "limit": 1024},
                    options("audit", session=session),
                )
                completed = {
                    (record["request_id"], record["status_code"])
                    for record in audit["records"]
                    if record["phase"] == "completed"
                }
                assert ("python-live-stale", 412) in completed
                assert ("python-live-accept", 200) in completed
        finally:
            if process.poll() is None:
                process.terminate()
            process.wait(timeout=30)
            if process.stdout is not None:
                process.stdout.close()
