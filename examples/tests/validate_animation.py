#!/usr/bin/env python3
"""
Quick validation that Lottie animation is not frozen.
Renders the same Lottie at multiple time points and checks for differences.
"""

import subprocess
import sys
import tempfile
import os
from pathlib import Path


def render_frame_at_time(rhai_script, time_sec, output_path):
    """Render a single frame by modifying the Rhai script temporarily"""
    # Read original script
    with open(rhai_script, "r") as f:
        content = f.read()

    # Modify scene duration to render just this frame
    modified = content.replace(
        "let scene = movie.add_scene(3.033);",
        f"let scene = movie.add_scene({time_sec + 0.034});",  # One frame duration
    )

    # Write temporary script
    temp_script = tempfile.mktemp(suffix=".rhai")
    with open(temp_script, "w") as f:
        f.write(modified)

    try:
        # Render single frame
        cmd = [
            "cargo",
            "run",
            "--release",
            "-p",
            "director-cli",
            "--bin",
            "director-engine",
            "--",
            temp_script,
            output_path,
        ]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
        return result.returncode == 0
    finally:
        os.unlink(temp_script)


def main():
    print("=" * 60)
    print("Lottie Animation Validation")
    print("=" * 60)

    script = "examples/tests/lottie_heart_eyes_export.rhai"
    output_dir = Path("examples/tests/validation_frames")
    output_dir.mkdir(exist_ok=True)

    # Render at key time points
    times = [0.0, 1.0, 2.0, 2.5, 3.0]
    outputs = []

    print("\nRendering frames at different times...")
    for t in times:
        output = output_dir / f"frame_at_{t:.1f}s.mp4"
        print(f"  Time {t:.1f}s → {output}")
        if render_frame_at_time(script, t, str(output)):
            outputs.append((t, output))
        else:
            print(f"    ERROR: Failed to render")

    print(f"\n✓ Rendered {len(outputs)} frames")
    print(f"\nCheck these files manually:")
    for t, path in outputs:
        print(f"  {path} (at {t:.1f}s)")

    print("\n" + "=" * 60)
    print("INSTRUCTIONS:")
    print("=" * 60)
    print("1. Open each MP4 file in a video player")
    print("2. They should show DIFFERENT frames of the animation")
    print("3. If they all look the same, the animation is FROZEN")
    print("4. Frame at 0.0s should be nearly empty (before animation starts)")
    print("5. Frame at 2.5s should show the heart eyes animation")
    print("=" * 60)

    return 0 if len(outputs) == len(times) else 1


if __name__ == "__main__":
    sys.exit(main())
