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

    def test_cluster_speakers_can_detect_three_groups(self) -> None:
        features = [
            (0.0, 1.0, 35.0, 0.020, 60.0),
            (1.0, 2.0, 38.0, 0.021, 63.0),
            (2.0, 3.0, 42.0, 0.024, 66.0),
            (3.0, 4.0, 68.0, 0.055, 110.0),
            (4.0, 5.0, 72.0, 0.058, 116.0),
            (5.0, 6.0, 76.0, 0.060, 122.0),
            (6.0, 7.0, 108.0, 0.090, 172.0),
            (7.0, 8.0, 112.0, 0.094, 178.0),
            (8.0, 9.0, 118.0, 0.097, 184.0),
        ]
        segments = _cluster_speakers(features)
        speakers = {s.speaker_id for s in segments}
        self.assertEqual(speakers, {"Speaker 1", "Speaker 2", "Speaker 3"})


if __name__ == "__main__":
    unittest.main()
