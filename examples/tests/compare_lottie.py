#!/usr/bin/env python3
"""
Lottie Video Comparison Tool (Lightweight)
Compares rendered Lottie video against official reference to find discrepancies
"""

import subprocess
import os
import sys
from pathlib import Path


def extract_frames(video_path, output_dir, prefix):
    """Extract all frames from video as PNG"""
    os.makedirs(output_dir, exist_ok=True)

    # Extract at 30fps
    cmd = [
        "ffmpeg",
        "-y",
        "-i",
        video_path,
        "-vf",
        "fps=30",
        os.path.join(output_dir, f"{prefix}_%03d.png"),
    ]
    subprocess.run(cmd, capture_output=True)

    # Get list of extracted frames
    frames = sorted(
        [
            f
            for f in os.listdir(output_dir)
            if f.startswith(prefix) and f.endswith(".png")
        ]
    )
    return [os.path.join(output_dir, f) for f in frames]


def compare_frames(our_frame, official_frame, output_dir, frame_num):
    """Compare two frames and generate diff images"""
    try:
        import numpy as np
        from PIL import Image

        # Load images
        img1 = Image.open(our_frame).convert("RGB")
        img2 = Image.open(official_frame).convert("RGB")

        # Ensure same size
        if img1.size != img2.size:
            print(
                f"  Frame {frame_num}: Size mismatch - Our: {img1.size}, Official: {img2.size}"
            )
            return None

        # Convert to numpy arrays
        arr1 = np.array(img1).astype(np.float32)
        arr2 = np.array(img2).astype(np.float32)

        # Calculate absolute difference
        diff = np.abs(arr1 - arr2)
        diff_max = np.max(diff)
        diff_mean = np.mean(diff)

        # Create diff visualization (scaled up for visibility)
        diff_img = (diff / 255.0 * 255).astype(np.uint8)
        diff_pil = Image.fromarray(diff_img)
        diff_path = os.path.join(output_dir, f"diff_{frame_num:03d}.png")
        diff_pil.save(diff_path)

        return {
            "frame": frame_num,
            "max_diff": diff_max,
            "mean_diff": diff_mean,
            "diff_path": diff_path,
        }
    except Exception as e:
        print(f"  Error comparing frame {frame_num}: {e}")
        return None


def main():
    # Paths
    our_video = "examples/tests/lottie_heart_eyes_output.mp4"
    official_video = "examples/tests/heart_eyes_official.mp4"
    work_dir = "examples/tests/comparison"

    # Create directories
    our_frames_dir = os.path.join(work_dir, "our_frames")
    official_frames_dir = os.path.join(work_dir, "official_frames")
    diff_dir = os.path.join(work_dir, "diffs")
    os.makedirs(diff_dir, exist_ok=True)

    print("=" * 60)
    print("Lottie Video Comparison Tool")
    print("=" * 60)

    # Check videos exist
    if not os.path.exists(our_video):
        print(f"ERROR: Our video not found: {our_video}")
        return 1
    if not os.path.exists(official_video):
        print(f"ERROR: Official video not found: {official_video}")
        return 1

    # Extract frames
    print("\n[1/4] Extracting frames from our video...")
    our_frames = extract_frames(our_video, our_frames_dir, "our")
    print(f"      Extracted {len(our_frames)} frames")

    print("\n[2/4] Extracting frames from official video...")
    official_frames = extract_frames(official_video, official_frames_dir, "official")
    print(f"      Extracted {len(official_frames)} frames")

    # Compare frame counts
    if len(our_frames) != len(official_frames):
        print(f"\nWARNING: Frame count mismatch!")
        print(f"  Our: {len(our_frames)} frames")
        print(f"  Official: {len(official_frames)} frames")
        min_frames = min(len(our_frames), len(official_frames))
        print(f"  Comparing first {min_frames} frames only")
    else:
        min_frames = len(our_frames)
        print(f"\n[3/4] Comparing {min_frames} frames...")

    # Compare each frame
    results = []

    for i in range(min_frames):
        result = compare_frames(our_frames[i], official_frames[i], diff_dir, i + 1)
        if result:
            results.append(result)

            # Progress indicator every 10 frames
            if (i + 1) % 10 == 0 or i == min_frames - 1:
                print(
                    f"      Processed {i + 1}/{min_frames} frames (last max_diff: {result['max_diff']:.2f})"
                )

    # Generate report
    print("\n[4/4] Generating report...")

    if not results:
        print("ERROR: No comparison results generated")
        return 1

    # Calculate statistics
    import numpy as np

    mean_diffs = [r["mean_diff"] for r in results]
    max_diffs = [r["max_diff"] for r in results]

    print("\n" + "=" * 60)
    print("COMPARISON RESULTS")
    print("=" * 60)

    print(f"\nOverall Statistics:")
    print(f"  Frames compared: {len(results)}")
    print(f"  Mean pixel difference (avg): {np.mean(mean_diffs):.2f}/255")
    print(f"  Max pixel difference (avg): {np.mean(max_diffs):.2f}/255")
    print(f"  Worst max pixel difference: {np.max(max_diffs):.2f}/255")

    # Find frames with significant differences
    threshold = 30
    significant = [r for r in results if r["max_diff"] > threshold]

    print(f"\nWorst 10 frames (highest max difference):")
    worst = sorted(results, key=lambda x: x["max_diff"], reverse=True)[:10]
    for w in worst:
        print(
            f"  Frame {w['frame']:3d}: Max diff={w['max_diff']:6.2f}, Mean diff={w['mean_diff']:6.2f}"
        )

    if significant:
        print(
            f"\n⚠️  {len(significant)} frames have significant differences (max_diff > {threshold})"
        )
        print(f"   First bad frame: {significant[0]['diff_path']}")
    else:
        print(f"\n✅ All frames within acceptable tolerance (max_diff <= {threshold})")

    # Check for zero-difference frames (exact matches)
    exact_matches = [r for r in results if r["max_diff"] < 1.0]
    print(
        f"\n🎯 Exact matches (max_diff < 1.0): {len(exact_matches)}/{len(results)} frames"
    )

    # Save detailed report
    report_path = os.path.join(work_dir, "comparison_report.txt")
    with open(report_path, "w") as f:
        f.write("Lottie Video Comparison Report\n")
        f.write("=" * 60 + "\n\n")
        f.write(f"Our video: {our_video}\n")
        f.write(f"Official video: {official_video}\n\n")
        f.write(f"Frames compared: {len(results)}\n\n")
        f.write("Frame-by-frame results:\n")
        f.write("-" * 60 + "\n")
        for r in results:
            f.write(
                f"Frame {r['frame']:3d}: Max={r['max_diff']:6.2f}, Mean={r['mean_diff']:6.2f}\n"
            )

    print(f"\n📄 Full report saved to: {report_path}")
    print(f"🖼️  Diff images saved to: {diff_dir}")
    print(f"\n💡 Tip: Check the diff images in {diff_dir}")
    print(f"   White pixels = identical, colored pixels = differences")

    return 0


if __name__ == "__main__":
    sys.exit(main())
