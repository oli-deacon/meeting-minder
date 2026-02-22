from __future__ import annotations

import argparse
from pathlib import Path

from core import analyze_wav


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Analyze meeting audio and compute per-speaker time.")
    parser.add_argument("--input", required=True, help="Input wav file path.")
    parser.add_argument("--output", required=True, help="Output analysis json file path.")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    input_path = Path(args.input)
    output_path = Path(args.output)

    if not input_path.exists():
        raise FileNotFoundError(f"input wav does not exist: {input_path}")

    analyze_wav(input_path=input_path, output_path=output_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
