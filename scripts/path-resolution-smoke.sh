#!/bin/sh
set -eu

out_dir="${TMPDIR:-/tmp}/diffst-path-resolution-smoke"
rm -rf "$out_dir"
mkdir -p "$out_dir"

typst compile --root . \
  tests/path-resolution/nested/path-input-success.typ \
  "$out_dir/path-input-success.pdf"

err_file="$out_dir/relative-string-input-fails.err"
if typst compile --root . \
  tests/path-resolution/nested/string-input-fails.typ \
  "$out_dir/relative-string-input-fails.pdf" 2>"$err_file"; then
  printf 'expected relative string input to fail, but it compiled\n' >&2
  exit 1
fi

if ! grep 'sources/caller-old.txt' "$err_file" >/dev/null; then
  cat "$err_file" >&2
  exit 1
fi

old_abs="$(pwd)/tests/path-resolution/nested/sources/caller-old.txt"
new_abs="$(pwd)/tests/path-resolution/nested/sources/caller-new.txt"
err_file="$out_dir/absolute-string-input-fails.err"
if typst compile --root . \
  --input "old=$old_abs" \
  --input "new=$new_abs" \
  tests/path-resolution/nested/absolute-string-input-fails.typ \
  "$out_dir/absolute-string-input-fails.pdf" 2>"$err_file"; then
  printf 'expected absolute string input to fail, but it compiled\n' >&2
  exit 1
fi

if ! grep "$old_abs" "$err_file" >/dev/null; then
  cat "$err_file" >&2
  exit 1
fi

printf 'verified nested path resolution fixtures in tests/path-resolution/nested\n'
