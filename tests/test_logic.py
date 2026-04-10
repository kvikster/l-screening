from __future__ import annotations

import importlib.util
import pathlib
import sys
import unittest

import pandas as pd


ROOT = pathlib.Path(__file__).resolve().parents[1]
LOGIC_PATH = ROOT / "server/python-version/logic.py"
SPEC = importlib.util.spec_from_file_location("screening_logic", LOGIC_PATH)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class LogicPipelineTests(unittest.TestCase):
    def test_parallel_merge_tracks_actual_seed_row_index(self) -> None:
        df = pd.DataFrame(
            [
                {"RT": 1.00, "Base Peak": 100.00, "Area": 50.0, "Polarity": "+", "File": "1_a", "Label": "s1a_a"},
                {"RT": 1.01, "Base Peak": 100.02, "Area": 52.0, "Polarity": "+", "File": "1_b", "Label": "s1b_a"},
                {"RT": 2.00, "Base Peak": 200.00, "Area": 200.0, "Polarity": "+", "File": "1_a", "Label": "s1a_b"},
                {"RT": 2.01, "Base Peak": 200.01, "Area": 190.0, "Polarity": "+", "File": "2_a", "Label": "s2a_b"},
            ]
        )

        _, _, results = MODULE.process_peaks(df)

        self.assertEqual(len(results), 2)

        parallel = next(row for row in results if row["ParallelMatch"])
        standalone = next(row for row in results if not row["ParallelMatch"])

        self.assertEqual(parallel["ParallelSourceSamples"], ["sample_1", "sample_2"])
        self.assertEqual(parallel["ReplicateCount"], 2)
        self.assertAlmostEqual(parallel["RT_mean"], 2.005, places=3)

        self.assertEqual(standalone["ParallelSourceSamples"], ["sample_1"])
        self.assertEqual(standalone["ReplicateCount"], 2)
        self.assertAlmostEqual(standalone["RT_mean"], 1.005, places=3)

    def test_parallel_merge_keeps_unmatched_and_blank_matches_stay_visible(self) -> None:
        df = pd.DataFrame(
            [
                {"RT": 1.00, "Base Peak": 100.00, "Area": 100.0, "Polarity": "+", "File": "1_a", "Label": "s1a"},
                {"RT": 1.01, "Base Peak": 100.05, "Area": 110.0, "Polarity": "+", "File": "1_b", "Label": "s1b"},
                {"RT": 2.00, "Base Peak": 200.00, "Area": 50.0, "Polarity": "+", "File": "1_a", "Label": "s1a"},
                {"RT": 2.01, "Base Peak": 200.04, "Area": 55.0, "Polarity": "+", "File": "1_b", "Label": "s1b"},
                {"RT": 1.00, "Base Peak": 100.02, "Area": 90.0, "Polarity": "+", "File": "2_a", "Label": "s2a"},
                {"RT": 1.02, "Base Peak": 100.03, "Area": 95.0, "Polarity": "+", "File": "2_b", "Label": "s2b"},
                {"RT": 1.00, "Base Peak": 100.01, "Area": 80.0, "Polarity": "+", "File": "blank_a", "Label": "b1"},
                {"RT": 1.01, "Base Peak": 100.02, "Area": 85.0, "Polarity": "+", "File": "blank_b", "Label": "b2"},
            ]
        )

        results_df, summary_df, results = MODULE.process_peaks(df)

        self.assertEqual(len(results), 2)
        merged = next(row for row in results if row["ParallelMatch"])
        singleton = next(row for row in results if not row["ParallelMatch"])

        self.assertEqual(merged["SampleType"], "sample")
        self.assertEqual(merged["ParallelSampleCount"], 2)
        self.assertEqual(merged["ReplicateCount"], 4)
        self.assertAlmostEqual(merged["Area_mean"], 98.75, places=2)
        self.assertEqual(merged["Status"], "Artifact")
        self.assertIsNotNone(merged["BlankAreaMean"])
        self.assertIn("BlankDetail", merged["Why"])

        self.assertEqual(singleton["Status"], "Real Compound")
        self.assertFalse(singleton["ParallelMatch"])
        self.assertEqual(singleton["ReplicateCount"], 2)

        sample_summary = summary_df[summary_df["Sample"] == "sample"].iloc[0]
        self.assertEqual(int(sample_summary["Confirmed"]), 2)
        self.assertEqual(int(sample_summary["Artifacts"]), 1)
        self.assertEqual(int(sample_summary["RealCompounds"]), 1)
        self.assertFalse(results_df.empty)

    def test_rt_only_mode_when_base_peak_is_missing(self) -> None:
        df = pd.DataFrame(
            [
                {"RT": 3.00, "Area": 100.0, "Polarity": "+", "File": "1_a", "Label": "s1a"},
                {"RT": 3.02, "Area": 105.0, "Polarity": "+", "File": "1_b", "Label": "s1b"},
                {"RT": 3.01, "Area": 98.0, "Polarity": "+", "File": "2_a", "Label": "s2a"},
                {"RT": 3.03, "Area": 102.0, "Polarity": "+", "File": "2_b", "Label": "s2b"},
            ]
        )

        _, _, results = MODULE.process_peaks(df)

        self.assertEqual(len(results), 1)
        result = results[0]
        self.assertEqual(result["MatchingMode"], "RT")
        self.assertIsNone(result["MZ_mean"])
        self.assertEqual(result["Status"], "Real Compound")

    def test_parallel_merge_only_merges_within_tolerance(self) -> None:
        df = pd.DataFrame(
            [
                {"RT": 1.00, "Base Peak": 100.00, "Area": 10.0, "Polarity": "+", "File": "1_a", "Label": "s1a_1"},
                {"RT": 1.00, "Base Peak": 100.00, "Area": 10.0, "Polarity": "+", "File": "1_b", "Label": "s1b_1"},
                {"RT": 1.00, "Base Peak": 100.00, "Area": 10.0, "Polarity": "+", "File": "1_c", "Label": "s1c_1"},
                {"RT": 2.00, "Base Peak": 200.00, "Area": 100.0, "Polarity": "+", "File": "2_a", "Label": "s2a_1"},
            ]
        )

        _, _, results = MODULE.process_peaks(df)

        # RT delta (1.0 min) exceeds default replicate_rt_tol (0.1),
        # so the two sample buckets must NOT be merged.
        self.assertEqual(len(results), 2)
        by_rt = sorted(results, key=lambda r: float(r["RT_mean"]))
        s1_cluster, s2_singleton = by_rt

        self.assertAlmostEqual(s1_cluster["RT_mean"], 1.0, places=2)
        self.assertEqual(s1_cluster["ReplicateCount"], 3)
        self.assertFalse(s1_cluster["ParallelMatch"])

        self.assertAlmostEqual(s2_singleton["RT_mean"], 2.0, places=2)
        self.assertEqual(s2_singleton["ReplicateCount"], 1)
        self.assertFalse(s2_singleton["ParallelMatch"])


if __name__ == "__main__":
    unittest.main()
