#!/usr/bin/env python3
"""Exercise the documented example against an owned installed RRFlow process."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import threading
import time

OUTPUT_LIMIT = 16 * 1024 * 1024


class Child:
    """Bound and reap exactly the process this test started, including on failure."""
    def __init__(self, args, label, limit=OUTPUT_LIMIT):
        self.label = label
        self.limit = limit
        self.output = [bytearray(), bytearray()]
        self.exceeded = threading.Event()
        self.read_failed = threading.Event()
        self.lock = threading.Lock()
        self.process = subprocess.Popen(
            [str(item) for item in args], stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0,
        )
        self.threads = []
        for index, pipe in enumerate((self.process.stdout, self.process.stderr)):
            def read(source=pipe, slot=index):
                try:
                    while chunk := os.read(source.fileno(), 65536):
                        with self.lock:
                            room = self.limit - len(self.output[slot])
                            self.output[slot].extend(chunk[:max(0, room)])
                            if len(chunk) > room:
                                self.exceeded.set()
                                break
                except OSError:
                    self.read_failed.set()
                finally:
                    source.close()
            thread = threading.Thread(target=read, daemon=True)
            self.threads.append(thread)
            thread.start()

    def stop(self):
        if self.process.poll() is None:
            try:
                self.process.kill()
            except OSError:
                if self.process.poll() is None:
                    raise
        self.process.wait(timeout=10)
        for thread in self.threads:
            thread.join(timeout=10)
        if any(thread.is_alive() for thread in self.threads):
            raise RuntimeError(f"{self.label}: capture did not terminate")
        if self.read_failed.is_set():
            raise RuntimeError(f"{self.label}: capture failed")
        if self.exceeded.is_set():
            raise RuntimeError(f"{self.label}: output limit exceeded")

    def finish(self, timeout):
        deadline = time.monotonic() + timeout
        try:
            while self.process.poll() is None:
                if self.exceeded.is_set():
                    raise RuntimeError(f"{self.label}: output limit exceeded")
                if time.monotonic() >= deadline:
                    raise RuntimeError(f"{self.label}: deadline exceeded")
                time.sleep(0.02)
        finally:
            self.stop()
        if self.exceeded.is_set():
            raise RuntimeError(f"{self.label}: output limit exceeded")
        return self.process.returncode, bytes(self.output[0]), bytes(self.output[1])

    def announcement(self, timeout=60):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if self.exceeded.is_set():
                raise RuntimeError(f"{self.label}: output limit exceeded")
            with self.lock:
                first = bytes(self.output[0]).partition(b"\n")
            if first[1]:
                return json.loads(first[0])
            if self.process.poll() is not None:
                raise RuntimeError(f"{self.label}: exited before listening")
            time.sleep(0.02)
        raise RuntimeError(f"{self.label}: listening deadline exceeded")


def command(args, label, timeout=120, limit=OUTPUT_LIMIT, expect_failure=False, expected_error=None):
    child = Child(args, label, limit)
    code, stdout, stderr = child.finish(timeout)
    if (code == 0) == expect_failure:
        stderr_digest = hashlib.sha256(stderr).hexdigest()
        raise RuntimeError(f"{label}: {'unexpected success' if expect_failure else 'failed'} "
                           f"(exit {code}; stderr bytes={len(stderr)} sha256={stderr_digest})")
    if expected_error is not None and expected_error not in stderr:
        raise RuntimeError(f"{label}: failed for an unexpected reason")
    return stdout


def check_view(view, expected, instance=None):
    tasks, relations = view["tasks"], view["relationships"]
    if (not tasks["scope"].startswith("instance:") or tasks["scope"] != relations["scope"]
            or tasks["known_at_cursor"] != relations["known_at_cursor"]
            or tasks["schema_revision"] != relations["schema_revision"]):
        raise ValueError("database view has inconsistent scope or stamps")
    if instance is not None and tasks["scope"] != f"instance:{instance}":
        raise ValueError("database view differs from the served instance")
    for result in (tasks, relations):
        if result["execution"]["truncated"] is not False or result["execution"]["returned_rows"] != 1 or len(result["rows"]) != 1:
            raise ValueError("database view is incomplete")
    value = lambda text: {"type": "string", "value": text}
    if tasks["rows"][0]["values"] != {"id": value("first-task"), "title": value("Build with rrflowDB"), "status": value(expected)}:
        raise ValueError("database task differs from independent expected state")
    if relations["rows"][0]["values"] != {"left.name": value("Local AI project"), "right.to_id": value("first-task")}:
        raise ValueError("database relationship differs from independent expected state")
    if view["verified_expected_status"] != expected:
        raise ValueError("example did not report expected verification")


def sha256(path):
    with path.open("rb") as file:
        return hashlib.file_digest(file, "sha256").hexdigest()


def stage(source, destination):
    shutil.copyfile(source, destination)
    shutil.copymode(source, destination)
    if destination.stat().st_nlink != 1 or sha256(source) != sha256(destination):
        raise RuntimeError("staged executable identity or link count differs")


def run(primary, example):
    temp_root = Path(tempfile.gettempdir()).resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix="rrflowdb-standalone-", dir=temp_root) as directory:
        root = Path(directory).resolve(strict=True)
        if root.parent != temp_root or not root.name.startswith("rrflowdb-standalone-"):
            raise RuntimeError("temporary fixture escaped its owned root")
        project = root / "project"
        project.mkdir()
        binary = root / ("rrflow.exe" if os.name == "nt" else "rrflow")
        app = root / ("rrflowdb.exe" if os.name == "nt" else "rrflowdb")
        stage(primary, binary)
        stage(example, app)
        identity = json.loads(command([binary, "install", "identity", "--json"], "identity"))
        plan = command([binary, "install", "plan", "--project", project, "--identity",
                        identity["installation_identity"], "--mode", "fresh", "--json"], "plan")
        plan_file = root / "plan.json"
        plan_file.write_bytes(plan)
        digest = json.loads(plan)["installation"]["plan_sha256"]
        command([binary, "install", "apply", "--project", project, "--mode", "fresh",
                 "--plan", plan_file, "--expect", digest], "apply")

        def serve():
            child = Child([binary, "--json", "serve", "--project", project, "--bind", "127.0.0.1:0"], "serve")
            try:
                announcement = child.announcement()
                url = announcement["url"]
                if announcement["status"] != "listening" or not url.startswith("http://127.0.0.1:"):
                    raise RuntimeError("serve: invalid listening address")
                instance = announcement["instance_id"]
                if not isinstance(instance, str) or not instance:
                    raise RuntimeError("serve: invalid instance identity")
                return child, url.removeprefix("http://"), instance
            except BaseException:
                child.stop()
                raise

        child, address, instance = serve()
        try:
            command([binary, "ready", "--project", project, "--address", address], "ready")
            command([app, project, address, "init"], "initialize")
            before = json.loads(command([app, project, address, "verify", "open"], "verify open"))
            check_view(before, "open", instance)
            command([app, project, address, "read"], "read")
            command([app, project, address, "update"], "update")
            aborted = json.loads(command([app, project, address, "abort"], "abort"))
            if aborted != {"state": "aborted", "data_changed": False}:
                raise ValueError("abort did not report the expected unchanged data")
            after = json.loads(command([app, project, address, "verify", "done"], "verify done"))
            check_view(after, "done", instance)
            command([app, project, address, "verify", "open"], "reject old status", expect_failure=True,
                    expected_error=b"database task does not match the expected example state")
            command([app, project, address, "schema"], "schema")
        finally:
            child.stop()
        command([binary, "verify", "--project", project, "--level", "quick"], "offline verify")
        child, address, reopened_instance = serve()
        try:
            if reopened_instance != instance:
                raise ValueError("reopened instance identity differs")
            reopened = json.loads(command([app, project, address, "verify", "done"], "verify reopened"))
            check_view(reopened, "done", instance)
            if reopened["tasks"]["known_at_cursor"] != after["tasks"]["known_at_cursor"]:
                raise ValueError("reopened database cursor differs")
        finally:
            child.stop()
        print(json.dumps({"standalone_rrflowdb": "passed", "primary_sha256": sha256(binary),
                          "example_sha256": sha256(app), "cursor": reopened["tasks"]["known_at_cursor"],
                          "scope": "synthetic fresh project, native real processes; not capacity qualification"}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("primary", type=Path)
    parser.add_argument("example", type=Path)
    args = parser.parse_args()
    run(args.primary.resolve(strict=True), args.example.resolve(strict=True))


if __name__ == "__main__":
    main()
