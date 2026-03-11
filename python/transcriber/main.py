from __future__ import annotations

import argparse
from pathlib import Path

from transcribe_core import transcribe_wav


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Transcribe meeting WAV audio offline and translate non-English speech to English."
    )
    parser.add_argument("--input", required=True, help="Input wav file path.")
    parser.add_argument("--output-json", required=True, help="Output transcript json file path.")
    parser.add_argument("--output-txt", required=True, help="Output transcript txt file path.")
    parser.add_argument("--analysis", required=False, help="Optional analysis json path for speaker labeling.")
    parser.add_argument("--model-size", default="medium", help="Whisper model size (default: medium).")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    input_path = Path(args.input)
    output_json_path = Path(args.output_json)
    output_txt_path = Path(args.output_txt)
    analysis_path = Path(args.analysis) if args.analysis else None

    transcribe_wav(
        input_path=input_path,
        output_json_path=output_json_path,
        output_txt_path=output_txt_path,
        analysis_path=analysis_path,
        model_size=args.model_size,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
