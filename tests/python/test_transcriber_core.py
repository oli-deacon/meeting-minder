from __future__ import annotations

from pathlib import Path
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "python" / "transcriber"))

from transcribe_core import (  # noqa: E402
    TranscriptResult,
    TranscriptSegment,
    _count_thai_chars,
    assign_speakers,
    result_to_payload,
    write_transcript_txt,
)


class TranscriberCoreTests(unittest.TestCase):
    def test_count_thai_chars(self) -> None:
        self.assertEqual(_count_thai_chars("hello"), 0)
        self.assertGreater(_count_thai_chars("สวัสดี"), 0)

    def test_result_to_payload_schema(self) -> None:
        result = TranscriptResult(
            session_id="session-abc",
            segments=[
                TranscriptSegment(
                    start_sec=0.0,
                    end_sec=1.2,
                    text_en="Hello team",
                    source_language="en",
                    speaker_id="Speaker 1",
                )
            ],
            full_text_en="Hello team",
            model_version="faster-whisper-medium",
            processing_ms=345,
        )

        payload = result_to_payload(result)
        self.assertEqual(payload["sessionId"], "session-abc")
        self.assertEqual(payload["modelVersion"], "faster-whisper-medium")
        self.assertEqual(payload["segments"][0]["sourceLanguage"], "en")
        self.assertEqual(payload["segments"][0]["speakerId"], "Speaker 1")

    def test_assign_speakers_by_overlap(self) -> None:
        transcript = [
            TranscriptSegment(0.0, 1.0, "One", "en", None),
            TranscriptSegment(1.0, 2.0, "Two", "th", None),
            TranscriptSegment(2.0, 2.2, "Tiny", "th", None),
        ]
        analysis = [
            {"startSec": 0.0, "endSec": 1.2, "speakerId": "Speaker 1"},
            {"startSec": 1.2, "endSec": 2.0, "speakerId": "Speaker 2"},
        ]

        assigned = assign_speakers(transcript, analysis)
        self.assertEqual(assigned[0].speaker_id, "Speaker 1")
        self.assertEqual(assigned[1].speaker_id, "Speaker 2")
        self.assertEqual(assigned[2].speaker_id, "Unknown")

    def test_write_transcript_txt_includes_language_preserved_in_source_data(self) -> None:
        result = TranscriptResult(
            session_id="session-xyz",
            segments=[
                TranscriptSegment(10.0, 12.0, "Good morning", "en", "Speaker 1"),
                TranscriptSegment(12.0, 14.5, "How are you", "th", "Speaker 2"),
            ],
            full_text_en="Good morning How are you",
            model_version="faster-whisper-medium",
            processing_ms=100,
        )

        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "transcript.txt"
            write_transcript_txt(result, path)
            text = path.read_text(encoding="utf-8")
            self.assertIn("Speaker 1: Good morning", text)
            self.assertIn("Speaker 2: How are you", text)


if __name__ == "__main__":
    unittest.main()
