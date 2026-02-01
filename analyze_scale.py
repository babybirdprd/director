#!/usr/bin/env python3
"""Measure scale animation by analyzing red square size in frames."""

import subprocess
import sys
from pathlib import Path


def extract_frames(video_path, output_dir):
    """Extract all frames from video."""
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Clean old frames
    for f in output_dir.glob("frame_*.png"):
        f.unlink()

    cmd = [
        "ffmpeg",
        "-y",
        "-i",
        str(video_path),
        "-vf",
        "fps=30",
        str(output_dir / "frame_%03d.png"),
    ]
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"Error extracting frames: {result.stderr}")
        return False
    return True


def measure_red_square(frame_path):
    """Measure the size of the red square in the frame."""
    try:
        from PIL import Image
        import numpy as np

        img = Image.open(frame_path).convert("RGB")
        arr = np.array(img)

        # Find red pixels (R>200, G<50, B<50)
        red_mask = (arr[:, :, 0] > 200) & (arr[:, :, 1] < 50) & (arr[:, :, 2] < 50)

        if not red_mask.any():
            return 0, 0

        # Calculate bounding box of red pixels
        rows = np.any(red_mask, axis=1)
        cols = np.any(red_mask, axis=0)
        height = np.sum(rows)
        width = np.sum(cols)

        return width, height
    except Exception as e:
        print(f"Error measuring {frame_path}: {e}")
        return 0, 0


def analyze_video(video_path, label):
    """Analyze a video file for scale animation."""
    print(f"\n{'=' * 60}")
    print(f"Analyzing: {label}")
    print(f"{'=' * 60}")

    output_dir = f"/tmp/analysis_{label.replace(' ', '_')}"

    if not extract_frames(video_path, output_dir):
        return

    # Measure at key frames
    frames = [0, 10, 20, 30]
    print(f"\nFrame measurements:")
    print(f"{'Frame':<8} {'Width':<8} {'Height':<8} {'Expected':<10} {'Status'}")
    print("-" * 50)

    for frame_num in frames:
        frame_path = Path(output_dir) / f"frame_{frame_num:03d}.png"
        if not frame_path.exists():
            print(f"{frame_num:<8} NOT FOUND")
            continue

        width, height = measure_red_square(frame_path)
        size = max(width, height)

        # Expected size: 100 + (frame_num / 30.0) * 200
        expected = 100 + (frame_num / 30.0) * 200

        # Allow 10% tolerance
        if abs(size - expected) < expected * 0.1:
            status = "OK"
        elif frame_num == 0 and size > 0:
            status = "OK (start)"
        else:
            status = "FAIL"

        print(f"{frame_num:<8} {width:<8} {height:<8} {expected:<10.0f} {status}")

    # Count unique sizes
    sizes = []
    for frame_path in sorted(Path(output_dir).glob("frame_*.png")):
        w, h = measure_red_square(frame_path)
        if w > 0:
            sizes.append((w, h))

    unique_sizes = set(sizes)
    print(f"\nTotal frames: {len(sizes)}")
    print(f"Unique sizes: {len(unique_sizes)}")

    if len(unique_sizes) <= 3:
        print("WARNING: Animation appears static (very few unique sizes)")
    elif len(unique_sizes) >= 10:
        print("GOOD: Animation shows variation (many unique sizes)")


def main():
    import argparse

    parser = argparse.ArgumentParser(description="Analyze Lottie scale animation")
    parser.add_argument("video", help="Path to video file")
    parser.add_argument("--label", default="Test", help="Label for output")
    args = parser.parse_args()

    analyze_video(args.video, args.label)


if __name__ == "__main__":
    main()
