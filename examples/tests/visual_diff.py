#!/usr/bin/env python3
"""
Visual frame diff tool using PIL - generates side-by-side and diff overlay
"""

import sys
from PIL import Image, ImageDraw, ImageFont
import numpy as np
from pathlib import Path


def visualize_diff(official_img, our_img, frame_num, output_dir):
    """Create visual comparison: official | our | diff heatmap | overlay"""

    # Convert to numpy arrays
    official = np.array(official_img).astype(np.float32)
    ours = np.array(our_img).astype(np.float32)

    # Calculate pixel-wise difference (per channel)
    diff = np.abs(official - ours)
    diff_gray = np.mean(diff, axis=2)  # Average across RGB

    # Create heatmap (convert to PIL)
    heatmap_array = np.zeros_like(official)
    max_diff = 255.0

    # Normalize difference to 0-1
    diff_norm = np.clip(diff_gray / max_diff, 0, 1)

    # Heatmap: Red = high difference, Green = low difference
    heatmap_array[:, :, 0] = (diff_norm * 255).astype(np.uint8)  # Red
    heatmap_array[:, :, 1] = ((1 - diff_norm) * 255).astype(np.uint8)  # Green
    heatmap_array[:, :, 2] = 50  # Slight blue tint

    heatmap = Image.fromarray(heatmap_array.astype(np.uint8))

    # Create overlay
    overlay = Image.blend(our_img, heatmap, alpha=0.4)

    # Resize all to fit side-by-side
    target_width = 400
    scale = target_width / official_img.width
    target_height = int(official_img.height * scale)

    official_small = official_img.resize(
        (target_width, target_height), Image.Resampling.LANCZOS
    )
    ours_small = our_img.resize((target_width, target_height), Image.Resampling.LANCZOS)
    heatmap_small = heatmap.resize(
        (target_width, target_height), Image.Resampling.LANCZOS
    )
    overlay_small = overlay.resize(
        (target_width, target_height), Image.Resampling.LANCZOS
    )

    # Create composite image
    total_width = target_width * 4
    total_height = target_height + 40  # Extra space for labels

    composite = Image.new("RGB", (total_width, total_height), (40, 40, 40))

    # Paste images
    composite.paste(official_small, (0, 40))
    composite.paste(ours_small, (target_width, 40))
    composite.paste(heatmap_small, (target_width * 2, 40))
    composite.paste(overlay_small, (target_width * 3, 40))

    # Add labels
    draw = ImageDraw.Draw(composite)
    try:
        font = ImageFont.truetype("arial.ttf", 20)
    except:
        font = ImageFont.load_default()

    labels = ["Official", "Our Render", "Diff Map", "Overlay"]
    colors = [(255, 255, 255), (255, 255, 255), (255, 255, 255), (255, 255, 255)]

    for i, (label, color) in enumerate(zip(labels, colors)):
        x = i * target_width + 10
        draw.text((x, 10), label, fill=color, font=font)

    # Add frame number
    draw.text(
        (10, total_height - 25), f"Frame {frame_num}", fill=(200, 200, 200), font=font
    )

    # Save
    output_path = output_dir / f"diff_frame_{frame_num:03d}.png"
    composite.save(output_path)

    # Stats
    mean_diff = np.mean(diff_gray)
    max_diff_val = np.max(diff_gray)
    diff_pixels = np.sum(diff_gray > 20)  # Pixels with >20 difference
    total_pixels = diff_gray.size
    diff_percent = (diff_pixels / total_pixels) * 100

    return mean_diff, max_diff_val, diff_percent


def extract_frames(video_path, frame_nums):
    """Extract specific frames from video using ffmpeg"""
    import subprocess
    import tempfile
    import os

    frames = {}

    for frame_num in frame_nums:
        # Use ffmpeg to extract specific frame
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
                frames[frame_num] = img.copy()  # Copy to memory
                img.close()  # Close file handle
            else:
                print(f"  Warning: Could not extract frame {frame_num}")
        except Exception as e:
            print(f"  Error extracting frame {frame_num}: {e}")
        finally:
            if Path(tmp_path).exists():
                try:
                    os.unlink(tmp_path)
                except:
                    pass  # Ignore Windows file lock issues

    return frames


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <official.mp4> <ours.mp4>")
        print(f"\nGenerates visual diff images showing where renders differ")
        sys.exit(1)

    official_path = sys.argv[1]
    ours_path = sys.argv[2]

    # Key frames to analyze
    key_frames = [0, 15, 30, 45, 60, 75, 90]

    print(f"Extracting key frames from both videos...")
    print(f"  Official: {official_path}")
    official_frames = extract_frames(official_path, key_frames)

    print(f"  Ours: {ours_path}")
    ours_frames = extract_frames(ours_path, key_frames)

    output_dir = Path("examples/tests/diff_visuals")
    output_dir.mkdir(exist_ok=True)

    print(f"\nAnalyzing frames...")
    print(f"{'Frame':>6} | {'Mean Diff':>10} | {'Max Diff':>9} | {'% Diff':>7}")
    print(f"{'-' * 45}")

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

        mean_diff, max_diff, diff_pct = visualize_diff(
            official_img, our_img, frame_num, output_dir
        )
        all_mean.append(mean_diff)
        all_max.append(max_diff)

        print(
            f"{frame_num:>6} | {mean_diff:>10.1f} | {max_diff:>9.1f} | {diff_pct:>7.1f}%"
        )

    # Summary
    print(f"\n{'=' * 50}")
    print(f"SUMMARY")
    print(f"{'=' * 50}")
    print(f"Frames analyzed: {len(all_mean)}")
    print(
        f"Average mean difference: {np.mean(all_mean):.1f}/255 ({np.mean(all_mean) / 255 * 100:.1f}%)"
    )
    print(f"Average max difference: {np.mean(all_max):.1f}/255")
    print(f"\nVisual diffs saved to: {output_dir.absolute()}")

    if np.mean(all_mean) > 50:
        print(f"\n⚠️  SIGNIFICANT RENDERING DIFFERENCES DETECTED")
        print(f"View the images to identify issues:")
        print(f"  - RED areas = major differences")
        print(f"  - GREEN areas = matching content")
        print(f"\nLikely causes of >50% difference:")
        print(f"  1. Different Lottie renderer (lottie-web vs Skia)")
        print(f"  2. Anti-aliasing/filtering differences")
        print(f"  3. Scaling/resampling differences")
        print(f"  4. Color space handling differences")
        print(f"  5. Missing/unsupported Lottie features")
    else:
        print(f"\n✓ Renders are visually similar")

    return 0


if __name__ == "__main__":
    sys.exit(main())
