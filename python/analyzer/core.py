from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import json
import math
import struct
import time
import wave
from typing import Iterable, List, Sequence


@dataclass
class Segment:
    start_sec: float
    end_sec: float
    speaker_id: str


@dataclass
class SpeakerStats:
    speaker_id: str
    total_sec: float
    percentage: float
    segment_count: int


@dataclass
class AnalysisMeta:
    total_speech_sec: float
    processing_ms: int
    model_version: str


@dataclass
class AnalysisResult:
    session_id: str
    total_speech_sec: float
    speakers: List[SpeakerStats]
    segments: List[Segment]
    meta: AnalysisMeta


def result_to_payload(result: AnalysisResult) -> dict:
    return {
        "sessionId": result.session_id,
        "totalSpeechSec": result.total_speech_sec,
        "speakers": [
            {
                "speakerId": s.speaker_id,
                "totalSec": s.total_sec,
                "percentage": s.percentage,
                "segmentCount": s.segment_count,
            }
            for s in result.speakers
        ],
        "segments": [
            {
                "startSec": s.start_sec,
                "endSec": s.end_sec,
                "speakerId": s.speaker_id,
            }
            for s in result.segments
        ],
        "meta": {
            "totalSpeechSec": result.meta.total_speech_sec,
            "processingMs": result.meta.processing_ms,
            "modelVersion": result.meta.model_version,
        },
    }


def write_analysis_json(result: AnalysisResult, output_path: Path) -> None:
    payload = result_to_payload(result)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")


def _read_mono_pcm16(path: Path) -> tuple[list[int], int]:
    with wave.open(str(path), "rb") as wf:
        sample_rate = wf.getframerate()
        channels = wf.getnchannels()
        sample_width = wf.getsampwidth()
        frame_count = wf.getnframes()
        raw = wf.readframes(frame_count)

    if sample_width != 2:
        raise ValueError("expected 16-bit PCM wav input")

    total_samples = len(raw) // 2
    if total_samples == 0:
        return [], sample_rate

    samples = struct.unpack(f"<{total_samples}h", raw)
    if channels == 1:
        return list(samples), sample_rate

    mono: list[int] = []
    for idx in range(0, len(samples), channels):
        frame = samples[idx : idx + channels]
        mono.append(int(sum(frame) / len(frame)))
    return mono, sample_rate


def _frame_samples(samples: list[int], frame_size: int) -> list[list[int]]:
    out: list[list[int]] = []
    for offset in range(0, len(samples), frame_size):
        frame = samples[offset : offset + frame_size]
        if len(frame) == frame_size:
            out.append(frame)
    return out


def _rms(frame: Sequence[int]) -> float:
    if not frame:
        return 0.0
    power = sum(sample * sample for sample in frame) / len(frame)
    return math.sqrt(power)


def _zcr(frame: Sequence[int]) -> float:
    if len(frame) < 2:
        return 0.0
    crossing = 0
    prev = frame[0]
    for sample in frame[1:]:
        if (sample >= 0 > prev) or (sample < 0 <= prev):
            crossing += 1
        prev = sample
    return crossing / len(frame)


def _mean_abs_diff(frame: Sequence[int]) -> float:
    if len(frame) < 2:
        return 0.0
    return sum(abs(frame[i] - frame[i - 1]) for i in range(1, len(frame))) / (len(frame) - 1)


def _voice_activity_segments(
    samples: list[int], sample_rate: int
) -> list[tuple[float, float, float, float, float]]:
    frame_samples = max(1, int(sample_rate * 0.03))
    frames = _frame_samples(samples, frame_samples)
    if not frames:
        return []

    rms_values = [_rms(frame) for frame in frames]
    sorted_rms = sorted(rms_values)
    n = len(sorted_rms)

    def percentile(q: float) -> float:
        idx = int(q * (n - 1))
        return sorted_rms[max(0, min(n - 1, idx))]

    p10 = percentile(0.10)
    p90 = percentile(0.90)
    # Adaptive VAD threshold based on clip energy spread:
    # threshold = noise floor + fraction of dynamic range.
    # This is robust for low-gain recordings where absolute RMS can be small.
    threshold = max(20.0, p10 + (p90 - p10) * 0.35)

    segments: list[tuple[int, int]] = []
    in_speech = False
    start_idx = 0
    for idx, rms in enumerate(rms_values):
        speaking = rms >= threshold
        if speaking and not in_speech:
            in_speech = True
            start_idx = idx
        if not speaking and in_speech:
            segments.append((start_idx, idx))
            in_speech = False
    if in_speech:
        segments.append((start_idx, len(rms_values)))

    frame_duration = frame_samples / sample_rate
    enriched: list[tuple[float, float, float, float, float]] = []
    chunk_samples = max(1, int(sample_rate * 1.0))
    min_chunk_samples = max(1, int(sample_rate * 0.35))
    for start_idx, end_idx in segments:
        start_sec = start_idx * frame_duration
        end_sec = end_idx * frame_duration
        duration = end_sec - start_sec
        if duration < 0.25:
            continue

        start_sample = start_idx * frame_samples
        end_sample = min(len(samples), end_idx * frame_samples)
        cursor = start_sample
        while cursor < end_sample:
            chunk_end = min(end_sample, cursor + chunk_samples)
            if chunk_end - cursor < min_chunk_samples:
                break
            chunk = samples[cursor:chunk_end]
            chunk_start_sec = cursor / sample_rate
            chunk_end_sec = chunk_end / sample_rate
            chunk_rms = _rms(chunk)
            chunk_zcr = _zcr(chunk)
            chunk_diff = _mean_abs_diff(chunk)
            enriched.append((chunk_start_sec, chunk_end_sec, chunk_rms, chunk_zcr, chunk_diff))
            cursor = chunk_end

    return enriched


def _cluster_speakers(vad_segments: Sequence[tuple[float, float, float, float, float]]) -> list[Segment]:
    if not vad_segments:
        return []

    if len(vad_segments) < 4:
        return [
            Segment(start_sec=start, end_sec=end, speaker_id="Speaker 1")
            for start, end, _, _, _ in vad_segments
        ]

    points: list[tuple[float, float, float]] = []
    for _, _, rms, zcr, diff in vad_segments:
        points.append((math.log1p(rms), zcr, diff))

    dims = 3
    means = [sum(p[i] for p in points) / len(points) for i in range(dims)]
    stdevs: list[float] = []
    for i in range(dims):
        var = sum((p[i] - means[i]) ** 2 for p in points) / len(points)
        stdevs.append(max(1e-6, math.sqrt(var)))

    norm_points = [tuple((p[i] - means[i]) / stdevs[i] for i in range(dims)) for p in points]

    seed_a = min(norm_points, key=lambda p: p[0])
    seed_b = max(norm_points, key=lambda p: p[0])
    c1 = [seed_a[0], seed_a[1], seed_a[2]]
    c2 = [seed_b[0], seed_b[1], seed_b[2]]

    labels = [0] * len(norm_points)
    for _ in range(20):
        changed = False
        for idx, point in enumerate(norm_points):
            d1 = sum((point[k] - c1[k]) ** 2 for k in range(dims))
            d2 = sum((point[k] - c2[k]) ** 2 for k in range(dims))
            next_label = 0 if d1 <= d2 else 1
            if next_label != labels[idx]:
                labels[idx] = next_label
                changed = True

        for cluster in (0, 1):
            members = [norm_points[i] for i, lab in enumerate(labels) if lab == cluster]
            if members:
                centroid = [sum(m[k] for m in members) / len(members) for k in range(dims)]
                if cluster == 0:
                    c1 = centroid
                else:
                    c2 = centroid
        if not changed:
            break

    counts = [sum(1 for l in labels if l == 0), sum(1 for l in labels if l == 1)]
    separation = math.sqrt(sum((c1[k] - c2[k]) ** 2 for k in range(dims)))
    min_ratio = min(counts) / len(labels)
    should_split = separation >= 1.15 and min_ratio >= 0.18 and min(counts) >= 2

    if not should_split:
        return [
            Segment(start_sec=start, end_sec=end, speaker_id="Speaker 1")
            for start, end, _, _, _ in vad_segments
        ]

    # Remove one-off flicker labels between same-speaker neighbors.
    for idx in range(1, len(labels) - 1):
        if labels[idx - 1] == labels[idx + 1] and labels[idx] != labels[idx - 1]:
            labels[idx] = labels[idx - 1]

    speakered: list[Segment] = []
    for idx, (start, end, _, _, _) in enumerate(vad_segments):
        speakered.append(
            Segment(start_sec=start, end_sec=end, speaker_id="Speaker 1" if labels[idx] == 0 else "Speaker 2")
        )

    return speakered


def merge_adjacent_segments(
    segments: Iterable[Segment],
    max_gap_sec: float = 0.2,
    min_duration_sec: float = 0.2,
) -> list[Segment]:
    ordered = sorted(segments, key=lambda s: (s.start_sec, s.end_sec))
    if not ordered:
        return []

    merged: list[Segment] = [ordered[0]]
    for segment in ordered[1:]:
        prev = merged[-1]
        gap = segment.start_sec - prev.end_sec
        if segment.speaker_id == prev.speaker_id and gap <= max_gap_sec:
            prev.end_sec = max(prev.end_sec, segment.end_sec)
        else:
            merged.append(segment)

    return [s for s in merged if (s.end_sec - s.start_sec) >= min_duration_sec]


def calculate_speaker_stats(segments: Sequence[Segment]) -> tuple[list[SpeakerStats], float]:
    totals: dict[str, float] = {}
    counts: dict[str, int] = {}
    for segment in segments:
        duration = max(0.0, segment.end_sec - segment.start_sec)
        totals[segment.speaker_id] = totals.get(segment.speaker_id, 0.0) + duration
        counts[segment.speaker_id] = counts.get(segment.speaker_id, 0) + 1

    total_speech = sum(totals.values())
    speakers: list[SpeakerStats] = []
    for speaker_id, total_sec in sorted(totals.items()):
        pct = (total_sec / total_speech * 100.0) if total_speech > 0 else 0.0
        speakers.append(
            SpeakerStats(
                speaker_id=speaker_id,
                total_sec=round(total_sec, 4),
                percentage=round(pct, 4),
                segment_count=counts[speaker_id],
            )
        )
    return speakers, round(total_speech, 4)


def analyze_wav(input_path: Path, output_path: Path) -> AnalysisResult:
    started = time.time()
    samples, sample_rate = _read_mono_pcm16(input_path)

    vad_segments = _voice_activity_segments(samples, sample_rate)
    speaker_segments = _cluster_speakers(vad_segments)
    merged = merge_adjacent_segments(speaker_segments)
    speakers, total_speech = calculate_speaker_stats(merged)

    processing_ms = int((time.time() - started) * 1000)
    session_id = input_path.parent.name
    result = AnalysisResult(
        session_id=session_id,
        total_speech_sec=total_speech,
        speakers=speakers,
        segments=merged,
        meta=AnalysisMeta(
            total_speech_sec=total_speech,
            processing_ms=processing_ms,
            model_version="heuristic-v2",
        ),
    )

    write_analysis_json(result, output_path)
    return result
