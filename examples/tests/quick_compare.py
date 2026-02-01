#!/usr/bin/env python3
"""
Quick comparison of Lottie renders - updated version with better output
"""

import sys
import subprocess
import tempfile
import os
from pathlib import Path
from PIL import Image
import numpy as np


def extract_frames(video_path, frame_nums):
    """Extract specific frames from video using ffmpeg"""
    frames = {}

    for frame_num in frame_nums:
        with tempfile.NamedTemporaryFile(suffix=".png", delete=False) as tmp:
            tmp_path = tmp.name

        cmd = [
            "ffmpeg",
            "-y",
            "-i",
            video_path,
            "-vf",
            f"select=eq(n\\,{frame_num})",
            "-vframes",
            "1",
            tmp_path,
        ]

        try:
            result = subprocess.run(cmd, capture_output=True, text=True)
            if result.returncode == 0 and Path(tmp_path).exists():
                img = Image.open(tmp_path)
                frames[frame_num] = img.copy()
                img.close()
        except Exception as e:
            print(f"Error extracting frame {frame_num}: {e}")
        finally:
            if Path(tmp_path).exists():
                try:
                    os.unlink(tmp_path)
                except:
                    pass

    return frames


def compare_frames(official_img, our_img):
    """Compare two frames and return statistics"""
    official = np.array(official_img).astype(np.float32)
    ours = np.array(our_img).astype(np.float32)

    diff = np.abs(official - ours)
    diff_gray = np.mean(diff, axis=2)

    mean_diff = np.mean(diff_gray)
    max_diff = np.max(diff_gray)
    diff_pixels = np.sum(diff_gray > 10)
    total_pixels = diff_gray.size
    diff_percent = (diff_pixels / total_pixels) * 100

    return mean_diff, max_diff, diff_percent


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <official.mp4> <ours.mp4>")
        sys.exit(1)

    official_path = sys.argv[1]
    ours_path = sys.argv[2]

    # Key frames to analyze
    key_frames = [0, 15, 30, 45, 60, 75, 90]

    print(f"Extracting frames from both videos...")
    print(f"  Official: {official_path}")
    official_frames = extract_frames(official_path, key_frames)

    print(f"  Ours: {ours_path}")
    ours_frames = extract_frames(ours_path, key_frames)

    print(f"\nComparing frames...")
    print(
        f"{'Frame':>6} | {'Mean Diff':>10} | {'Max Diff':>9} | {'% Diff':>7} | {'Status':>10}"
    )
    print(f"{'-' * 60}")

    all_mean = []
    all_max = []

    for frame_num in key_frames:
        if frame_num not in official_frames or frame_num not in ours_frames:
            continue

        official_img = official_frames[frame_num]
        our_img = ours_frames[frame_num]

        # Ensure same size
        if official_img.size != our_img.size:
            our_img = our_img.resize(official_img.size, Image.Resampling.LANCZOS)

        mean_diff, max_diff, diff_pct = compare_frames(official_img, our_img)
        all_mean.append(mean_diff)
        all_max.append(max_diff)

        # Status indicator
        if mean_diff < 10:
            status = "EXCELLENT"
        elif mean_diff < 25:
            status = "GOOD"
        elif mean_diff < 50:
            status = "MODERATE"
        else:
            status = "POOR"

        print(
            f"{frame_num:>6} | {mean_diff:>10.1f} | {max_diff:>9.1f} | {diff_pct:>7.1f}% | {status:>10}"
        )

    # Summary
    avg_mean = np.mean(all_mean)
    avg_max = np.mean(all_max)

    print(f"\n{'=' * 60}")
    print(f"SUMMARY")
    print(f"{'=' * 60}")
    print(f"Frames analyzed: {len(all_mean)}")
    print(f"Average mean difference: {avg_mean:.1f}/255 ({avg_mean / 255 * 100:.1f}%)")
    print(f"Average max difference: {avg_max:.1f}/255")

    if avg_mean < 10:
        print(f"\nResult: EXCELLENT - Renders are nearly identical")
    elif avg_mean < 25:
        print(f"\nResult: GOOD - Minor differences, visually acceptable")
    elif avg_mean < 50:
        print(f"\nResult: MODERATE - Notable differences, may need review")
    else:
        print(f"\nResult: POOR - Significant differences detected")

    return 0 if avg_mean < 50 else 1


if __name__ == "__main__":
    sys.exit(main())
