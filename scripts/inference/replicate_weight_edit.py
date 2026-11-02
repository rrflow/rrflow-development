"""Test a published sparse weight edit against a decision rule frozen before any complete run.

This offline tool has no engine, training, admission or activation authority. Its outputs are
experiment evidence for docs/reference/inference/learning/weight-space-edits.md; a passing
decision activates nothing. Only the prepare and evaluate commands need third-party packages.
"""
from __future__ import annotations

import argparse
import ast
from array import array
import functools
import gzip
import hashlib
import importlib.metadata
import io
import math
import os
from pathlib import Path
import platform
import shutil
import struct
import sys
import tempfile
import time
import zipfile

from review_outcomes import canonical, digest, fields, identity, read_bounded, strict_json

MAX_HEADER_BYTES = 64 * 1024 * 1024
MAX_EDIT_ARRAY_BYTES = 64 * 1024 * 1024
MAX_BENCHMARK_BYTES = 64 * 1024 * 1024
MAX_SAMPLES_BYTES = 1024 * 1024 * 1024
CHUNK_BYTES = 1024 * 1024
DTYPE_BYTES = {"BF16": 2, "F16": 2, "F32": 4, "F64": 8, "I64": 8, "I32": 4, "I16": 2, "I8": 1, "U8": 1, "BOOL": 1}
NPY_TYPES = {"<i4": "i", "<u2": "H"}
TEXT_PREFIXES = ("model.language_model.", "lm_head.")
CONFIG_FILES = ("config.json", "generation_config.json", "tokenizer_config.json", "chat_template.jinja")
RECEIPT = "rebuild-receipt.json"
INSTRUCTION = "Please reason step by step, and put your final answer within \\boxed{}."
GRADER_VERSION = "0.9.0"
ENVIRONMENT_PACKAGES = ("vllm", "transformers", "torch", "math-verify", "latex2sympy2_extended",
                        "antlr4-python3-runtime")
# An allow-list, because other VLLM_ variables can hold credentials.
ENVIRONMENT_SETTINGS = ("VLLM_USE_FLASHINFER_SAMPLER",)
# The published thinking evaluation and the model card's non-thinking benchmark setting.
PROTOCOLS = {
    "think35": {"thinking": True, "sampling": {"temperature": 0.6, "top_p": 0.95, "top_k": 20,
                                               "presence_penalty": 1.5, "max_tokens": 32768, "seed": 0}},
    "nothink35": {"thinking": False, "sampling": {"temperature": 0.7, "top_p": 0.8, "top_k": 20,
                                                  "presence_penalty": 1.5, "max_tokens": 8192, "seed": 0}},
}
SAMPLES = {"math500": 4, "gsm8k": 2}
PROBLEMS = {"math500": 500, "gsm8k": 1319}
# Frozen before any complete run; see the decision rule in the owning record.
MARGIN_POINTS = 5.0
Z95 = 1.96


def file_digest(path: Path) -> str:
    state = hashlib.sha256()
    with path.open("rb") as source:
        while chunk := source.read(CHUNK_BYTES):
            state.update(chunk)
    return state.hexdigest()


def tensor_index(path: Path) -> tuple[dict, int]:
    """Read and validate one safetensors header without reading tensor bytes."""
    size = path.stat().st_size
    with path.open("rb") as source:
        prefix = source.read(8)
        if len(prefix) != 8:
            raise ValueError("safetensors file is too short")
        length = struct.unpack("<Q", prefix)[0]
        if length > MAX_HEADER_BYTES or 8 + length > size:
            raise ValueError("safetensors header exceeds its bound or the file")
        header = strict_json(source.read(length))
    if not isinstance(header, dict):
        raise ValueError("safetensors header must be an object")
    start = 8 + length
    tensors = {}
    for name, info in header.items():
        if name == "__metadata__":
            continue
        fields(info, {"dtype", "shape", "data_offsets"})
        try:
            begin, end = info["data_offsets"]
            count = math.prod(info["shape"])
            valid = (info["dtype"] in DTYPE_BYTES and 0 <= begin <= end <= size - start
                     and end - begin == count * DTYPE_BYTES[info["dtype"]])
        except (TypeError, ValueError):
            valid = False
        if not valid:
            raise ValueError(f"invalid safetensors tensor record: {name}")
        tensors[name] = info
    return tensors, start


def read_npy(content: bytes) -> array:
    """Decode one little-endian one-dimensional array; nothing here can load a pickle."""
    if content[:6] != b"\x93NUMPY" or len(content) < 12:
        raise ValueError("edit array is not a NumPy file")
    wide = content[6] >= 2
    length = struct.unpack("<I" if wide else "<H", content[8:12 if wide else 10])[0]
    body = 12 + length if wide else 10 + length
    try:
        header = ast.literal_eval(content[12 if wide else 10:body].decode("latin1"))
    except SyntaxError as error:
        raise ValueError("edit array header is malformed") from error
    if (not isinstance(header, dict) or header.get("descr") not in NPY_TYPES or header.get("fortran_order") is not False
            or not isinstance(header.get("shape"), tuple) or len(header["shape"]) != 1):
        raise ValueError("edit array must be a one-dimensional int32 or uint16 array")
    values = array(NPY_TYPES[header["descr"]])
    if (len(content) - body) != header["shape"][0] * values.itemsize:
        raise ValueError("edit array length differs from its header")
    values.frombytes(content[body:])
    return values


def load_edit(edit_dir: Path) -> dict:
    """Bind one sparse edit's manifest and archive; a low-rank edit is refused, not approximated."""
    manifest_bytes = read_bounded(edit_dir / "manifest.json")
    archive_bytes = read_bounded(edit_dir / "edit.npz")
    manifest = fields(strict_json(manifest_bytes), {"tensors", "changed_tensors", "changed_entries_total"}, {"note"})
    tensors = manifest["tensors"]
    if not isinstance(tensors, list) or not tensors or manifest["changed_tensors"] != len(tensors):
        raise ValueError("edit manifest must list its changed tensors")
    if not all(isinstance(record, dict) and record.get("form") == "sparse" for record in tensors):
        raise ValueError("only sparse edits are supported")
    arrays = {}
    try:
        archive = zipfile.ZipFile(io.BytesIO(archive_bytes))
    except zipfile.BadZipFile as error:
        raise ValueError("edit archive is not a zip file") from error
    with archive:
        expected = {f"t{i}_{part}.npy" for i in range(len(tensors)) for part in ("idx", "val")}
        if set(archive.namelist()) != expected:
            raise ValueError("edit archive members differ from a sparse edit of its manifest")
        for i, record in enumerate(tensors):
            fields(record, {"name", "shape", "dtype", "changed_entries", "changed_columns", "changed_rows",
                            "relative_change", "base_sha256", "edited_sha256", "submitted_file", "form"})
            if record["dtype"] != "bfloat16":
                raise ValueError("only bfloat16 edits are supported")
            identity(record["base_sha256"])
            identity(record["edited_sha256"])
            pair = []
            for part in ("idx", "val"):
                member = archive.getinfo(f"t{i}_{part}.npy")
                if member.file_size > MAX_EDIT_ARRAY_BYTES:
                    raise ValueError("edit array exceeds its bound")
                pair.append(read_npy(archive.read(member)))
            arrays[record["name"]] = (record, pair[0], pair[1])
    if len(arrays) != len(tensors):
        raise ValueError("edit manifest names one tensor twice")
    if manifest["changed_entries_total"] != sum(record["changed_entries"] for record in tensors):
        raise ValueError("edit manifest total differs from its tensors")
    return {"manifestSha256": digest(manifest_bytes), "archiveSha256": digest(archive_bytes), "tensors": arrays}


def bf16_floats(content: bytes) -> array:
    bits = array("H")
    bits.frombytes(content)
    floats = array("f")
    floats.frombytes(array("I", (bit << 16 for bit in bits)).tobytes())
    return floats


def relative_change(base: bytes, edited: bytes) -> float:
    before, after = bf16_floats(base), bf16_floats(edited)
    denominator = math.fsum(value * value for value in before)
    if denominator == 0:
        raise ValueError("cannot measure relative change of a zero tensor")
    return math.sqrt(math.fsum((new - old) ** 2 for new, old in zip(after, before)) / denominator)


def apply_sparse(base: bytes, record: dict, indices: array, values: array) -> bytes:
    bits = array("H")
    bits.frombytes(base)
    if len(indices) != len(values):
        raise ValueError("edit indices and values differ in length")
    previous, changed = -1, 0
    for index, value in zip(indices, values):
        if not previous < index < len(bits):
            raise ValueError("edit indices must increase and stay inside the tensor")
        changed += bits[index] != value
        bits[index] = value
        previous = index
    if changed != record["changed_entries"]:
        raise ValueError("edit changes a different entry count than its manifest declares")
    return bits.tobytes()


def rebuild(base_file: Path, config_dir: Path, tokenizer: Path, output: Path,
            edit_dir: Path | None = None, expected_base: str | None = None) -> dict:
    """Publish one text-only checkpoint, edited or not, so both arms load from the same file shape."""
    if sys.byteorder != "little":
        raise ValueError("this tool assumes a little-endian host")
    if output.exists():
        raise ValueError("output already exists")
    edit = load_edit(edit_dir) if edit_dir else None
    tensors, start = tensor_index(base_file)
    names = sorted(name for name in tensors if name.startswith(TEXT_PREFIXES))
    edits = edit["tensors"] if edit else {}
    if not names or edits.keys() - set(names):
        raise ValueError("edit names a tensor outside the text checkpoint")
    base_sha = file_digest(base_file)
    if expected_base and base_sha != expected_base:
        raise ValueError("base file differs from the pinned digest")
    header, offset = {"__metadata__": {"format": "pt"}}, 0
    for name in names:
        length = tensors[name]["data_offsets"][1] - tensors[name]["data_offsets"][0]
        header[name] = {"dtype": tensors[name]["dtype"], "shape": tensors[name]["shape"],
                        "data_offsets": [offset, offset + length]}
        offset += length
    encoded = canonical(header).rstrip(b"\n")
    encoded += b" " * (-len(encoded) % 8)
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=".weight-edit-", dir=output.parent))
    try:
        records = []
        with base_file.open("rb") as source, (staging / "model.safetensors").open("wb") as target:
            target.write(struct.pack("<Q", len(encoded)) + encoded)
            for name in names:
                begin, end = tensors[name]["data_offsets"]
                source.seek(start + begin)
                if name not in edits:
                    remaining = end - begin
                    while remaining:
                        chunk = source.read(min(remaining, CHUNK_BYTES))
                        if not chunk:
                            raise ValueError("safetensors file ended inside a tensor")
                        target.write(chunk)
                        remaining -= len(chunk)
                    continue
                record, indices, values = edits[name]
                if tensors[name]["dtype"] != "BF16" or tensors[name]["shape"] != record["shape"]:
                    raise ValueError(f"base tensor type or shape differs from the edit: {name}")
                content = source.read(end - begin)
                if digest(content) != record["base_sha256"]:
                    raise ValueError(f"base tensor differs from the one the edit was made on: {name}")
                edited = apply_sparse(content, record, indices, values)
                if digest(edited) != record["edited_sha256"]:
                    raise ValueError(f"rebuilt tensor differs from the evaluated checkpoint: {name}")
                target.write(edited)
                records.append({"name": name, "changedEntries": record["changed_entries"],
                                "relativeChange": relative_change(content, edited),
                                "manifestRelativeChange": record["relative_change"],
                                "baseSha256": record["base_sha256"], "editedSha256": record["edited_sha256"]})
        configs = {}
        for name in CONFIG_FILES:
            shutil.copyfile(config_dir / name, staging / name)
            configs[name] = file_digest(staging / name)
        shutil.copyfile(tokenizer, staging / "tokenizer.json")
        receipt = {"schemaVersion": 1, "purpose": "weight_edit_replication_checkpoint",
                   "baseFileSha256": base_sha, "modelSha256": file_digest(staging / "model.safetensors"),
                   "tokenizerSha256": file_digest(staging / "tokenizer.json"), "configSha256": configs,
                   "textTensors": len(names),
                   "edit": edit and {"manifestSha256": edit["manifestSha256"], "archiveSha256": edit["archiveSha256"],
                                     "tensors": records}}
        (staging / RECEIPT).write_bytes(canonical(receipt))
        os.replace(staging, output)
        return receipt
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def box_at(value: str, index: int) -> str | None:
    key = "\\boxed" if value.startswith("\\boxed", index) else "\\fbox"
    rest = value[index + len(key):]
    stripped = rest.lstrip()
    if stripped.startswith("{"):
        depth = 0
        for position, character in enumerate(stripped):
            if character == "{":
                depth += 1
            elif character == "}":
                depth -= 1
                if depth == 0:
                    return stripped[1:position].strip() or None
        return None
    if key == "\\boxed" and rest.startswith(" "):
        return rest.strip().split("$")[0].split("\n")[0].strip() or None
    return None


def extract(value: str) -> str | None:
    """The last boxed answer after any reasoning block, matching the published grader's reading."""
    if "</think>" in value:
        value = value.split("</think>")[-1].strip()
    index = value.rfind("\\boxed")
    if index < 0:
        index = value.rfind("\\fbox")
    return None if index < 0 else box_at(value, index)


def math_verify_grader():
    """The published checker; another version could silently change every score."""
    observed = importlib.metadata.version("math-verify")
    if observed != GRADER_VERSION:
        raise ValueError(f"math-verify {GRADER_VERSION} is required, found {observed}")
    import math_verify

    @functools.lru_cache(maxsize=1 << 16)
    def parsed(value: str):
        return math_verify.parse("\\boxed{" + value + "}", parsing_timeout=5)

    def is_correct(prediction: str | None, gold: str | None) -> bool:
        if prediction is None or gold is None:
            return False
        # One unparseable answer must not discard hours of generation; regrade catches a broken grader.
        try:
            target, answer = parsed(gold), parsed(prediction)
            return bool(target and answer and math_verify.verify(target, answer, timeout_seconds=5))
        except Exception:
            return False

    return is_correct


def vllm_generator(checkpoint: Path, protocol: dict, memory_fraction: float):
    from transformers import AutoTokenizer
    from vllm import LLM, SamplingParams
    sampling = protocol["sampling"]
    tokenizer = AutoTokenizer.from_pretrained(str(checkpoint))
    llm = LLM(model=str(checkpoint), dtype="bfloat16", gpu_memory_utilization=memory_fraction,
              max_model_len=sampling["max_tokens"] + 2048, seed=0)

    def generate(prompts: list[str], samples: int) -> list[list[tuple]]:
        rendered = [tokenizer.apply_chat_template([{"role": "user", "content": prompt}], tokenize=False,
                                                  add_generation_prompt=True, enable_thinking=protocol["thinking"])
                    for prompt in prompts]
        return [[(sample.text, sample.finish_reason, len(sample.token_ids)) for sample in result.outputs]
                for result in llm.generate(rendered, SamplingParams(n=samples, **sampling))]

    return generate


def observed_environment() -> dict:
    versions = {}
    for package in ENVIRONMENT_PACKAGES:
        try:
            versions[package] = importlib.metadata.version(package)
        except importlib.metadata.PackageNotFoundError:
            versions[package] = None
    try:
        import torch
        device = torch.cuda.get_device_name(0)
    except Exception as error:
        device = f"unavailable: {type(error).__name__}"
    return {"packages": versions, "platform": platform.platform(), "device": device,
            "settings": {name: os.environ.get(name) for name in ENVIRONMENT_SETTINGS}}


def load_benchmark(path: Path) -> tuple[list[dict], str]:
    content = read_bounded(path, MAX_BENCHMARK_BYTES)
    rows = [fields(strict_json(line), {"id", "problem", "answer"}) for line in content.splitlines() if line]
    if not rows or len({row["id"] for row in rows}) != len(rows):
        raise ValueError("benchmark needs unique problem identities")
    return rows, digest(content)


def evaluate(checkpoint: Path, benchmark: Path, arm: Path, name: str, protocol_name: str,
             shard: tuple[int, int] = (0, 1), limit: int | None = None, generate=None, is_correct=None,
             environment=None, memory_fraction: float = 0.85) -> dict:
    """Run one shard of one arm; a repeat with an identical job reads its result and generates nothing."""
    protocol = PROTOCOLS[protocol_name]
    if name not in SAMPLES or not 0 <= shard[0] < shard[1]:
        raise ValueError("unsupported benchmark set or shard")
    receipt = strict_json(read_bounded(checkpoint / RECEIPT))
    rows, benchmark_sha = load_benchmark(benchmark)
    job = {"protocol": protocol_name, "sampling": protocol["sampling"], "thinking": protocol["thinking"],
           "samples": SAMPLES[name], "prompt": INSTRUCTION, "benchmarkSha256": benchmark_sha,
           "checkpointModelSha256": file_digest(checkpoint / "model.safetensors"),
           "baseFileSha256": receipt["baseFileSha256"], "edited": receipt["edit"] is not None,
           "shard": list(shard), "limit": limit}
    if job["checkpointModelSha256"] != receipt["modelSha256"]:
        raise ValueError("checkpoint model differs from its rebuild receipt")
    part = arm / "parts" / f"{name}_s{shard[0]}of{shard[1]}"
    if part.exists():
        existing = strict_json(read_bounded(part / "scores.json"))
        if existing.get("_job") != job:
            raise ValueError("this result part already holds a different job")
        return existing
    rows = rows[shard[0]::shard[1]]
    rows = rows if limit is None else rows[:limit]
    if not rows:
        raise ValueError("this shard and limit select no problems")
    is_correct = is_correct or math_verify_grader()
    generate = generate or vllm_generator(checkpoint, protocol, memory_fraction)
    began = time.monotonic()
    outputs = generate([row["problem"] + "\n" + INSTRUCTION for row in rows], SAMPLES[name])
    if len(outputs) != len(rows):
        raise ValueError("generator returned a different problem count")
    per_problem, samples, ok, total, cut, tokens = [], [], 0, 0, 0, 0
    for row, row_outputs in zip(rows, outputs):
        if len(row_outputs) != SAMPLES[name]:
            raise ValueError("generator returned a different sample count")
        correct = []
        for value, finish, token_count in row_outputs:
            if finish not in ("stop", "length"):
                raise ValueError("a generation ended without finishing or reaching its limit; nothing was saved")
            correct.append(int(is_correct(extract(value), row["answer"])))
            cut += finish == "length"
            tokens += token_count
        ok, total = ok + sum(correct), total + len(correct)
        per_problem.append({"id": row["id"], "correct": correct, "pred_first": extract(row_outputs[0][0])})
        samples.append({"id": row["id"], "answer": row["answer"], "text": row_outputs[0][0],
                        "finishReason": row_outputs[0][1]})
    scores = {name: {"ok": ok, "total": total, "cut": cut, "tokens_sum": tokens, "n_problems": len(rows),
                     "samples": SAMPLES[name], "acc": ok / total, "cut_share": cut / total,
                     "mean_tokens": tokens / total, "s": round(time.monotonic() - began)},
              "_job": job, "_environment": (environment or observed_environment)()}
    part.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=".weight-edit-", dir=part.parent))
    try:
        (staging / f"{name}_per_problem.json").write_bytes(canonical(per_problem))
        (staging / f"{name}_first_samples.jsonl.gz").write_bytes(
            gzip.compress(b"".join(canonical(sample) for sample in samples), mtime=0))
        (staging / "scores.json").write_bytes(canonical(scores))
        os.replace(staging, part)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise
    return scores


def result_parts(arm: Path, name: str) -> list[Path]:
    parts = sorted((arm / "parts").glob(f"{name}_s*of*"))
    if not parts:
        raise ValueError(f"no {name} results in {arm}")
    return parts


def load_arm(arm: Path, name: str) -> dict:
    """Read one arm's per-problem results, in this tool's layout or the published one."""
    problems, jobs = {}, []
    totals = {"ok": 0, "total": 0, "cut": 0, "tokens_sum": 0}
    for part in result_parts(arm, name):
        scores = strict_json(read_bounded(part / "scores.json"))
        ok = total = 0
        for row in strict_json(read_bounded(part / f"{name}_per_problem.json")):
            correct = row["correct"]
            if row["id"] in problems or not correct or any(value not in (0, 1) for value in correct):
                raise ValueError(f"duplicate or malformed problem result: {row['id']}")
            problems[row["id"]] = correct
            ok, total = ok + sum(correct), total + len(correct)
        record = scores[name]
        if (ok, total) != (record["ok"], record["total"]):
            raise ValueError(f"per-problem results differ from the recorded totals in {part.name}")
        for key in totals:
            totals[key] += record[key]
        jobs.append(scores.get("_job"))
    return {"problems": problems, "jobs": jobs, "totals": totals}


def summary(loaded: dict) -> dict:
    totals = loaded["totals"]
    return {"accuracy": 100 * totals["ok"] / totals["total"], "cutShare": 100 * totals["cut"] / totals["total"],
            "meanTokens": totals["tokens_sum"] / totals["total"]}


def compare(baseline: Path, candidate: Path, name: str) -> dict:
    """Pair two arms problem by problem; the problem is the unit, so shared difficulty cancels."""
    before, after = load_arm(baseline, name), load_arm(candidate, name)
    if before["problems"].keys() != after["problems"].keys():
        raise ValueError("arms cover different problems")
    if len(before["problems"]) < 2:
        raise ValueError("a paired comparison needs at least two problems")
    differences = [100 * (sum(after["problems"][key]) / len(after["problems"][key])
                          - sum(values) / len(values)) for key, values in before["problems"].items()]
    mean = math.fsum(differences) / len(differences)
    variance = math.fsum((value - mean) ** 2 for value in differences) / (len(differences) - 1)
    error = math.sqrt(variance / len(differences))
    return {"set": name, "problems": len(differences), "baseline": summary(before), "candidate": summary(after),
            "difference": mean, "standardError": error, "ci95": [mean - Z95 * error, mean + Z95 * error],
            "improved": sum(value > 0 for value in differences), "regressed": sum(value < 0 for value in differences),
            "tied": sum(value == 0 for value in differences)}


def require_arm(arm: Path, name: str, edited: bool, protocol: str, base: str | None) -> str:
    """A decision reads only complete arms of the expected kind, all built from one base file."""
    loaded = load_arm(arm, name)
    if len(loaded["problems"]) != PROBLEMS[name]:
        raise ValueError(f"{arm} does not cover every {name} problem")
    for job in loaded["jobs"]:
        if (not job or job.get("limit") is not None or job.get("edited") is not edited
                or job.get("protocol") != protocol or not job.get("baseFileSha256")):
            raise ValueError(f"{arm} is not a complete {protocol} arm of the expected model")
        if base not in (None, job["baseFileSha256"]):
            raise ValueError(f"{arm} was built from a different base file")
        base = job["baseFileSha256"]
    return base


def decide(published: Path, edited: Path, base: Path, control: Path, names: tuple[str, ...] = ("math500", "gsm8k")) -> dict:
    """Apply the frozen rule: reproduce the effect, then require an advantage over the documented setting."""
    results = {}
    for name in names:
        identity_base = require_arm(edited, name, True, "think35", None)
        require_arm(base, name, False, "think35", identity_base)
        require_arm(control, name, False, "nothink35", identity_base)
        results[name] = {"gain": compare(base, edited, name), "againstPublished": compare(published, edited, name),
                         "againstControl": compare(control, edited, name)}
    gained = all(result["gain"]["ci95"][0] > 0 for result in results.values())
    close = all(abs(result["againstPublished"]["difference"]) <= MARGIN_POINTS for result in results.values())
    effect = ("reproduced" if gained and close else "partial" if gained else "not_reproduced")
    if all(result["againstControl"]["ci95"][0] > 0 for result in results.values()):
        advantage = "edit_better_than_setting"
    elif any(result["againstControl"]["ci95"][1] < 0 for result in results.values()):
        advantage = "setting_better_than_edit"
    else:
        advantage = "no_demonstrated_advantage"
    proceed = effect == "reproduced" and advantage == "edit_better_than_setting"
    return {"schemaVersion": 1, "purpose": "weight_edit_replication_decision", "marginPoints": MARGIN_POINTS,
            "effect": effect, "advantage": advantage, "decision": "continue" if proceed else "stop",
            "scope": "this published edit of this model only; authorizes and activates nothing", "results": results}


def regrade(arm: Path, name: str, is_correct=None) -> dict:
    """Check extraction, and grading when a grader is given, against an arm's recorded first samples."""
    checked = extraction_agree = grade_agree = 0
    disagreements = []
    for part in result_parts(arm, name):
        recorded = {row["id"]: row for row in strict_json(read_bounded(part / f"{name}_per_problem.json"))}
        with gzip.open(part / f"{name}_first_samples.jsonl.gz", "rb") as source:
            content = source.read(MAX_SAMPLES_BYTES + 1)
        if len(content) > MAX_SAMPLES_BYTES:
            raise ValueError("first samples exceed their bound")
        for line in content.splitlines():
            sample = strict_json(line)
            row = recorded[sample["id"]]
            answer = extract(sample["text"])
            checked += 1
            extraction_ok = answer == row["pred_first"]
            extraction_agree += extraction_ok
            grade_ok = None
            if is_correct:
                grade_ok = int(is_correct(answer, sample["answer"])) == row["correct"][0]
                grade_agree += grade_ok
            if (not extraction_ok or grade_ok is False) and len(disagreements) < 20:
                disagreements.append({"id": sample["id"], "recorded": row["pred_first"], "observed": answer,
                                      "recordedCorrect": row["correct"][0]})
    return {"set": name, "checked": checked, "extractionAgree": extraction_agree,
            "gradeAgree": grade_agree if is_correct else None, "disagreements": disagreements}


def download_benchmarks() -> dict:
    """Explicit network step, with the published problem identities and gold answers."""
    from datasets import load_dataset
    return {"math500": [{"id": row["unique_id"], "problem": row["problem"], "answer": row["answer"]}
                        for row in load_dataset("HuggingFaceH4/MATH-500", split="test")],
            "gsm8k": [{"id": f"gsm8k_test_{i}", "problem": row["question"],
                       "answer": row["answer"].split("####")[-1].strip().replace(",", "")}
                      for i, row in enumerate(load_dataset("openai/gsm8k", "main", split="test"))]}


def check_against_published(name: str, rows: list[dict], published: Path) -> None:
    """Find a changed dataset now, not after hours of generation."""
    if {row["id"] for row in rows} != set(load_arm(published, name)["problems"]):
        raise ValueError(f"{name} problems differ from the published evaluation")
    answers = {row["id"]: row["answer"] for row in rows}
    for part in result_parts(published, name):
        with gzip.open(part / f"{name}_first_samples.jsonl.gz", "rb") as source:
            content = source.read(MAX_SAMPLES_BYTES + 1)
        if len(content) > MAX_SAMPLES_BYTES:
            raise ValueError("first samples exceed their bound")
        for line in content.splitlines():
            sample = strict_json(line)
            if answers[sample["id"]] != sample["answer"]:
                raise ValueError(f"{name} gold answer differs from the published evaluation: {sample['id']}")


def prepare(output_dir: Path, published: Path, tables: dict | None = None) -> dict:
    tables = tables or download_benchmarks()
    for name, rows in tables.items():
        check_against_published(name, rows, published)
    output_dir.mkdir(parents=True, exist_ok=True)
    result = {}
    for name, rows in tables.items():
        content = b"".join(canonical(row) for row in rows)
        path = output_dir / f"{name}.jsonl"
        if path.exists() and path.read_bytes() != content:
            raise ValueError(f"{path} already holds different content")
        path.write_bytes(content)
        result[name] = {"problems": len(rows), "sha256": digest(content)}
    return result


def shard_value(value: str) -> tuple[int, int]:
    index, count = value.split("/")
    return int(index), int(count)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    command = commands.add_parser("prepare")
    command.add_argument("--published", type=Path, required=True, help="the release's eval/unedited folder")
    command.add_argument("output_dir", type=Path)
    command = commands.add_parser("rebuild")
    command.add_argument("--base-file", type=Path, required=True)
    command.add_argument("--config-dir", type=Path, required=True)
    command.add_argument("--tokenizer", type=Path, required=True)
    command.add_argument("--edit-dir", type=Path)
    command.add_argument("--expect-base-sha256")
    command.add_argument("output", type=Path)
    command = commands.add_parser("evaluate")
    command.add_argument("--checkpoint", type=Path, required=True)
    command.add_argument("--benchmark", type=Path, required=True)
    command.add_argument("--arm", type=Path, required=True)
    command.add_argument("--set", choices=sorted(SAMPLES), required=True)
    command.add_argument("--protocol", choices=sorted(PROTOCOLS), required=True)
    command.add_argument("--shard", type=shard_value, default=(0, 1))
    command.add_argument("--limit", type=int, help="smoke run only; a decision refuses limited arms")
    command.add_argument("--gpu-memory-fraction", type=float, default=0.85)
    command = commands.add_parser("regrade")
    command.add_argument("--arm", type=Path, required=True)
    command.add_argument("--set", choices=sorted(SAMPLES), required=True)
    command.add_argument("--extraction-only", action="store_true")
    command = commands.add_parser("compare")
    command.add_argument("--baseline", type=Path, required=True)
    command.add_argument("--candidate", type=Path, required=True)
    command.add_argument("--set", choices=sorted(SAMPLES), required=True)
    command = commands.add_parser("decide")
    command.add_argument("--published", type=Path, required=True)
    command.add_argument("--edited", type=Path, required=True)
    command.add_argument("--base", type=Path, required=True)
    command.add_argument("--control", type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.command == "prepare":
            result = prepare(args.output_dir, args.published)
        elif args.command == "rebuild":
            result = rebuild(args.base_file, args.config_dir, args.tokenizer, args.output, args.edit_dir,
                             args.expect_base_sha256)
        elif args.command == "evaluate":
            result = evaluate(args.checkpoint, args.benchmark, args.arm, args.set, args.protocol, args.shard,
                              args.limit, memory_fraction=args.gpu_memory_fraction)
        elif args.command == "regrade":
            result = regrade(args.arm, args.set, None if args.extraction_only else math_verify_grader())
        elif args.command == "compare":
            result = compare(args.baseline, args.candidate, args.set)
        else:
            result = decide(args.published, args.edited, args.base, args.control)
    except (ValueError, OSError, KeyError, TypeError) as error:
        parser.exit(1, f"weight edit replication rejected: {error}\n")
    sys.stdout.write(canonical(result).decode("utf-8"))


if __name__ == "__main__":
    main()
