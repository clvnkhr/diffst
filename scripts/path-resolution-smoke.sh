#!/bin/sh
set -eu

out_dir="${TMPDIR:-/tmp}/diffst-path-resolution-smoke"
rm -rf "$out_dir"
mkdir -p "$out_dir"

typst compile --root . \
  tests/path-resolution/nested/path-input-success.typ \
  "$out_dir/path-input-success.pdf"

err_file="$out_dir/string-input-fails.err"
if typst compile --root . \
  tests/path-resolution/nested/string-input-fails.typ \
  "$out_dir/string-input-fails.pdf" 2>"$err_file"; then
  printf 'expected string-input-fails.typ to fail, but it compiled\n' >&2
  exit 1
fi

if ! grep 'sources/caller-old.txt' "$err_file" >/dev/null; then
  cat "$err_file" >&2
  exit 1
fi

printf 'verified nested path resolution fixtures in tests/path-resolution/nested\n'
