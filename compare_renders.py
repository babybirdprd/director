#!/usr/bin/env python3
"""Compare frames pixel-by-pixel between our render and official render."""

import sys
from pathlib import Path
from PIL import Image
import numpy as np


def compare_frames(ours_path, official_path, frame_num):
    """Compare two frames and return pixel difference percentage."""
    try:
        img1 = Image.open(ours_path).convert("RGB")
        img2 = Image.open(official_path).convert("RGB")

        # Ensure same size
        if img1.size != img2.size:
            img2 = img2.resize(img1.size, Image.Resampling.LANCZOS)

        arr1 = np.array(img1)
        arr2 = np.array(img2)

        # Calculate difference
        diff = np.abs(arr1.astype(float) - arr2.astype(float))
        mean_diff = np.mean(diff)
        max_diff = np.max(diff)

        # Count pixels with significant difference (>10/255)
        significant_diff = np.sum(diff > 10) / diff.size * 100

        print(f"Frame {frame_num}:")
        print(
            f"  Mean pixel difference: {mean_diff:.1f}/255 ({mean_diff / 255 * 100:.1f}%)"
        )
        print(f"  Max pixel difference: {max_diff:.1f}/255")
        print(f"  Pixels with >10 diff: {significant_diff:.1f}%")

        if significant_diff < 5:
            print(f"  Status: VERY SIMILAR")
        elif significant_diff < 15:
            print(f"  Status: MODERATE DIFFERENCE")
        else:
            print(f"  Status: SIGNIFICANT DIFFERENCE")

        return mean_diff
    except Exception as e:
        print(f"Error comparing frame {frame_num}: {e}")
        return None


def check_animation_progression(frames_dir):
    """Check if frames show animation progression."""
    frames = sorted(Path(frames_dir).glob("frame_*.png"))
    if len(frames) < 2:
        print("Not enough frames to check progression")
        return

    print(f"\nAnimation progression check ({len(frames)} frames):")
    prev_hash = None
    identical_count = 0

    for i, frame_path in enumerate(frames):
        img = Image.open(frame_path).convert("RGB")
        current_hash = hash(np.array(img).tobytes())

        if prev_hash is not None:
            if current_hash == prev_hash:
                identical_count += 1
                status = "IDENTICAL"
            else:
                status = "DIFFERENT"
        else:
            status = "FIRST"

        print(f"  Frame {i}: {status}")
        prev_hash = current_hash

    if identical_count == len(frames) - 1:
        print(f"  WARNING: All frames are identical - animation is static!")
    else:
        print(
            f"  GOOD: {len(frames) - identical_count - 1}/{len(frames) - 1} frames show changes"
        )


def main():
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument("--ours", default="/tmp/frames_ours")
    parser.add_argument("--official", default="/tmp/frames_official")
    args = parser.parse_args()

    print("=" * 60)
    print("COMPARING OUR RENDER VS OFFICIAL RENDER")
    print("=" * 60)

    # Compare each frame
    ours_frames = sorted(Path(args.ours).glob("frame_*.png"))
    official_frames = sorted(Path(args.official).glob("frame_*.png"))

    for i, (ours, official) in enumerate(zip(ours_frames, official_frames)):
        compare_frames(ours, official, i)

    # Check animation progression in our render
    print("\n" + "=" * 60)
    check_animation_progression(args.ours)


if __name__ == "__main__":
    main()
