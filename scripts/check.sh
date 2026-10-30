#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
python3 scripts/check-source-size.py
python3 scripts/check-native-fixture.py
if [[ "$(uname -s)" == "Darwin" ]]; then
    python3 scripts/check-resource-usage.py
fi
cargo fmt --package quick-note -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
