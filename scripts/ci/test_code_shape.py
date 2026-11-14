#!/usr/bin/env python3
"""Regressions for the code-shape ratchet."""

import unittest

from check_code_shape import (
    ENTRY_LIMIT,
    HARNESS_LIMIT,
    SOURCE_LIMIT,
    Shape,
    classify,
    compare,
    measure,
    tighten,
)


def lines(count: int) -> str:
    return "".join("let x = 1;\n" for _ in range(count))


class ClassifyTests(unittest.TestCase):
    def test_entry_harness_and_source(self):
        self.assertEqual(classify("crates/kernel/rrd-core/src/lib.rs"), "entry")
        self.assertEqual(classify("crates/adapters/rrflow-cli/src/main.rs"), "entry")
        self.assertEqual(classify("crates/kernel/rrd-core/src/engine/lib.rs"), "source")
        self.assertEqual(classify("crates/kernel/rrd-core/tests/runtime.rs"), "harness")
        self.assertEqual(classify("crates/kernel/rrd-core/src/engine/tests/query.rs"), "harness")
        self.assertEqual(classify("crates/kernel/rrd-core/src/access/versioned_read_tests.rs"), "harness")
        self.assertEqual(classify("crates/kernel/rrd-core/src/access/tests.rs"), "harness")
        self.assertEqual(classify("crates/kernel/rrd-core/examples/bench.rs"), "harness")
        self.assertEqual(classify("crates/kernel/rrd-core/src/runtime.rs"), "source")


class MeasureTests(unittest.TestCase):
    def test_each_kind_has_its_own_limit(self):
        shape = measure({
            "crates/a/x/src/lib.rs": lines(ENTRY_LIMIT + 1),
            "crates/a/x/src/big.rs": lines(SOURCE_LIMIT + 1),
            "crates/a/x/src/fine.rs": lines(SOURCE_LIMIT),
            "crates/a/x/tests/long.rs": lines(HARNESS_LIMIT),
        })
        self.assertEqual(shape.oversize, {"crates/a/x/src/lib.rs": ENTRY_LIMIT + 1, "crates/a/x/src/big.rs": SOURCE_LIMIT + 1})

    def test_patterns_are_counted_outside_harness_files_only(self):
        text = ("fn a() -> Result<u8, String> { todo!() }\n"
                "fn b() -> Box<dyn std::error::Error> { todo!() }\n"
                "#[allow(clippy::too_many_arguments)]\n"
                "#[path = \"x_tests.rs\"] mod t;\n"
                "let h = sha2::Sha256::digest(b\"x\");\n")
        shape = measure({"crates/a/x/src/thing/mod.rs": text, "crates/a/x/tests/t.rs": text,
                         "crates/kernel/rrd-core/src/digest.rs": "sha2::Sha256::new()\n"})
        path = "crates/a/x/src/thing/mod.rs"
        for name in ("mod_rs", "path_attribute", "string_error", "boxed_error", "too_many_arguments_allow", "sha2_crate"):
            self.assertEqual(shape.patterns[name], {path: 1}, name)


class RatchetTests(unittest.TestCase):
    def setUp(self):
        self.baseline = Shape(oversize={"crates/a/x/src/big.rs": 900},
                              patterns={"string_error": {"crates/a/x/src/e.rs": 3}})

    def test_unchanged_passes(self):
        self.assertEqual(compare(self.baseline, Shape.from_json(self.baseline.to_json())), [])

    def test_a_new_oversize_file_fails(self):
        current = Shape(oversize={"crates/a/x/src/big.rs": 900, "crates/a/x/src/new.rs": 801},
                        patterns={"string_error": {"crates/a/x/src/e.rs": 3}})
        self.assertTrue(any("new.rs: 801 lines exceeds" in p for p in compare(self.baseline, current)))

    def test_growth_fails_and_cannot_be_tightened_away(self):
        current = Shape(oversize={"crates/a/x/src/big.rs": 950}, patterns={"string_error": {"crates/a/x/src/e.rs": 4}})
        problems = compare(self.baseline, current)
        self.assertTrue(any("grew from 900 to 950" in p for p in problems))
        self.assertTrue(any("rose from 3 to 4" in p for p in problems))
        tightened = tighten(self.baseline, current)
        self.assertEqual(tightened.oversize["crates/a/x/src/big.rs"], 900)
        self.assertEqual(tightened.patterns["string_error"]["crates/a/x/src/e.rs"], 3)
        self.assertTrue(compare(tightened, current), "tightening never admits growth")

    def test_a_new_pattern_site_fails(self):
        current = Shape(oversize={"crates/a/x/src/big.rs": 900},
                        patterns={"string_error": {"crates/a/x/src/e.rs": 3, "crates/a/x/src/f.rs": 1}})
        self.assertTrue(any("f.rs: string_error rose from 0 to 1" in p for p in compare(self.baseline, current)))

    def test_improvement_must_be_recorded_and_then_holds(self):
        current = Shape(oversize={}, patterns={"string_error": {"crates/a/x/src/e.rs": 1}})
        problems = compare(self.baseline, current)
        self.assertTrue(any("no longer over its limit" in p for p in problems))
        self.assertTrue(any("fell from 3 to 1" in p for p in problems))
        tightened = tighten(self.baseline, current)
        self.assertEqual(compare(tightened, current), [])
        regressed = Shape(oversize={"crates/a/x/src/big.rs": 801}, patterns={"string_error": {"crates/a/x/src/e.rs": 2}})
        self.assertEqual(len(compare(tightened, regressed)), 2, "a fixed file cannot regress")


if __name__ == "__main__":
    unittest.main()
