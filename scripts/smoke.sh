#!/bin/sh
set -eu

out_dir="${TMPDIR:-/tmp}/diffst-smoke"
rm -rf "$out_dir"
mkdir -p "$out_dir"

cargo test
cargo build --release --target wasm32-unknown-unknown

find examples -type f -name '*.typ' | sort | while IFS= read -r example; do
  out_file="$out_dir/$(printf '%s' "$example" | tr '/.' '__').pdf"
  typst compile --root . "$example" "$out_file"
done

printf 'compiled example PDFs to %s\n' "$out_dir"
