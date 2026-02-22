from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "python" / "analyzer"))

from core import Segment, _cluster_speakers, calculate_speaker_stats, merge_adjacent_segments  # noqa: E402


class AnalyzerCoreTests(unittest.TestCase):
    def test_merge_adjacent_segments_same_speaker(self) -> None:
        segments = [
            Segment(start_sec=0.0, end_sec=1.0, speaker_id="Speaker 1"),
            Segment(start_sec=1.1, end_sec=2.0, speaker_id="Speaker 1"),
            Segment(start_sec=2.4, end_sec=2.8, speaker_id="Speaker 2"),
        ]
        merged = merge_adjacent_segments(segments, max_gap_sec=0.2)
        self.assertEqual(len(merged), 2)
        self.assertAlmostEqual(merged[0].start_sec, 0.0)
        self.assertAlmostEqual(merged[0].end_sec, 2.0)

    def test_calculate_speaker_stats_percentages(self) -> None:
        segments = [
            Segment(start_sec=0.0, end_sec=3.0, speaker_id="Speaker 1"),
            Segment(start_sec=3.0, end_sec=5.0, speaker_id="Speaker 2"),
        ]
        stats, total = calculate_speaker_stats(segments)
        self.assertAlmostEqual(total, 5.0)
        self.assertEqual(len(stats), 2)
        self.assertAlmostEqual(stats[0].percentage, 60.0)
        self.assertAlmostEqual(stats[1].percentage, 40.0)

    def test_cluster_speakers_detects_two_groups(self) -> None:
        # (start, end, rms, zcr, diff)
        features = [
            (0.0, 1.0, 40.0, 0.02, 70.0),
            (1.0, 2.0, 45.0, 0.025, 72.0),
            (2.0, 3.0, 43.0, 0.022, 69.0),
            (3.0, 4.0, 110.0, 0.08, 170.0),
            (4.0, 5.0, 120.0, 0.085, 176.0),
            (5.0, 6.0, 115.0, 0.082, 173.0),
        ]
        segments = _cluster_speakers(features)
        speakers = {s.speaker_id for s in segments}
        self.assertEqual(speakers, {"Speaker 1", "Speaker 2"})


if __name__ == "__main__":
    unittest.main()
