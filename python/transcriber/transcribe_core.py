from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import json
import re
import time
from typing import Any, Sequence


UNKNOWN_SPEAKER = "Unknown"
THAI_CHAR_RE = re.compile(r"[\u0E00-\u0E7F]")


@dataclass
class TranscriptSegment:
    start_sec: float
    end_sec: float
    text_en: str
    source_language: str
    speaker_id: str | None


@dataclass
class TranscriptResult:
    session_id: str
    segments: list[TranscriptSegment]
    full_text_en: str
    model_version: str
    processing_ms: int


def _format_ts(seconds: float) -> str:
    total = max(0, int(seconds))
    mm = total // 60
    ss = total % 60
    return f"{mm:02d}:{ss:02d}"


def result_to_payload(result: TranscriptResult) -> dict[str, Any]:
    return {
        "sessionId": result.session_id,
        "segments": [
            {
                "startSec": segment.start_sec,
                "endSec": segment.end_sec,
                "textEn": segment.text_en,
                "sourceLanguage": segment.source_language,
                "speakerId": segment.speaker_id,
            }
            for segment in result.segments
        ],
        "fullTextEn": result.full_text_en,
        "modelVersion": result.model_version,
        "processingMs": result.processing_ms,
    }


def write_transcript_json(result: TranscriptResult, output_path: Path) -> None:
    payload = result_to_payload(result)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")


def write_transcript_txt(result: TranscriptResult, output_path: Path) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    lines: list[str] = []
    for segment in result.segments:
        speaker = segment.speaker_id or UNKNOWN_SPEAKER
        lines.append(
            f"[{_format_ts(segment.start_sec)}-{_format_ts(segment.end_sec)}] {speaker}: {segment.text_en}"
        )
    output_path.write_text("\n".join(lines).strip() + "\n", encoding="utf-8")


def _overlap(a_start: float, a_end: float, b_start: float, b_end: float) -> float:
    return max(0.0, min(a_end, b_end) - max(a_start, b_start))


def assign_speakers(
    transcript_segments: Sequence[TranscriptSegment],
    analysis_segments: Sequence[dict[str, Any]],
) -> list[TranscriptSegment]:
    if not analysis_segments:
        return list(transcript_segments)

    enriched: list[TranscriptSegment] = []
    for segment in transcript_segments:
        best_speaker = None
        best_overlap = 0.0
        for diarized in analysis_segments:
            overlap = _overlap(
                segment.start_sec,
                segment.end_sec,
                float(diarized.get("startSec", 0.0)),
                float(diarized.get("endSec", 0.0)),
            )
            if overlap > best_overlap:
                best_overlap = overlap
                best_speaker = diarized.get("speakerId")

        duration = max(0.0, segment.end_sec - segment.start_sec)
        overlap_ratio = best_overlap / duration if duration > 0 else 0.0
        speaker = str(best_speaker) if best_speaker and overlap_ratio >= 0.25 else UNKNOWN_SPEAKER
        enriched.append(
            TranscriptSegment(
                start_sec=segment.start_sec,
                end_sec=segment.end_sec,
                text_en=segment.text_en,
                source_language=segment.source_language,
                speaker_id=speaker,
            )
        )

    return enriched


def _load_analysis_segments(analysis_path: Path | None) -> list[dict[str, Any]]:
    if analysis_path is None or not analysis_path.exists():
        return []
    payload = json.loads(analysis_path.read_text(encoding="utf-8"))
    segments = payload.get("segments", [])
    if isinstance(segments, list):
        return [segment for segment in segments if isinstance(segment, dict)]
    return []


def _build_model(model_size: str) -> tuple[Any, str]:
    try:
        from faster_whisper import WhisperModel
    except ModuleNotFoundError as exc:
        raise RuntimeError(
            "Missing transcription dependency 'faster-whisper'. Install with: pip install -r python/requirements.txt"
        ) from exc

    model = WhisperModel(model_size, device="cpu", compute_type="int8")
    return model, f"faster-whisper-{model_size}"


def _count_thai_chars(value: str) -> int:
    return len(THAI_CHAR_RE.findall(value))


def _segments_from_generated(generated_segments: Any, default_language: str) -> tuple[list[TranscriptSegment], list[str]]:
    transcript_segments: list[TranscriptSegment] = []
    full_text_chunks: list[str] = []
    for generated in generated_segments:
        text = (getattr(generated, "text", "") or "").strip()
        if not text:
            continue
        seg_lang = getattr(generated, "language", None) or default_language
        transcript_segments.append(
            TranscriptSegment(
                start_sec=float(getattr(generated, "start", 0.0)),
                end_sec=float(getattr(generated, "end", 0.0)),
                text_en=text,
                source_language=str(seg_lang),
                speaker_id=None,
            )
        )
        full_text_chunks.append(text)
    return transcript_segments, full_text_chunks


def transcribe_wav(
    input_path: Path,
    output_json_path: Path,
    output_txt_path: Path,
    analysis_path: Path | None,
    model_size: str = "medium",
) -> TranscriptResult:
    if not input_path.exists():
        raise FileNotFoundError(f"input wav does not exist: {input_path}")

    started = time.time()
    model, model_version = _build_model(model_size)

    generated_segments, info = model.transcribe(
        str(input_path),
        task="translate",
        beam_size=5,
        vad_filter=True,
        initial_prompt="Translate all speech into natural English. Do not output Thai script.",
    )

    source_lang = getattr(info, "language", None) or "unknown"
    transcript_segments, full_text_chunks = _segments_from_generated(generated_segments, str(source_lang))
    combined_text = " ".join(full_text_chunks).strip()

    # Mixed-language meetings can leave Thai tokens in translate mode.
    # Fallback to Thai-forced translation and keep the cleaner English output.
    if _count_thai_chars(combined_text) > 0:
        th_segments, th_info = model.transcribe(
            str(input_path),
            task="translate",
            language="th",
            beam_size=5,
            vad_filter=True,
            initial_prompt="Translate all speech into natural English. Do not output Thai script.",
        )
        th_source_lang = getattr(th_info, "language", None) or "th"
        th_transcript_segments, th_full_text_chunks = _segments_from_generated(th_segments, str(th_source_lang))
        th_combined_text = " ".join(th_full_text_chunks).strip()
        if _count_thai_chars(th_combined_text) < _count_thai_chars(combined_text):
            transcript_segments = th_transcript_segments
            full_text_chunks = th_full_text_chunks
            combined_text = th_combined_text

    analysis_segments = _load_analysis_segments(analysis_path)
    with_speakers = assign_speakers(transcript_segments, analysis_segments)

    result = TranscriptResult(
        session_id=input_path.parent.name,
        segments=with_speakers,
        full_text_en=combined_text,
        model_version=model_version,
        processing_ms=int((time.time() - started) * 1000),
    )

    write_transcript_json(result, output_json_path)
    write_transcript_txt(result, output_txt_path)
    return result
