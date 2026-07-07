#!/bin/sh
set -eu

package_name="diffst"
package_version="0.1.0"
package_dir="package-pr/packages/preview/$package_name/$package_version"
out_dir="${TMPDIR:-/tmp}/diffst-local-package-smoke"
local_package_dir="${XDG_DATA_HOME:-$HOME/.local/share}/typst/packages/local/$package_name/$package_version"

if [ "$(uname)" = "Darwin" ]; then
  local_package_dir="$HOME/Library/Application Support/typst/packages/local/$package_name/$package_version"
fi

cleanup() {
  rm -rf "$local_package_dir"
  rm -rf "$out_dir"
}

trap cleanup EXIT

scripts/package-pr.py

typst-package-check check --offline "$package_dir"

rm -rf "$local_package_dir"
mkdir -p "$(dirname "$local_package_dir")"

package_symlink="$(find "$package_dir" -type l -print -quit)"
if [ -n "$package_symlink" ]; then
  printf 'package tree contains symlink: %s\n' "$package_symlink" >&2
  exit 1
fi

(
  cd "$package_dir"
  typship check
)

cp -R "$package_dir" "$local_package_dir"

installed_symlink="$(find "$local_package_dir" -type l -print -quit)"
if [ -n "$installed_symlink" ]; then
  printf 'local package install contains symlink: %s\n' "$installed_symlink" >&2
  exit 1
fi

if [ ! -f "$local_package_dir/lib.typ" ] || [ ! -f "$local_package_dir/plugin.wasm" ]; then
  printf 'local package install is missing lib.typ or plugin.wasm\n' >&2
  exit 1
fi

if [ "$(cd "$package_dir" && pwd -P)" = "$(cd "$local_package_dir" && pwd -P)" ]; then
  printf 'local package install resolves to the package staging directory\n' >&2
  exit 1
fi

rm -rf "$out_dir"
mkdir -p "$out_dir"

test_file="$out_dir/test.typ"
pdf_file="$out_dir/test.pdf"

printf 'alpha\nbeta\ngamma\n' > "$out_dir/old.txt"
printf 'alpha\nbetter\ngamma\n' > "$out_dir/new.txt"

cat > "$test_file" <<'TYP'
#import "@local/diffst:0.1.0": diffst, diffst-content, diffst-report, diffst-summary

#set page(height: auto)

#let old = "alpha\nbeta\ngamma\n"
#let new = "alpha\nbetter\ngamma\n"

#diffst(path("old.txt"), path("new.txt"))

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
