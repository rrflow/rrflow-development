"""Bounded, non-authoritative observations of an owned C-07 progress stream.

The campaign's own verified receipts remain authoritative. This reader never
copies event contents and never controls or retries the measured process.
"""
from __future__ import annotations

from collections import Counter
import json
import os
from pathlib import Path
import stat
import sys
import threading
import time

MAX_READ_BYTES = 2 * 1024 * 1024
MAX_LINE_BYTES = 8 * 1024 * 1024
KINDS = frozenset({
    "campaign", "corpus_intent", "corpus_acknowledged", "corpus_verified",
    "reopened_corpus_verified", "lifecycle_intent", "lifecycle_acknowledged",
    "load_intent", "load_outcome", "readback", "resource", "reopen",
    "reader_interference", "backup_started", "backup", "failure", "cold_file",
    "cold_start", "final_readback",
})


class ProgressError(RuntimeError):
    """A bounded observer refusal, without raw event or filesystem contents."""


class ProgressReader:
    def __init__(self, evidence: Path):
        self.evidence = evidence
        self.identity = None
        self.offset = 0
        self.pending = b""
        self.counts = Counter()

    def poll(self) -> tuple[bool, int, bool]:
        try:
            parent = self.evidence.lstat()
        except FileNotFoundError:
            if self.identity is not None:
                raise ProgressError("progress_directory_disappeared")
            return False, 0, False
        if not stat.S_ISDIR(parent.st_mode) or stat.S_ISLNK(parent.st_mode):
            raise ProgressError("progress_directory_invalid")
        path = self.evidence / "progress.jsonl"
        try:
            observed = path.lstat()
        except FileNotFoundError:
            if self.identity is not None:
                raise ProgressError("progress_file_disappeared")
            return False, 0, False
        if not stat.S_ISREG(observed.st_mode) or stat.S_ISLNK(observed.st_mode):
            raise ProgressError("progress_file_invalid")
        identity = (observed.st_dev, observed.st_ino)
        if not observed.st_ino:
            raise ProgressError("progress_identity_unavailable")
        if self.identity is not None and identity != self.identity:
            raise ProgressError("progress_file_replaced")
        if observed.st_size < self.offset:
            raise ProgressError("progress_file_truncated")
        flags = os.O_RDONLY | getattr(os, "O_BINARY", 0) | getattr(os, "O_NOFOLLOW", 0)
        try:
            fd = os.open(path, flags)
            with os.fdopen(fd, "rb") as source:
                opened = os.fstat(source.fileno())
                if (opened.st_dev, opened.st_ino) != identity:
                    raise ProgressError("progress_file_replaced")
                if opened.st_size < self.offset:
                    raise ProgressError("progress_file_truncated")
                source.seek(self.offset)
                block = source.read(MAX_READ_BYTES)
        except OSError as error:
            raise ProgressError("progress_read_failed") from error
        self.identity = identity
        self.offset += len(block)
        lines = (self.pending + block).split(b"\n")
        self.pending = lines.pop()
        if len(self.pending) > MAX_LINE_BYTES:
            raise ProgressError("progress_record_too_large")
        for line in lines:
            if not line or len(line) > MAX_LINE_BYTES:
                raise ProgressError("progress_record_invalid")
            try:
                event = json.loads(line)
            except (ValueError, UnicodeError, RecursionError) as error:
                raise ProgressError("progress_record_invalid") from error
            if not isinstance(event, dict):
                raise ProgressError("progress_record_invalid")
            kind = event.get("kind")
            self.counts[kind if isinstance(kind, str) and kind in KINDS else "other"] += 1
        return True, max(0, observed.st_size - self.offset), bool(self.pending)


def cgroup_counters(group: Path | None) -> dict:
    if group is None:
        return {"status": "unavailable"}
    try:
        result = {"status": "observed"}
        for name in ("memory.current", "memory.peak"):
            value = int((group / name).read_text(encoding="ascii").strip())
            if value < 0:
                raise ValueError("invalid cgroup counter")
            result[name.replace(".", "_")] = value
        for name, wanted in (("memory.events", {"oom", "oom_kill"}), ("cpu.stat", {"usage_usec", "throttled_usec"})):
            fields = {}
            for line in (group / name).read_text(encoding="ascii").splitlines():
                key, value = line.split()
                if key in wanted:
                    fields[key] = int(value)
                    if fields[key] < 0:
                        raise ValueError("invalid cgroup counter")
            if fields.keys() != wanted:
                raise ValueError("missing cgroup field")
            result[name.replace(".", "_")] = fields
        return result
    except (OSError, ValueError, UnicodeError):
        return {"status": "unavailable"}


class ProgressObserver:
    def __init__(self, evidence: Path, group: Path | None, interval_ms: int, max_bytes: int):
        if not 1_000 <= interval_ms <= 60_000 or not 4_096 <= max_bytes <= 16 * 1024 * 1024:
            raise ProgressError("observer_limits_invalid")
        self.evidence = evidence
        self.group = group
        self.interval = interval_ms / 1000
        self.max_bytes = max_bytes
        self.reader = ProgressReader(evidence)
        self.output_path = evidence.with_name(evidence.name + ".observations.jsonl")
        self.stop_event = threading.Event()
        self.started = None
        self.written = 0
        self.thread = None
        self.output = None
        self.error = None

    def __enter__(self):
        # Exclusive creation leaves any prior operator evidence untouched.
        self.output = self.output_path.open("x", encoding="utf-8", newline="\n")
        self.started = time.monotonic_ns()
        self.thread = threading.Thread(target=self._run, name="rrflow-progress-observer", daemon=True)
        try:
            self.thread.start()
        except BaseException:
            self.output.close()
            raise
        return self

    def _emit(self, final: bool) -> None:
        available, backlog, partial = self.reader.poll()
        row = {
            "format_version": 1,
            "kind": "observation",
            "elapsed_ns": time.monotonic_ns() - self.started,
            "counts": dict(self.reader.counts),
            "source_available": available,
            "progress_bytes_read": self.reader.offset,
            "backlog_bytes": backlog,
            "partial_record": partial,
            "limits": cgroup_counters(self.group),
            "final": final,
        }
        raw = json.dumps(row, separators=(",", ":")) + "\n"
        size = len(raw.encode("utf-8"))
        if self.written + size > self.max_bytes:
            raise ProgressError("observer_output_limit")
        self.output.write(raw)
        self.output.flush()
        self.written += size
        print("c07_progress_observation=" + raw.rstrip(), file=sys.stderr, flush=True)

    def _run(self) -> None:
        try:
            while not self.stop_event.is_set():
                self._emit(False)
                if self.stop_event.wait(self.interval):
                    break
            self._emit(True)
        except Exception as error:
            self.error = str(error) if isinstance(error, ProgressError) else "observer_failed"
            print("c07_progress_observer_error=" + self.error, file=sys.stderr, flush=True)

    def __exit__(self, exc_type, exc, traceback):
        self.stop_event.set()
        self.thread.join(timeout=10)
        if self.thread.is_alive():
            print("c07_progress_observer_error=observer_stop_timeout", file=sys.stderr, flush=True)
        else:
            self.output.close()
        return False
