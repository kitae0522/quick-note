#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
scripts/build-app.sh release
open "dist/Quick Note.app"
