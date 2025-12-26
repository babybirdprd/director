#!/bin/bash
set -e

# Build director-plan
echo "Building director-plan..."
cargo build -p director-plan

REPO_ROOT=$(pwd)
DIRECTOR_BIN=$(pwd)/target/debug/director-plan
WORK_DIR=$(mktemp -d)
echo "Working in $WORK_DIR"

cd $WORK_DIR
echo "Cloning rust-skia-engine..."
git clone https://github.com/babybirdprd/rust-skia-engine skia_engine
cd skia_engine

# Setup director-plan structure
mkdir -p plan/tickets assets

# Create Ticket
echo "Creating Ticket..."
cat > plan/tickets/T-FAIL.toml <<EOF
[meta]
id = "T-FAIL"
title = "Visual Regression Failure Test"
status = "in_progress"
priority = "high"
owner = "Jules"
created_at = 2024-05-22T00:00:00Z

[spec]
description = "Test to verify visual regression failure artifacts are captured."
constraints = []
relevant_files = ["crates/director-core/tests/visual_regression.rs"]

[verification]
command = "sh -c 'mkdir -p target/artifacts/T-FAIL && cargo test -p director-core --no-default-features --features mock_video --test visual_regression test_visual_basic_box; cp crates/director-core/target/visual_regression_failures/test_visual_basic_box_actual.png target/artifacts/T-FAIL/actual.png; cp crates/director-core/target/visual_regression_failures/test_visual_basic_box_diff.png target/artifacts/T-FAIL/diff.png'"
golden_image = "crates/director-core/tests/snapshots/test_visual_basic_box.png"

[history]
log = []
EOF

# Modify test to fail (Red box)
echo "Modifying test to fail..."
sed -i 's/Color::new(0.0, 0.0, 1.0, 1.0)/Color::new(1.0, 0.0, 0.0, 1.0)/' crates/director-core/tests/visual_regression.rs

# Start server
echo "Starting director-plan server..."
$DIRECTOR_BIN serve > server.log 2>&1 &
SERVER_PID=$!

cleanup() {
  echo "Cleaning up..."
  kill $SERVER_PID || true
  rm -rf $WORK_DIR
}
trap cleanup EXIT

# Wait for server
sleep 5

# Trigger verification
echo "Triggering verification..."
curl -X POST http://localhost:3000/api/tickets/T-FAIL/verify

# Check artifacts
if [ -f "target/public/artifacts/T-FAIL/diff.png" ]; then
  echo "SUCCESS: Diff image generated at target/public/artifacts/T-FAIL/diff.png"
  mkdir -p $REPO_ROOT/proof
  cp target/public/artifacts/T-FAIL/*.png $REPO_ROOT/proof/
else
  echo "FAILURE: Diff image not found."
  cat server.log
  exit 1
fi
