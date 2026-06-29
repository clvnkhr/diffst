#!/bin/sh
set -eu

package_dir="package-pr/packages/preview/diffst/0.1.0"
out_dir="${TMPDIR:-/tmp}/diffst-local-package-smoke"
local_package_dir="${XDG_DATA_HOME:-$HOME/.local/share}/typst/packages/local/diffst/0.1.0"

if [ "$(uname)" = "Darwin" ]; then
  local_package_dir="$HOME/Library/Application Support/typst/packages/local/diffst/0.1.0"
fi

cleanup() {
  rm -rf "$local_package_dir"
  rm -rf "$out_dir"
}

trap cleanup EXIT

scripts/package-pr.py

typst-package-check check --offline "$package_dir"

rm -rf "$local_package_dir"

(
  cd "$package_dir"
  typship check
  typship install local
)

rm -rf "$out_dir"
mkdir -p "$out_dir"

test_file="$out_dir/test.typ"
pdf_file="$out_dir/test.pdf"

cat > "$test_file" <<'TYP'
#import "@local/diffst:0.1.0": diffst-content, diffst-report, diffst-summary

#set page(height: auto)

#let old = "alpha\nbeta\ngamma\n"
#let new = "alpha\nbetter\ngamma\n"

#diffst-content(old, new, old-label: "old.txt", new-label: "new.txt")

#let report = diffst-report(
  old,
  new,
  old-label: "old.txt",
  new-label: "new.txt",
)

#diffst-summary(report)
TYP

typst compile "$test_file" "$pdf_file"

printf 'compiled local package smoke PDF to %s\n' "$pdf_file"
