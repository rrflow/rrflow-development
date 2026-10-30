"""Projection corruption probes use the actual exported catalogue, not routing data."""

import copy
import json
import unittest

from check_generated_surfaces import contract_export
from rust_client_projection import render


class RustProjectionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.raw, cls.catalogue = contract_export("--catalogue")

    def encode(self, catalogue):
        return render(json.dumps(catalogue).encode(), catalogue)

    def selected(self):
        return next(e for e in self.catalogue["endpoints"] if
                    e["authentication"] == "session_bearer" and
                    e["method"] == "post" and "{" not in e["path"])

    def test_repeat_generation_is_byte_identical(self):
        self.assertEqual(render(self.raw, self.catalogue), render(self.raw, self.catalogue))

    def test_changed_owner_metadata_changes_projection(self):
        catalogue = copy.deepcopy(self.catalogue)
        selected = self.selected()
        endpoint = next(e for e in catalogue["endpoints"] if e["operation"] == selected["operation"])
        endpoint["path"] += "/denial-probe"
        output = self.encode(catalogue)
        self.assertIn(endpoint["path"].encode(), output)
        endpoint["operation"] += "-denial-probe"
        self.assertIn(endpoint["operation"].encode(), self.encode(catalogue))
        catalogue["endpoints"].remove(endpoint)
        self.assertNotIn(endpoint["operation"].encode(), self.encode(catalogue))

    def test_duplicate_operation_or_invalid_type_fails(self):
        catalogue = copy.deepcopy(self.catalogue)
        catalogue["endpoints"].append(copy.deepcopy(self.selected()))
        with self.assertRaisesRegex(ValueError, "duplicate"):
            self.encode(catalogue)
        catalogue = copy.deepcopy(self.catalogue)
        endpoint = next(e for e in catalogue["endpoints"] if e == self.selected())
        endpoint["request_type"] = 'Injected; panic!("not code")'
        with self.assertRaisesRegex(ValueError, "wire type"):
            self.encode(catalogue)

    def test_empty_supported_surface_is_not_success(self):
        with self.assertRaisesRegex(ValueError, "no supported"):
            self.encode({"endpoints": []})


if __name__ == "__main__":
    unittest.main()
