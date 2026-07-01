#!/bin/sh
set -eu

out_dir="${TMPDIR:-/tmp}/diffst-failure-smoke"
rm -rf "$out_dir"
mkdir -p "$out_dir"

err_file="$out_dir/non-path-input.err"

if typst compile --root . tests/failure/non-path-input.typ "$out_dir/non-path-input.pdf" 2>"$err_file"; then
  printf 'expected tests/failure/non-path-input.typ to fail, but it compiled\n' >&2
  exit 1
fi

if ! grep -E 'path|str|string' "$err_file" >/dev/null; then
  cat "$err_file" >&2
  exit 1
fi

printf 'verified failure fixture at tests/failure/non-path-input.typ\n'
