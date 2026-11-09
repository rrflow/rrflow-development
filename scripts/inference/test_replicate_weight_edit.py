"""Boundary tests for the frozen weight-edit replication tool, using synthetic fixtures only."""
from array import array
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import struct
import tempfile
import unittest
from unittest import mock
import zipfile

from replicate_weight_edit import (compare, decide, evaluate, extract, load_arm, observed_environment, prepare, rebuild,
                                   regrade, require_arm, RECEIPT)
from review_outcomes import canonical

EDITED = "model.language_model.layers.0.mlp.weight"
OTHER = "model.language_model.layers.1.mlp.weight"
VISION = "model.visual.blocks.0.weight"


def bf16(values):
    return array("H", values).tobytes()


def sha(content):
    return hashlib.sha256(content).hexdigest()


def npy(descr, code, values):
    body = array(code, values).tobytes()
    header = f"{{'descr': '{descr}', 'fortran_order': False, 'shape': ({len(values)},), }}".encode("latin1")
    return b"\x93NUMPY\x01\x00" + struct.pack("<H", len(header)) + header + body


def safetensors(path, tensors, dtype="BF16"):
    header, offset, blobs = {"__metadata__": {"format": "pt"}}, 0, []
    for name, (shape, content) in tensors.items():
        header[name] = {"dtype": dtype, "shape": shape, "data_offsets": [offset, offset + len(content)]}
        offset += len(content)
        blobs.append(content)
    encoded = json.dumps(header).encode()
    path.write_bytes(struct.pack("<Q", len(encoded)) + encoded + b"".join(blobs))


class Fixture(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        # 0x3F80 is 1.0 and 0x4000 is 2.0 in bfloat16.
        self.base_bits = [0x3F80, 0x3F80, 0x3F80, 0x3F80]
        self.edited_bits = [0x4000, 0x3F80, 0x4000, 0x3F80]
        self.base_file = self.root / "base.safetensors"
        safetensors(self.base_file, {EDITED: ([2, 2], bf16(self.base_bits)), OTHER: ([1, 2], bf16([1, 2])),
                                     VISION: ([1, 1], bf16([9]))})
        self.config = self.root / "config"
        self.config.mkdir()
        for name in ("config.json", "generation_config.json", "tokenizer_config.json", "chat_template.jinja"):
            (self.config / name).write_text(name)
        self.tokenizer = self.root / "tokenizer.json"
        self.tokenizer.write_text("{}")
        self.edit = self.root / "edit"
        self.edit.mkdir()
        self.write_edit()

    def record(self, **changes):
        return {"name": EDITED, "shape": [2, 2], "dtype": "bfloat16", "changed_entries": 2, "changed_columns": 1,
                "changed_rows": 2, "relative_change": 1.0, "base_sha256": sha(bf16(self.base_bits)),
                "edited_sha256": sha(bf16(self.edited_bits)), "submitted_file": "x.npy", "form": "sparse", **changes}

    def write_edit(self, indices=(0, 2), values=(0x4000, 0x4000), record=None, members=None):
        record = record or self.record()
        (self.edit / "manifest.json").write_bytes(canonical(
            {"tensors": [record], "changed_tensors": 1, "changed_entries_total": record["changed_entries"], "note": "x"}))
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w", zipfile.ZIP_DEFLATED) as archive:
            contents = members or {"t0_idx.npy": npy("<i4", "i", indices), "t0_val.npy": npy("<u2", "H", values)}
            for name, content in contents.items():
                archive.writestr(name, content)
        (self.edit / "edit.npz").write_bytes(buffer.getvalue())

    def build(self, name="out", edited=True, **options):
        return rebuild(self.base_file, self.config, self.tokenizer, self.root / name,
                       self.edit if edited else None, **options)


class RebuildTests(Fixture):
    def test_publishes_text_checkpoint_with_only_the_edit_changed(self):
        receipt = self.build()
        output = self.root / "out"
        content = (output / "model.safetensors").read_bytes()
        length = struct.unpack("<Q", content[:8])[0]
        header = json.loads(content[8:8 + length])
        self.assertEqual(length % 8, 0)
        self.assertNotIn(VISION, header)
        begin, end = header[EDITED]["data_offsets"]
        self.assertEqual(content[8 + length + begin:8 + length + end], bf16(self.edited_bits))
        begin, end = header[OTHER]["data_offsets"]
        self.assertEqual(content[8 + length + begin:8 + length + end], bf16([1, 2]))
        self.assertEqual(receipt["modelSha256"], sha(content))
        self.assertEqual(receipt["textTensors"], 2)
        self.assertEqual(receipt["edit"]["tensors"][0]["changedEntries"], 2)
        # Two of four unit entries gain 1.0: sqrt(2) / sqrt(4).
        self.assertAlmostEqual(receipt["edit"]["tensors"][0]["relativeChange"], 0.5 ** 0.5)
        self.assertEqual(json.loads((output / RECEIPT).read_text())["modelSha256"], receipt["modelSha256"])

    def test_unedited_checkpoint_has_the_same_shape_and_no_edit(self):
        receipt = self.build(edited=False)
        self.assertIsNone(receipt["edit"])
        self.assertEqual(receipt["textTensors"], 2)

    def assertRefused(self, message, **options):
        with self.assertRaisesRegex(ValueError, message):
            self.build(**options)
        self.assertFalse((self.root / "out").exists())
        self.assertEqual([path.name for path in self.root.glob(".weight-edit-*")], [])

    def test_changed_base_tensor_fails_closed(self):
        safetensors(self.base_file, {EDITED: ([2, 2], bf16([0x3F80, 0x3F80, 0x3F80, 0x4000])), OTHER: ([1, 2], bf16([1, 2]))})
        self.assertRefused("base tensor differs")

    def test_tampered_edit_value_fails_closed(self):
        self.write_edit(values=(0x4000, 0x4040))
        self.assertRefused("rebuilt tensor differs")

    def test_edit_archive_expansion_has_an_aggregate_bound(self):
        with mock.patch("replicate_weight_edit.MAX_EDIT_ARCHIVE_ARRAY_BYTES", 1):
            self.assertRefused("aggregate bound")

    def test_duplicate_edit_archive_member_is_refused(self):
        index_content = npy("<i4", "i", (0, 2))
        value_content = npy("<u2", "H", (0x4000, 0x4000))
        buffer = io.BytesIO()
        with self.assertWarns(UserWarning):
            with zipfile.ZipFile(buffer, "w", zipfile.ZIP_DEFLATED) as archive:
                archive.writestr("t0_idx.npy", index_content)
                archive.writestr("t0_idx.npy", index_content)
                archive.writestr("t0_val.npy", value_content)
        (self.edit / "edit.npz").write_bytes(buffer.getvalue())
        self.assertRefused("archive members differ")

    def test_wrong_pinned_base_fails_closed(self):
        self.assertRefused("pinned digest", expected_base="0" * 64)

    def test_lowrank_edit_is_refused(self):
        self.write_edit(record=self.record(form="lowrank"), members={
            "t0_idx.npy": npy("<i4", "i", (0,)), "t0_val.npy": npy("<u2", "H", (1,)),
            "t0_a.npy": b"", "t0_b.npy": b""})
        self.assertRefused("only sparse")

    def test_out_of_range_and_unordered_indices_are_refused(self):
        self.write_edit(indices=(0, 99))
        self.assertRefused("inside the tensor")
        self.write_edit(indices=(2, 0))
        self.assertRefused("must increase")

    def test_changed_entry_count_must_match_the_manifest(self):
        self.write_edit(record=self.record(changed_entries=3))
        self.assertRefused("different entry count")

    def test_pickled_or_extra_archive_members_are_refused(self):
        pickled = b"\x93NUMPY\x01\x00" + struct.pack("<H", 58) + b"{'descr': '|O', 'fortran_order': False, 'shape': (2,), }   "
        self.write_edit(members={"t0_idx.npy": pickled, "t0_val.npy": npy("<u2", "H", (1, 2))})
        self.assertRefused("one-dimensional")
        self.write_edit(members={"t0_idx.npy": npy("<i4", "i", (0, 2)), "t0_val.npy": npy("<u2", "H", (1, 2)), "x.npy": b""})
        self.assertRefused("archive members")

    def test_corrupt_archive_and_array_header_are_refused(self):
        (self.edit / "edit.npz").write_bytes(b"not a zip")
        self.assertRefused("not a zip")
        broken = b"\x93NUMPY\x01\x00" + struct.pack("<H", 3) + b"{{{"
        self.write_edit(members={"t0_idx.npy": broken, "t0_val.npy": npy("<u2", "H", (1, 2))})
        self.assertRefused("malformed")

    def test_shape_and_type_mismatch_are_refused(self):
        self.write_edit(record=self.record(shape=[4, 1]))
        self.assertRefused("type or shape")

    def test_malformed_safetensors_offsets_are_refused(self):
        safetensors(self.base_file, {EDITED: ([9, 9], bf16(self.base_bits))})
        self.assertRefused("invalid safetensors")
        header = json.dumps({EDITED: {"dtype": "BF16", "shape": "x", "data_offsets": 1}}).encode()
        self.base_file.write_bytes(struct.pack("<Q", len(header)) + header)
        self.assertRefused("invalid safetensors")

    def test_existing_output_is_not_overwritten(self):
        self.build()
        with self.assertRaisesRegex(ValueError, "already exists"):
            self.build()


class ExtractionTests(unittest.TestCase):
    def test_published_reading(self):
        for value, expected in [("a \\boxed{1} b \\boxed{2}", "2"), ("\\boxed{\\frac{1}{2}}", "\\frac{1}{2}"),
                                ("\\boxed{3}</think>none", None), ("x</think>\\boxed{4}", "4"),
                                ("\\boxed{5", None), ("\\boxed 6$ tail", "6"), ("\\fbox{7}", "7"), ("\\boxed{}", None),
                                ("\\boxed{8} \\fbox{9}", "8"), ("no answer", None)]:
            self.assertEqual(extract(value), expected, value)


def stub_generator(answers, finish="stop"):
    def generate(prompts, samples):
        return [[(f"\\boxed{{{answers(i, s)}}}", finish, 10) for s in range(samples)] for i in range(len(prompts))]
    return generate


def exact(prediction, gold):
    return prediction == gold


class EvaluationFixture(Fixture):
    def setUp(self):
        super().setUp()
        self.benchmark = self.root / "bench.jsonl"
        self.benchmark.write_bytes(b"".join(canonical({"id": f"p{i}", "problem": f"q{i}", "answer": "1"}) for i in range(4)))
        self.build("edited")
        self.build("base", edited=False)

    def run_arm(self, arm, checkpoint="edited", protocol="think35", answers=lambda i, s: 1, **options):
        return evaluate(self.root / checkpoint, self.benchmark, self.root / arm, "math500", protocol,
                        generate=options.pop("generate", stub_generator(answers)), is_correct=exact,
                        environment=lambda: {"test": True}, **options)


class EnvironmentTests(unittest.TestCase):
    def test_records_the_sampler_setting_but_no_other_engine_variable(self):
        with mock.patch.dict(os.environ, {"VLLM_USE_FLASHINFER_SAMPLER": "0", "VLLM_API_KEY": "secret-value"}):
            observed = observed_environment()
        self.assertEqual(observed["settings"], {"VLLM_USE_FLASHINFER_SAMPLER": "0"})
        self.assertNotIn("secret-value", json.dumps(observed))


class EvaluateTests(EvaluationFixture):
    def test_writes_the_published_layout_and_binds_the_job(self):
        result = self.run_arm("arm")
        self.assertEqual(result["math500"]["ok"], 16)
        self.assertEqual(result["_job"]["edited"], True)
        self.assertEqual(result["_job"]["baseFileSha256"], sha(self.base_file.read_bytes()))
        self.assertEqual(load_arm(self.root / "arm", "math500")["totals"]["total"], 16)
        self.assertEqual(result["_job"]["sampling"]["max_tokens"], 32768)

    def test_prompt_carries_the_published_instruction(self):
        seen = []
        def generate(prompts, samples):
            seen.extend(prompts)
            return stub_generator(lambda i, s: 1)(prompts, samples)
        self.run_arm("arm", generate=generate)
        self.assertEqual(seen[0], "q0\nPlease reason step by step, and put your final answer within \\boxed{}.")

    def test_identical_job_is_not_generated_again(self):
        self.run_arm("arm")
        def refuse(prompts, samples):
            raise AssertionError("must not generate")
        self.assertEqual(self.run_arm("arm", generate=refuse)["math500"]["ok"], 16)

    def test_different_job_cannot_replace_a_result(self):
        self.run_arm("arm")
        with self.assertRaisesRegex(ValueError, "different job"):
            self.run_arm("arm", protocol="nothink35")

    def test_abnormal_finish_saves_nothing(self):
        with self.assertRaisesRegex(ValueError, "nothing was saved"):
            self.run_arm("arm", generate=stub_generator(lambda i, s: 1, finish="abort"))
        self.assertFalse((self.root / "arm" / "parts" / "math500_s0of1").exists())

    def test_shards_cover_the_set_without_overlap(self):
        self.run_arm("arm", shard=(0, 2))
        self.run_arm("arm", shard=(1, 2))
        self.assertEqual(sorted(load_arm(self.root / "arm", "math500")["problems"]), ["p0", "p1", "p2", "p3"])

    def test_empty_selection_is_refused_before_generating(self):
        def refuse(prompts, samples):
            raise AssertionError("must not generate")
        with self.assertRaisesRegex(ValueError, "no problems"):
            self.run_arm("arm", limit=0, generate=refuse)
        with self.assertRaisesRegex(ValueError, "no problems"):
            self.run_arm("arm", shard=(9, 10), generate=refuse)

    def test_changed_checkpoint_is_refused(self):
        with (self.root / "edited" / "model.safetensors").open("ab") as target:
            target.write(b"x")
        with self.assertRaisesRegex(ValueError, "rebuild receipt"):
            self.run_arm("arm")

    def test_changed_tokenizer_or_configuration_is_refused(self):
        for name in ("tokenizer.json", "chat_template.jinja", "generation_config.json"):
            (self.root / "edited").rename(self.root / f"edited-before-{name}")
            self.build("edited")
            with (self.root / "edited" / name).open("ab") as target:
                target.write(b"x")
            with self.assertRaisesRegex(ValueError, "tokenizer or configuration"):
                self.run_arm("arm")

    def test_job_binds_the_tokenizer_and_configuration(self):
        job = self.run_arm("arm")["_job"]
        receipt = json.loads((self.root / "edited" / RECEIPT).read_text())
        self.assertEqual(job["tokenizerSha256"], receipt["tokenizerSha256"])
        self.assertEqual(job["configSha256"], receipt["configSha256"])


class CompareTests(EvaluationFixture):
    def test_paired_numbers_on_a_hand_checked_fixture(self):
        self.run_arm("before", checkpoint="base", answers=lambda i, s: 0)
        self.run_arm("after", answers=lambda i, s: 1 if i < 2 else 0)
        result = compare(self.root / "before", self.root / "after", "math500")
        # Per-problem differences in points are 100, 100, 0, 0: mean 50, sample sd 57.735, standard error 28.8675.
        self.assertAlmostEqual(result["difference"], 50.0)
        self.assertAlmostEqual(result["standardError"], 28.8675, places=3)
        self.assertEqual((result["improved"], result["regressed"], result["tied"]), (2, 0, 2))
        self.assertAlmostEqual(result["baseline"]["accuracy"], 0.0)
        self.assertAlmostEqual(result["candidate"]["accuracy"], 50.0)

    def test_different_problem_sets_are_refused(self):
        self.run_arm("before", shard=(0, 2))
        self.run_arm("after", shard=(1, 2))
        with self.assertRaisesRegex(ValueError, "different problems"):
            compare(self.root / "before", self.root / "after", "math500")

    def test_totals_must_match_per_problem_results(self):
        self.run_arm("arm")
        path = self.root / "arm" / "parts" / "math500_s0of1" / "scores.json"
        scores = json.loads(path.read_text())
        scores["math500"]["ok"] += 1
        path.write_bytes(canonical(scores))
        with self.assertRaisesRegex(ValueError, "recorded totals"):
            load_arm(self.root / "arm", "math500")

    def test_duplicate_problem_across_parts_is_refused(self):
        self.run_arm("arm", shard=(0, 2))
        source = self.root / "arm" / "parts" / "math500_s0of2"
        target = self.root / "arm" / "parts" / "math500_s1of2"
        target.mkdir()
        for path in source.iterdir():
            (target / path.name).write_bytes(path.read_bytes())
        with self.assertRaisesRegex(ValueError, "duplicate"):
            load_arm(self.root / "arm", "math500")


class DecideTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.names = ("small",)
        self.patcher = [("PROBLEMS", {"small": 40}), ("SAMPLES", {"small": 2})]
        import replicate_weight_edit as module
        self.module = module
        self.saved = {name: getattr(module, name) for name, _ in self.patcher}
        for name, value in self.patcher:
            setattr(module, name, value)
        self.addCleanup(lambda: [setattr(module, name, value) for name, value in self.saved.items()])

    def arm(self, name, correct_problems, job=None, base="b" * 64, count=40, set_name="small"):
        part = self.root / name / "parts" / f"{set_name}_s0of1"
        part.mkdir(parents=True)
        rows = [{"id": f"p{i}", "correct": [int(i < correct_problems)] * 2, "pred_first": None} for i in range(count)]
        ok = sum(sum(row["correct"]) for row in rows)
        scores = {set_name: {"ok": ok, "total": 2 * count, "cut": 0, "tokens_sum": 0,
                             "samples": self.module.SAMPLES[set_name]}}
        if job is not None:
            protocol = job.get("protocol", "think35")
            config = self.module.PROTOCOLS[protocol]
            record = {"protocol": protocol, "sampling": config["sampling"], "thinking": config["thinking"],
                      "samples": self.module.SAMPLES[set_name], "prompt": self.module.INSTRUCTION,
                      "benchmarkSha256": "c" * 64,
                      "checkpointModelSha256": "d" * 64 if job.get("edited") else "e" * 64,
                      "tokenizerSha256": "a" * 64,
                      "configSha256": {config_name: "a" * 64 for config_name in self.module.CONFIG_FILES},
                      "baseFileSha256": base, "edited": job.get("edited", False), "shard": [0, 1], "limit": None}
            scores["_job"] = {**record, **job}
        (part / f"{set_name}_per_problem.json").write_bytes(canonical(rows))
        (part / "scores.json").write_bytes(canonical(scores))
        return self.root / name

    def decide(self, edited=20, base=2, control=2, published=20, **options):
        published_arm = self.arm("published", published)
        edited_arm = self.arm("edited", edited, options.get("edited_job", {"edited": True, "protocol": "think35"}))
        base_arm = self.arm("base", base, {"edited": False, "protocol": "think35"})
        control_arm = self.arm("control", control, options.get("control_job", {"edited": False, "protocol": "nothink35"}),
                               base=options.get("control_base", "b" * 64))
        return decide(published_arm, edited_arm, base_arm, control_arm, self.names)

    def test_reproduced_and_better_than_the_setting_continues(self):
        result = self.decide()
        self.assertEqual((result["effect"], result["advantage"], result["decision"]),
                         ("reproduced", "edit_better_than_setting", "continue"))

    def test_setting_at_least_as_good_stops_even_when_reproduced(self):
        result = self.decide(control=20)
        self.assertEqual((result["effect"], result["advantage"], result["decision"]),
                         ("reproduced", "no_demonstrated_advantage", "stop"))

    def test_setting_clearly_better_stops(self):
        result = self.decide(control=38)
        self.assertEqual((result["advantage"], result["decision"]), ("setting_better_than_edit", "stop"))

    def test_gain_far_from_the_published_score_is_partial_and_stops(self):
        result = self.decide(edited=14, published=30)
        self.assertEqual((result["effect"], result["decision"]), ("partial", "stop"))

    def test_no_gain_is_not_reproduced_and_stops(self):
        result = self.decide(edited=2, base=2, published=2)
        self.assertEqual((result["effect"], result["decision"]), ("not_reproduced", "stop"))

    def test_incomplete_arm_cannot_decide(self):
        self.arm("short", 1, {"edited": True, "protocol": "think35"}, count=2)
        with self.assertRaisesRegex(ValueError, "does not cover"):
            decide(self.root / "short", self.root / "short", self.root / "short", self.root / "short", self.names)

    def test_published_layout_cannot_stand_in_for_a_replication_arm(self):
        with self.assertRaisesRegex(ValueError, "unsupported exchange fields"):
            self.decide(edited_job={"protocol": "think35", "ckpt": "x", "shard": "0/10"})

    def test_limited_run_cannot_decide(self):
        with self.assertRaisesRegex(ValueError, "not a complete"):
            self.decide(edited_job={"edited": True, "protocol": "think35", "limit": 2})

    def test_wrong_protocol_arm_cannot_decide(self):
        with self.assertRaisesRegex(ValueError, "not a complete"):
            self.decide(edited_job={"edited": True, "protocol": "nothink35"})

    def test_changed_sampling_parameters_cannot_decide(self):
        changed = dict(self.module.PROTOCOLS["think35"]["sampling"], temperature=0.1)
        with self.assertRaisesRegex(ValueError, "frozen run settings"):
            self.decide(edited_job={"edited": True, "protocol": "think35", "sampling": changed})

    def test_fewer_samples_cannot_decide(self):
        with self.assertRaisesRegex(ValueError, "frozen run settings"):
            self.decide(edited_job={"edited": True, "protocol": "think35", "samples": 1})

    def test_result_vectors_must_match_the_declared_sample_count(self):
        published = self.arm("published", 20)
        edited = self.arm("edited", 20, {"edited": True, "protocol": "think35"})
        base = self.arm("base", 2, {"edited": False, "protocol": "think35"})
        control = self.arm("control", 2, {"edited": False, "protocol": "nothink35"})
        part = edited / "parts" / "small_s0of1"
        rows_path = part / "small_per_problem.json"
        rows = json.loads(rows_path.read_text())
        for row in rows:
            row["correct"] = row["correct"][:1]
        rows_path.write_bytes(canonical(rows))
        scores_path = part / "scores.json"
        scores = json.loads(scores_path.read_text())
        scores["small"]["ok"] = sum(sum(row["correct"]) for row in rows)
        scores["small"]["total"] = sum(len(row["correct"]) for row in rows)
        scores_path.write_bytes(canonical(scores))
        with self.assertRaisesRegex(ValueError, "frozen run settings"):
            decide(published, edited, base, control, self.names)

    def test_duplicate_shard_identity_cannot_stand_in_for_a_complete_arm(self):
        arm = self.arm("duplicate-shard", 20, {"edited": True, "protocol": "think35"})
        original = arm / "parts" / "small_s0of1"
        rows = json.loads((original / "small_per_problem.json").read_text())
        original.rmdir() if not list(original.iterdir()) else None
        for path in list(original.iterdir()):
            path.unlink()
        original.rmdir()
        for index, selected in enumerate((rows[::3], rows[1::3], rows[2::3])):
            part = arm / "parts" / f"small_s{index}of2"
            part.mkdir()
            ok = sum(sum(row["correct"]) for row in selected)
            (part / "small_per_problem.json").write_bytes(canonical(selected))
            job = {"protocol": "think35", "sampling": self.module.PROTOCOLS["think35"]["sampling"],
                   "thinking": True, "samples": 2, "prompt": self.module.INSTRUCTION,
                   "benchmarkSha256": "c" * 64, "checkpointModelSha256": "d" * 64,
                   "tokenizerSha256": "a" * 64,
                   "configSha256": {config_name: "a" * 64 for config_name in self.module.CONFIG_FILES},
                   "baseFileSha256": "b" * 64, "edited": True, "shard": [min(index, 1), 2], "limit": None}
            scores = {"small": {"ok": ok, "total": 2 * len(selected), "cut": 0,
                                 "tokens_sum": 0, "samples": 2}, "_job": job}
            (part / "scores.json").write_bytes(canonical(scores))
        with self.assertRaisesRegex(ValueError, "complete, unique set of shards"):
            require_arm(arm, "small", True, "think35", None)

    def test_changed_prompt_cannot_decide(self):
        with self.assertRaisesRegex(ValueError, "frozen run settings"):
            self.decide(edited_job={"edited": True, "protocol": "think35", "prompt": "easier prompt"})

    def test_arms_must_share_benchmark_and_unedited_checkpoint(self):
        with self.assertRaisesRegex(ValueError, "different benchmark contents"):
            self.decide(edited_job={"edited": True, "protocol": "think35", "benchmarkSha256": "f" * 64})
        self.root = self.root / "different-checkpoint"
        self.root.mkdir()
        with self.assertRaisesRegex(ValueError, "different model checkpoints"):
            self.decide(control_job={"edited": False, "protocol": "nothink35",
                                     "checkpointModelSha256": "f" * 64})

    def test_arms_must_use_one_checkpoint_across_benchmark_sets(self):
        self.module.PROBLEMS["other"] = 40
        self.module.SAMPLES["other"] = 2
        for set_name in ("small", "other"):
            self.arm("published", 20, set_name=set_name)
            edited_job = {"edited": True, "protocol": "think35"}
            if set_name == "other":
                edited_job["checkpointModelSha256"] = "f" * 64
            self.arm("edited", 20, edited_job, set_name=set_name)
            self.arm("base", 2, {"edited": False, "protocol": "think35"}, set_name=set_name)
            self.arm("control", 2, {"edited": False, "protocol": "nothink35"}, set_name=set_name)
        with self.assertRaisesRegex(ValueError, "across benchmark sets"):
            decide(self.root / "published", self.root / "edited", self.root / "base", self.root / "control",
                   ("small", "other"))

    def test_arms_from_different_bases_cannot_decide(self):
        with self.assertRaisesRegex(ValueError, "different base"):
            self.decide(control_base="c" * 64)


class PrepareTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        part = self.root / "published" / "parts" / "math500_s0of1"
        part.mkdir(parents=True)
        (part / "math500_per_problem.json").write_bytes(canonical(
            [{"id": "p0", "correct": [0, 0], "pred_first": None}, {"id": "p1", "correct": [0, 0], "pred_first": None}]))
        (part / "scores.json").write_bytes(canonical({"math500": {"ok": 0, "total": 4, "cut": 0, "tokens_sum": 0}}))
        (part / "math500_first_samples.jsonl.gz").write_bytes(gzip.compress(canonical(
            {"id": "p0", "answer": "1", "text": ""})))

    def table(self, answer="1", ids=("p0", "p1")):
        return {"math500": [{"id": i, "problem": "q", "answer": answer} for i in ids]}

    def test_matching_benchmark_is_written_with_its_digest(self):
        result = prepare(self.root / "out", self.root / "published", self.table())
        content = (self.root / "out" / "math500.jsonl").read_bytes()
        self.assertEqual(result["math500"], {"problems": 2, "sha256": sha(content)})

    def test_changed_problems_or_gold_answers_are_refused_before_writing(self):
        with self.assertRaisesRegex(ValueError, "problems differ"):
            prepare(self.root / "out", self.root / "published", self.table(ids=("p0", "p9")))
        with self.assertRaisesRegex(ValueError, "gold answer differs"):
            prepare(self.root / "out", self.root / "published", self.table(answer="2"))
        self.assertFalse((self.root / "out").exists())


class RegradeTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.part = Path(temporary.name) / "parts" / "math500_s0of1"
        self.part.mkdir(parents=True)

    def write(self, recorded_answer, recorded_correct):
        (self.part / "math500_per_problem.json").write_bytes(canonical(
            [{"id": "p", "correct": [recorded_correct, 0], "pred_first": recorded_answer}]))
        (self.part / "math500_first_samples.jsonl.gz").write_bytes(gzip.compress(canonical(
            {"id": "p", "answer": "1", "text": "\\boxed{1}"})))
        return self.part.parent.parent

    def test_agreement(self):
        result = regrade(self.write("1", 1), "math500", exact)
        self.assertEqual((result["checked"], result["extractionAgree"], result["gradeAgree"]), (1, 1, 1))
        self.assertEqual(result["disagreements"], [])

    def test_disagreements_are_reported_not_hidden(self):
        result = regrade(self.write("2", 0), "math500", exact)
        self.assertEqual((result["extractionAgree"], result["gradeAgree"]), (0, 0))
        self.assertEqual(result["disagreements"][0]["observed"], "1")

    def test_extraction_only_needs_no_grader(self):
        result = regrade(self.write("1", 1), "math500")
        self.assertIsNone(result["gradeAgree"])


if __name__ == "__main__":
    unittest.main()
