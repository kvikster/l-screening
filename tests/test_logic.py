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


class TestPartialBlankMatch(unittest.TestCase):
    """Regression: only sample_1 has a blank match; sample_2 is too far from the blank in RT."""

    def _make_df(self) -> pd.DataFrame:
        # sample_1: RT≈2.00, 2 replicates (files 1_a, 1_b)
        # sample_2: RT≈2.08, 2 replicates (files 2_a, 2_b) — within 0.1 of sample_1 → will merge
        # blank:    RT≈1.95, 2 replicates (files blank_a, blank_b)
        #   |sample_1 − blank| = 0.05 ≤ 0.1 → MATCHES
        #   |sample_2 − blank| = 0.13 > 0.1 → NO MATCH
        return pd.DataFrame([
            {"RT": 1.99, "Base Peak": 150.0, "Area": 500.0, "Polarity": "+", "File": "1_a", "Label": "s1a"},
            {"RT": 2.01, "Base Peak": 150.1, "Area": 510.0, "Polarity": "+", "File": "1_b", "Label": "s1b"},
            {"RT": 2.07, "Base Peak": 150.0, "Area": 480.0, "Polarity": "+", "File": "2_a", "Label": "s2a"},
            {"RT": 2.09, "Base Peak": 150.1, "Area": 490.0, "Polarity": "+", "File": "2_b", "Label": "s2b"},
            {"RT": 1.94, "Base Peak": 150.0, "Area": 100.0, "Polarity": "+", "File": "blank_a", "Label": "b1"},
            {"RT": 1.96, "Base Peak": 150.1, "Area": 110.0, "Polarity": "+", "File": "blank_b", "Label": "b2"},
        ])

    def test_partial_blank_match_sources_with_blank_count(self) -> None:
        _, _, results = MODULE.process_peaks(self._make_df())
        merged = next(r for r in results if r["ParallelMatch"])
        blank_sub = merged["Why"]["BlankSubtraction"]
        self.assertEqual(blank_sub["SourcesWithBlankMatch"], 1)
        self.assertEqual(blank_sub["TotalSources"], 2)

    def test_partial_blank_match_agg_blank_area_from_sample1_only(self) -> None:
        _, _, results = MODULE.process_peaks(self._make_df())
        merged = next(r for r in results if r["ParallelMatch"])
        # blank cluster area_mean ≈ (100+110)/2 = 105; weighted by sample_1 replicate_count=2
        self.assertIsNotNone(merged["BlankAreaMean"])
        self.assertAlmostEqual(float(merged["BlankAreaMean"]), 105.0, places=1)

    def test_partial_blank_match_ratio_and_status(self) -> None:
        _, _, results = MODULE.process_peaks(self._make_df())
        merged = next(r for r in results if r["ParallelMatch"])
        # area_mean ≈ (505*2 + 485*2)/4 = 495; blank≈105 → ratio≈4.71 > 3.0 → Real Compound
        self.assertIsNotNone(merged["SignalToBlankRatio"])
        self.assertGreater(float(merged["SignalToBlankRatio"]), 3.0)
        self.assertEqual(merged["Status"], "Real Compound")

    def test_partial_blank_match_per_source_detail(self) -> None:
        _, _, results = MODULE.process_peaks(self._make_df())
        merged = next(r for r in results if r["ParallelMatch"])
        per_source = merged["Why"]["BlankSubtraction"]["PerSource"]
        self.assertEqual(len(per_source), 2)
        s1 = next(s for s in per_source if s["SampleType"] == "sample_1")
        s2 = next(s for s in per_source if s["SampleType"] == "sample_2")
        self.assertIsNotNone(s1["BlankAreaMean"])
        self.assertIsNone(s2["BlankAreaMean"])
        self.assertEqual(s2["Status"], "Real Compound")  # no blank match → Real Compound


class TestSurrogateValidation(unittest.TestCase):
    def _make_df(self, surrogate_area: float = 143_250.0, surrogate_rt: float = 5.31) -> pd.DataFrame:
        return pd.DataFrame([
            {
                "RT": surrogate_rt,
                "Base Peak": 128.0,
                "Area": surrogate_area,
                "Polarity": "+",
                "File": "surrogate_a",
                "Label": "d8-Naphthalene",
                "operator_mark": "surrogate",
            },
            {
                "RT": surrogate_rt + 0.01,
                "Base Peak": 128.1,
                "Area": surrogate_area * 1.02,
                "Polarity": "+",
                "File": "surrogate_b",
                "Label": "d8-Naphthalene",
                "operator_mark": "surrogate",
            },
        ])

    def _config(self, **overrides):
        config = {
            "surrogates": [
                {
                    "name": "d8-Naphthalene",
                    "expected_rt": 5.23,
                    "expected_area": 150000.0,
                    "expected_mz": 128.0,
                    "rt_window": 0.2,
                    "recovery_min_pct": 70.0,
                    "recovery_max_pct": 130.0,
                }
            ]
        }
        config.update(overrides)
        return config

    def test_surrogate_happy_path(self) -> None:
        _, _, results = MODULE.process_peaks(self._make_df(), self._config())
        surrogate = results[0]
        self.assertTrue(surrogate["IsSurrogate"])
        self.assertTrue(surrogate["SurrogatePass"])
        self.assertAlmostEqual(float(surrogate["SurrogateRecoveryPct"]), 96.5, places=1)
        self.assertAlmostEqual(float(surrogate["SurrogateRtShift"]), 0.085, places=3)
        self.assertEqual(surrogate["Status"], "Surrogate OK")
        self.assertTrue(surrogate["Why"]["SurrogateValidation"]["Pass"])

    def test_surrogate_recovery_fail_still_in_output(self) -> None:
        _, _, results = MODULE.process_peaks(self._make_df(surrogate_area=400_000.0), self._config())
        surrogate = results[0]
        self.assertTrue(surrogate["IsSurrogate"])
        self.assertFalse(surrogate["SurrogatePass"])
        self.assertEqual(surrogate["Status"], "Surrogate Failed")
        self.assertGreater(float(surrogate["SurrogateRecoveryPct"]), 130.0)

    def test_surrogate_without_matching_spec(self) -> None:
        _, _, results = MODULE.process_peaks(self._make_df(), {"surrogates": []})
        surrogate = results[0]
        self.assertTrue(surrogate["IsSurrogate"])
        self.assertIsNone(surrogate["SurrogatePass"])
        self.assertEqual(surrogate["Status"], "Surrogate")
        self.assertNotIn("SurrogateValidation", surrogate["Why"])

    def test_surrogate_rt_drift_fails(self) -> None:
        _, _, results = MODULE.process_peaks(self._make_df(surrogate_rt=5.45), self._config())
        surrogate = results[0]
        self.assertTrue(surrogate["IsSurrogate"])
        self.assertFalse(surrogate["SurrogatePass"])
        self.assertEqual(surrogate["Status"], "Surrogate Failed")
        self.assertGreater(abs(float(surrogate["SurrogateRtShift"])), 0.2)


if __name__ == "__main__":
    unittest.main()
