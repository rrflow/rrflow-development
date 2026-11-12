#!/usr/bin/env python3
"""Does RRFlow's context tool find the files a real change needed? No model is involved.

For each task it installs a fresh engine, imports the repository's text at the task's starting
revision, and sends the task's text to the real rrflow-mcp process over the protocol an agent
uses. It scores the files that come back against the files the change touched, beside plain BM25
over the identical chunks, and writes the lab's ordinary report.

This measures lexical and graph retrieval over text this harness chunked. It does not measure
attunement, parsing or embeddings, and a good score does not show an agent would use the result.
Credentials stay in this process and its children, and in one private file that is removed.

Usage: retrieval_signal.py --manifest FILE --repository CLONE --rrflow BIN --mcp BIN --lab BIN
       --work DIR --out FILE [--tasks N]
"""
import argparse
import collections
import hashlib
import json
import math
import os
import re
import shutil
import socket
import subprocess
import sys
import time
import tomllib
import urllib.request
from pathlib import Path

TOOL_TIMEOUT_SECONDS = 600
# The installed default ceiling for one recall request.
MAX_ITEMS = 128
TOKEN = re.compile(r"[a-z0-9_]+")
TOP_FILES = (10, 50)


def run(*command, **options):
    return subprocess.run(command, check=True, capture_output=True, text=True, encoding="utf-8", **options)


def free_port():
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        return probe.getsockname()[1]


def start_engine(rrflow, project, work):
    """A fresh install, with its own copy of the binary because installation refuses a hard link."""
    project.mkdir(parents=True)
    binary = work / "bin" / Path(rrflow).name
    if not binary.exists():
        binary.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(rrflow, binary)
    identity = json.loads(run(str(binary), "install", "identity", "--json").stdout)["installation_identity"]
    plan = json.loads(
        run(str(binary), "install", "plan", "--project", str(project), "--identity", identity, "--mode", "fresh", "--json").stdout
    )
    plan_file = work / "plan.json"
    plan_file.write_text(json.dumps(plan), encoding="utf-8")
    run(
        str(binary), "install", "apply", "--project", str(project), "--mode", "fresh",
        "--plan", str(plan_file), "--expect", plan["installation"]["plan_sha256"],
    )
    port = free_port()
    log = open(work / "engine.log", "ab")
    engine = subprocess.Popen(
        [str(binary), "serve", "--project", str(project), "--bind", f"127.0.0.1:{port}"],
        stdout=log, stderr=log,
    )
    endpoint = f"http://127.0.0.1:{port}"
    for _ in range(240):
        try:
            urllib.request.urlopen(f"{endpoint}/v1/capabilities", timeout=2)
            break
        except OSError:
            if engine.poll() is not None:
                raise SystemExit(f"the engine exited with {engine.returncode}; see {work / 'engine.log'}")
            time.sleep(0.5)
    else:
        raise SystemExit("the engine did not answer")
    locator = tomllib.loads((project / ".rrflow" / "config.toml").read_text(encoding="utf-8"))
    key = json.loads((project / locator["operator_credential"]).read_text(encoding="utf-8"))
    return engine, endpoint, locator["identity"]["instance_id"], key["principal_id"], key["credential"]


def ask_through_mcp(mcp, endpoint, instance, principal, key_file, query):
    """Speaks the handshake an agent would, then calls the tool once."""
    child = subprocess.Popen(
        [mcp, "daemon", "--url", endpoint, "--instance", instance, "--principal", principal, "--api-key-file", str(key_file)],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    )
    messages = [
        {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-06-18", "capabilities": {},
            "clientInfo": {"name": "rrflow-lab", "version": "1"},
        }},
        {"jsonrpc": "2.0", "method": "notifications/initialized"},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list"},
        {"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "rrflow_context", "arguments": {
            "query": query, "max_items": MAX_ITEMS, "max_output_bytes": 524288, "max_storage_keys": 100000,
        }}},
    ]
    started = time.monotonic()
    payload = "".join(json.dumps(message) + "\n" for message in messages).encode()
    try:
        stdout, stderr = child.communicate(payload, timeout=TOOL_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        child.kill()
        return None, "the tool call timed out", time.monotonic() - started
    seconds = time.monotonic() - started
    replies = {reply.get("id"): reply for reply in map(json.loads, stdout.decode().splitlines()) if reply.get("id")}
    if 1 not in replies or "error" in replies[1]:
        return None, f"the MCP handshake failed: {stderr.decode()[:300]}", seconds
    tools = [tool["name"] for tool in replies.get(2, {}).get("result", {}).get("tools", [])]
    if tools != ["rrflow_context"]:
        return None, f"unexpected tools {tools}", seconds
    result = replies.get(3, {}).get("result")
    if result is None:
        return None, f"no tool result: {json.dumps(replies.get(3))[:300]}", seconds
    text = result["content"][0]["text"]
    if result.get("isError"):
        return None, text[:400], seconds
    return json.loads(text), None, seconds


def plain_bm25(chunks, query, limit, k1=1.2, b=0.75):
    """Textbook BM25, written here so the yardstick shares no code with the engine."""
    tokens = [collections.Counter(TOKEN.findall(chunk["text"].lower())) for chunk in chunks]
    lengths = [sum(counter.values()) for counter in tokens]
    average = sum(lengths) / max(len(lengths), 1)
    frequency = collections.Counter(term for counter in tokens for term in counter)
    scores = [0.0] * len(chunks)
    total = len(chunks)
    for term in set(TOKEN.findall(query.lower())):
        seen = frequency.get(term, 0)
        if not seen:
            continue
        idf = math.log(1 + (total - seen + 0.5) / (seen + 0.5))
        for position, counter in enumerate(tokens):
            count = counter.get(term, 0)
            if count:
                scores[position] += idf * count * (k1 + 1) / (count + k1 * (1 - b + b * lengths[position] / average))
    order = sorted(range(total), key=lambda position: (-scores[position], chunks[position]["id"]))
    return [chunks[position]["path"] for position in order[:limit] if scores[position] > 0]


def distinct(paths):
    return list(dict.fromkeys(paths))


def recall_at(ranked_files, wanted, count):
    return len(set(ranked_files[:count]) & wanted) / len(wanted)


def measure_task(task, arguments, work):
    checkout = work / "checkout"
    run("git", "-C", arguments.repository, "worktree", "add", "--detach", "--force", str(checkout), task["revision"])
    engine = None
    try:
        engine, endpoint, instance, principal, secret = start_engine(arguments.rrflow, work / "project", work)
        chunks_file = work / "chunks.jsonl"
        try:
            imported = json.loads(run(
                arguments.lab, "tasks-import", "--checkout", str(checkout), "--endpoint", endpoint,
                "--chunks-out", str(chunks_file),
                env={**os.environ, "RRFLOW_LAB_INSTANCE": instance, "RRFLOW_LAB_PRINCIPAL": principal, "RRFLOW_LAB_SECRET": secret},
            ).stdout)
        except subprocess.CalledProcessError as failure:
            # A refused import is a result, and it stays visible in the report.
            return {"task": task["id"], "error": f"the import failed: {failure.stderr.strip()[:400]}"}
        chunks = [json.loads(line) for line in chunks_file.read_text(encoding="utf-8").splitlines()]
        by_identity = {chunk["id"]: chunk["path"] for chunk in chunks}
        key_file = work / "mcp-api-key"
        key_file.write_text(secret, encoding="utf-8")
        if os.name != "nt":
            os.chmod(key_file, 0o600)
        existing = {chunk["path"] for chunk in chunks}
        # A file the change created did not exist yet, so no search could find it.
        wanted = set(task["expected_files"]) & existing
        packet, error, seconds = ask_through_mcp(arguments.mcp, endpoint, instance, principal, key_file, task["prompt"])
        row = {
            "task": task["id"], "chunks": imported["chunks"], "files": imported["files"],
            "import_seconds": imported["seconds"], "expected": len(task["expected_files"]),
            "retrievable": len(wanted), "seconds": seconds, "error": error,
        }
        if not wanted:
            row["error"] = row["error"] or "the change only created files"
            return row
        plain = distinct(plain_bm25(chunks, task["prompt"], MAX_ITEMS))
        for count in TOP_FILES:
            row[f"plain_at_{count}"] = recall_at(plain, wanted, count)
            row[f"chance_at_{count}"] = min(count / max(len(existing), 1), 1.0)
        if packet is not None:
            ranked = distinct(by_identity[item["identity"]] for item in packet["items"] if item["identity"] in by_identity)
            row["rrflow_items"] = len(packet["items"])
            row["rrflow_files_returned"] = len(ranked)
            row["output_bytes"] = packet["output_bytes"]
            row["truncated"] = packet["truncated"]
            row["keys_examined"] = packet["read_evidence"].get("keys_examined")
            for count in TOP_FILES:
                row[f"rrflow_at_{count}"] = recall_at(ranked, wanted, count)
        return row
    finally:
        if engine is not None:
            engine.kill()
            engine.wait()
        run("git", "-C", arguments.repository, "worktree", "remove", "--force", str(checkout))


def mean(values):
    values = list(values)
    return sum(values) / len(values) if values else 0.0


def build_report(rows, manifest_bytes, manifest, generated_at):
    measured = [row for row in rows if "plain_at_10" in row]
    answered = [row for row in measured if "rrflow_at_10" in row]
    observations = []
    def observe(system, metric, value, samples, case=None):
        entry = {"system": system, "metric": metric, "value": value, "samples": samples}
        if case:
            entry["case"] = case
        observations.append(entry)
    for count in TOP_FILES:
        metric = f"file_recall_at_{count}"
        observe("rrflow-context", metric, mean(row[f"rrflow_at_{count}"] for row in answered), len(answered))
        observe("plain-bm25", metric, mean(row[f"plain_at_{count}"] for row in measured), len(measured))
        observe("chance", metric, mean(row[f"chance_at_{count}"] for row in measured), len(measured))
        for row in answered:
            observe("rrflow-context", metric, row[f"rrflow_at_{count}"], 1, {"task": row["task"]})
        for row in measured:
            observe("plain-bm25", metric, row[f"plain_at_{count}"], 1, {"task": row["task"]})
    if answered:
        observe("rrflow-context", "seconds_per_query", sorted(row["seconds"] for row in answered)[len(answered) // 2], len(answered))
    wins = sum(row["rrflow_at_50"] > row["plain_at_50"] for row in answered)
    losses = sum(row["rrflow_at_50"] < row["plain_at_50"] for row in answered)
    rrflow_mean, plain_mean = (mean(row[f"{name}_at_50"] for row in answered) for name in ("rrflow", "plain"))
    failures = [row for row in rows if row.get("error")]
    def system(id_, label, status="completed", reason=None):
        entry = {"id": id_, "label": label, "kind": "retrieval", "status": status}
        if reason:
            entry["reason"] = reason
        return entry
    return {
        "schema_version": 1,
        "test": {"id": "rrflow-package-retrieval", "title": "Do retrieved files match what real changes needed?", "family": "agent-tasks", "revision": 1},
        "evidence_class": "diagnostic",
        "generated_by": "eval/workbench/lab/retrieval_signal.py",
        "generated_at_unix_ms": generated_at,
        "environment": {},
        "identities": {"manifest": hashlib.sha256(manifest_bytes).hexdigest()},
        "systems": [
            system("rrflow-context", "RRFlow's rrflow_context tool over text this harness imported",
                   "completed" if answered else "failed", None if answered else "no query was answered"),
            system("plain-bm25", "Plain BM25 over the same chunks"),
            system("chance", "Picking files at random"),
        ],
        "dimensions": [{"id": "task", "label": "Task"}],
        "metrics": [
            {"id": f"file_recall_at_{count}", "label": f"Needed files among the top {count} returned", "unit": "fraction", "better": "higher"}
            for count in TOP_FILES
        ] + [{"id": "seconds_per_query", "label": "Median seconds per query", "unit": "s", "better": "lower"}],
        "observations": observations,
        "verdicts": [{
            "id": "retrieves-at-least-plain-bm25",
            "question": "Did RRFlow's tool find at least as many needed files as plain BM25 over the same chunks?",
            "system": "rrflow-context",
            "outcome": "not_run" if not answered else ("met" if rrflow_mean >= plain_mean else "not_met"),
            "basis": f"mean file recall in the top 50 of {rrflow_mean:.3f} against {plain_mean:.3f} on {len(answered)} tasks: RRFlow higher on {wins}, plain BM25 higher on {losses}, {len(failures)} tasks unanswered or unscorable.",
        }],
        "views": [
            {"kind": "bars", "id": "recall-10", "title": "Needed files in the top 10", "metric": "file_recall_at_10", "fixed": {}},
            {"kind": "bars", "id": "recall-50", "title": "Needed files in the top 50", "metric": "file_recall_at_50", "fixed": {}},
            {"kind": "table", "id": "by-task", "title": "By task", "metrics": ["file_recall_at_10", "file_recall_at_50"], "rows": ["task"], "fixed": {}},
        ],
        "limits": [
            "No model was involved. This shows what a search returned, not whether an agent would use it or plan better with it.",
            "This measures lexical and graph retrieval over text this harness chunked. It does not measure attunement, parsing or embeddings, which RRFlow cannot yet apply to a repository.",
            "The query is the task's text, unchanged. An agent would write its own query.",
            "A file the change created did not exist yet, so only files present at the starting revision are scored.",
            "A task's answer is what one real change touched, which is one valid plan among others.",
            "Seconds come from whatever host ran this, with no isolation, so read them only as rough.",
        ] + [f"{row['task']}: {row['error'][:300]}" for row in failures],
        "artifacts": [],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    for name in ("manifest", "repository", "rrflow", "mcp", "lab", "work", "out"):
        parser.add_argument(f"--{name}", required=True)
    parser.add_argument("--tasks", type=int, default=0)
    arguments = parser.parse_args()
    manifest_bytes = Path(arguments.manifest).read_bytes()
    manifest = json.loads(manifest_bytes)
    tasks = manifest["tasks"][: arguments.tasks or None]
    out = Path(arguments.out)
    rows_file = out.with_suffix(".rows.jsonl")
    done = {}
    if rows_file.exists():
        done = {row["task"]: row for row in map(json.loads, rows_file.read_text(encoding="utf-8").splitlines())}
    for task in tasks:
        if task["id"] in done:
            continue
        work = Path(arguments.work) / task["id"]
        shutil.rmtree(work, ignore_errors=True)
        work.mkdir(parents=True)
        started = time.monotonic()
        row = measure_task(task, arguments, work)
        done[task["id"]] = row
        with rows_file.open("a", encoding="utf-8", newline="\n") as rows:
            rows.write(json.dumps(row) + "\n")
        print(json.dumps({key: (round(value, 3) if isinstance(value, float) else value) for key, value in row.items()}), f"total={time.monotonic() - started:.0f}s", flush=True)
        shutil.rmtree(work, ignore_errors=True)
    report = build_report([done[task["id"]] for task in tasks if task["id"] in done], manifest_bytes, manifest, int(time.time() * 1000))
    out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8", newline="\n")
    run(arguments.lab, "report-validate", "--file", str(out))
    print(report["verdicts"][0]["basis"])


if __name__ == "__main__":
    sys.exit(main())
