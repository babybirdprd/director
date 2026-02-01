#!/bin/bash

# Export Lottie to MP4 and capture debug output
cargo run --release -p director-cli --bin director-engine -- \
  examples/tests/lottie_heart_eyes_export.rhai \
  examples/tests/lottie_trim_test.mp4 2>&1 | grep -E "(LOTTIE-TRIM|Rendering|Frame)" | head -100