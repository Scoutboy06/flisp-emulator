#!/usr/bin/env python3

import subprocess
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent
GOLDEN_DIR = ROOT / "golden_files"
QAFLISP = ROOT / "qaflisp"


def run_qaflisp(test_dir: Path, input_file: Path):
    result = subprocess.run(
        [str(QAFLISP), input_file.name],
        cwd=test_dir,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    if result.returncode != 0:
        print(f"[ERROR] qaflisp failed in {test_dir}")
        print(result.stdout)
        print(result.stderr)
        sys.exit(1)


def main():
    if not QAFLISP.exists():
        print("[ERROR] qaflisp executable not found")
        sys.exit(1)

    sflisp_files = sorted(GOLDEN_DIR.rglob("*.sflisp"))

    if not sflisp_files:
        print("[ERROR] No .sflisp files found")
        sys.exit(1)

    print(f"[INFO] Found {len(sflisp_files)} test cases")

    for sflisp in sflisp_files:
        test_dir = sflisp.parent
        print(f"[INFO] Generating golden files in {test_dir.relative_to(ROOT)}")

        run_qaflisp(test_dir, sflisp)

    print("[INFO] Golden file generation complete")


if __name__ == "__main__":
    main()
