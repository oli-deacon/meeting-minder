from __future__ import annotations

from pathlib import Path
import json
import math
import struct
import tempfile
import unittest
import wave

import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "python" / "analyzer"))

from core import analyze_wav  # noqa: E402


def _write_test_wav(path: Path, sample_rate: int = 16000) -> None:
    samples: list[int] = []

    def add_sine(seconds: float, freq: float, amp: int = 6000) -> None:
        count = int(sample_rate * seconds)
        for i in range(count):
            value = int(amp * math.sin(2 * math.pi * freq * (i / sample_rate)))
            samples.append(value)

    def add_silence(seconds: float) -> None:
        samples.extend([0] * int(sample_rate * seconds))

    add_sine(1.2, 180.0)
    add_silence(0.4)
    add_sine(1.1, 240.0)

    with wave.open(str(path), "wb") as wf:
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(sample_rate)
        raw = struct.pack(f"<{len(samples)}h", *samples)
        wf.writeframes(raw)


class AnalyzerIntegrationTests(unittest.TestCase):
    def test_analyze_wav_writes_expected_schema(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            base = Path(temp_dir)
            session_dir = base / "session-123"
            session_dir.mkdir(parents=True, exist_ok=True)
            wav_path = session_dir / "audio.wav"
            out_path = session_dir / "analysis.json"
            _write_test_wav(wav_path)

            result = analyze_wav(wav_path, out_path)

            self.assertTrue(out_path.exists())
            payload = json.loads(out_path.read_text(encoding="utf-8"))
            self.assertEqual(payload["sessionId"], "session-123")
            self.assertIn("speakers", payload)
            self.assertIn("segments", payload)
            self.assertIn("meta", payload)
            self.assertGreaterEqual(result.total_speech_sec, 1.0)


if __name__ == "__main__":
    unittest.main()
