#!/bin/sh
set -eu

out_dir="${TMPDIR:-/tmp}/diffst-smoke"
rm -rf "$out_dir"
mkdir -p "$out_dir"

cargo test
cargo build --release --target wasm32-unknown-unknown
wasm_artifact="target/wasm32-unknown-unknown/release/diffst_wasm.wasm"
wasm-opt -Oz --enable-bulk-memory "$wasm_artifact" -o plugin.wasm

sh scripts/failure-smoke.sh
sh scripts/path-resolution-smoke.sh

find examples -type f -name '*.typ' | sort | while IFS= read -r example; do
  out_file="$out_dir/$(printf '%s' "$example" | tr '/.' '__').pdf"
  typst compile --root . "$example" "$out_file"
done

printf 'compiled example PDFs to %s\n' "$out_dir"
