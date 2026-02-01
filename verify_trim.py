#!/usr/bin/env python3
"""Extract and compare frames to verify trim path animation is working."""

import subprocess
import os
from PIL import Image
import numpy as np


def extract_frame(video_path, frame_num, output_path):
    """Extract a specific frame from video."""
    timestamp = frame_num / 30.0  # 30fps
    cmd = [
        "ffmpeg",
        "-y",
        "-i",
        video_path,
        "-ss",
        str(timestamp),
        "-vframes",
        "1",
        output_path,
    ]
    subprocess.run(cmd, capture_output=True)


def compare_frames(frame1_path, frame2_path):
    """Compare two frames and return pixel difference percentage."""
    img1 = Image.open(frame1_path).convert("RGB")
    img2 = Image.open(frame2_path).convert("RGB")

    arr1 = np.array(img1)
    arr2 = np.array(img2)

    # Calculate difference
    diff = np.abs(arr1.astype(float) - arr2.astype(float))
    diff_percent = np.mean(diff) / 255.0 * 100

    return diff_percent


def main():
    video_path = "examples/tests/trim_path_output.mp4"

    if not os.path.exists(video_path):
        print(f"Video not found: {video_path}")
        return

    print("Extracting frames to verify trim path animation...")

    # Extract frames at key points
    frames = [0, 15, 30]  # Start, middle, end
    frame_paths = []

    for frame_num in frames:
        output_path = f"/tmp/trim_frame_{frame_num}.png"
        extract_frame(video_path, frame_num, output_path)
        frame_paths.append((frame_num, output_path))
        print(f"Extracted frame {frame_num}")

    # Compare consecutive frames
    print("\nComparing frames:")
    for i in range(len(frame_paths) - 1):
        frame1_num, frame1_path = frame_paths[i]
        frame2_num, frame2_path = frame_paths[i + 1]

        diff = compare_frames(frame1_path, frame2_path)
        print(f"Frame {frame1_num} -> {frame2_num}: {diff:.2f}% pixel difference")

        if diff > 5.0:
            print("  ✓ Significant change detected - animation is working!")
        else:
            print("  ✗ Minimal change - animation may be stuck")

    # Cleanup
    for _, path in frame_paths:
        if os.path.exists(path):
            os.remove(path)


if __name__ == "__main__":
    main()
