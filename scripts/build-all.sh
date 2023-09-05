#!/usr/bin/env bash
set -eu

cargo build --release --target wasm32-wasi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
cd $SCRIPT_DIR
cd ../packages/fusion
npm run prepack
