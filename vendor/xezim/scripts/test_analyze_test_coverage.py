"""Regression checks for failures during coverage profiling."""
import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location(
    "coverage_analyzer", Path(__file__).with_name("analyze_test_coverage.py")
)
analyzer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(analyzer)


class ProfileFailures(unittest.TestCase):
    def run_unit(self, root, script):
        program = root / "unit.py"
        program.write_text(script)
        return analyzer.run_one_unit(
            root, "misc", Path(sys.executable), "misc::profile_probe",
            str(program), "test", root / "profiles", "unused", "unused"
        )

    def test_failed_test_aborts_analysis_and_removes_partial_profiles(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(RuntimeError, "profile_probe.*exit code 7"):
                self.run_unit(root, "raise SystemExit(7)\n")
            self.assertEqual(list((root / "profiles").iterdir()), [])

    def test_success_without_instrumentation_aborts_analysis(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(RuntimeError, "no coverage profile"):
                self.run_unit(root, "pass\n")
            self.assertEqual(list((root / "profiles").iterdir()), [])

    def test_profiled_success_keeps_measured_regions(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            regions = {("runtime.rs", 10, 1)}
            with mock.patch.object(analyzer, "extract_covered_regions", return_value=regions):
                result = self.run_unit(root, "import os\nfrom pathlib import Path\nPath(os.environ['LLVM_PROFILE_FILE']).write_text('profile')\n")
            self.assertEqual(result[4], regions)
            self.assertEqual(list((root / "profiles").iterdir()), [])


if __name__ == "__main__":
    unittest.main()
