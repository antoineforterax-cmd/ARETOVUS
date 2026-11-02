#!/usr/bin/env python3
"""D2 regression vectors for the deterministic model, not physical benchmarks."""
import unittest

from scale_cost_model import DEFAULT_HEIGHTS, estimate


class StorageScaleModelTests(unittest.TestCase):
    def test_historical_small_reference_vectors(self) -> None:
        self.assertEqual(estimate(8192)["minimal_existing_path_metadata_checks_modelled"], 528512)
        self.assertEqual(estimate(100001)["minimal_existing_path_metadata_checks_modelled"], 78226556)

    def test_large_heights_are_exact_not_runs(self) -> None:
        expected = [
            (1_000_000, 15625, 7_813_015_625),
            (10_000_000, 156250, 781_255_156_250),
            (100_000_000, 1562500, 78_125_051_562_500),
            (500_000_000, 7812500, 1_953_125_257_812_500),
        ]
        self.assertEqual(tuple(e[0] for e in expected), DEFAULT_HEIGHTS)
        for height, pages, checks in expected:
            with self.subTest(height=height):
                result = estimate(height)
                self.assertTrue(result["model_only"])
                self.assertEqual(result["snapshot_calls_modelled"], pages)
                self.assertEqual(result["minimal_existing_path_metadata_checks_modelled"], checks)
                self.assertNotIn("actual_disk", result)

    def test_partial_final_page(self) -> None:
        self.assertEqual(estimate(1)["minimal_existing_path_metadata_checks_modelled"], 2)
        self.assertEqual(estimate(65)["minimal_existing_path_metadata_checks_modelled"], 131)

    def test_assumed_raw_size_is_not_measurement(self) -> None:
        result = estimate(1_000_000, assumed_bytes_per_block=1024)
        self.assertTrue(result["model_only"])
        self.assertEqual(result["illustrative_raw_block_bytes"], 1_000_001 * 1024)
        self.assertIn("filesystem overhead", result["raw_size_excludes"])

    def test_invalid_inputs_rejected(self) -> None:
        for bad in (0, -1):
            with self.assertRaises(ValueError):
                estimate(bad)
        with self.assertRaises(ValueError):
            estimate(1, page_size=0)
        with self.assertRaises(ValueError):
            estimate(1, assumed_bytes_per_block=0)


if __name__ == "__main__":
    unittest.main()
