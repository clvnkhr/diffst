# diffst audit

Fresh audit after the cleanup/refactor pass. This focuses on what still feels
risky, awkward, under-tested, or release-blocking in the current working tree.

## Fixed in the latest pass

- Item 1: `lib.typ` now loads a package-local `plugin.wasm` instead of the
  development-only `target/wasm32-unknown-unknown/release/...` path.
  `scripts/smoke.sh` refreshes that artifact after the release WASM build.
- Item 5: `diffst-hunks` now carries the previous op through the loop instead
  of scanning all earlier ops each time a hunk starts. `examples/options/hunks.typ`
  adds compile-time assertions for hunk count and context.
- Item 7: `scripts/smoke.sh` now recreates the smoke output directory before
  compiling examples.
- Item 8: raw and summary stat helpers now share `_stat-info`, a single
  descriptor source for values and summary rendering metadata.
- Item 10: Rust report rows now borrow line strings during serialization instead
  of cloning every row's old/new text.
- Item 4: `report.meta` now includes `old_line_endings` and
  `new_line_endings` values (`lf`, `crlf`, `cr`, `mixed`, or `none`), and the
  debug panel renders them.
- Item 2: WASM options now use `#[serde(deny_unknown_fields)]`, with a
  regression test for misspelled option keys.
- Item 3: The default table now renders a visible note when trailing-newline
  presence differs.

## Findings

### 1. Smoke tests compile examples but do not assert rendered content

- `scripts/smoke.sh:10-12` compiles every example to a PDF, which is a useful
  syntax/integration check.
- It will not catch semantic regressions like collapsed rows hiding too much,
  wrong hunk ranges, trailing-newline metadata disappearing, or visual overlap.
- Suggested fix: add data-level tests for `diffst-debug-raw`, `diffst-hunks-raw`,
  and `diffst-row-counts-raw`; optionally export a few stable examples to SVG or
  PNG for snapshot comparison.

### 2. Theme customization stops at colors

- `lib.typ:23-40` centralizes typography constants, which is cleaner, but font
  family and sizes are still private constants.
- Users can override colors through `default-colors`, but cannot tune mono font,
  code size, line number size, or cell padding without replacing bigger layout
  pieces.
- Suggested fix: introduce a small `default-theme` dictionary, or expose
  typography/padding fields on the Elembic element if styling flexibility is a
  package goal.

### 3. Error messages expose Rust/serde wording directly

- `Options::from_json` in `src/lib.rs:101-103` wraps serde errors as
  `invalid options JSON: ...`.
- That is much better than silent fallback, but type errors from serde can be
  wordy and not always phrased in package terms.
- Suggested fix: keep serde for validation, but add targeted tests for the most
  common user mistakes and decide whether the raw messages are acceptable.

### 4. `algorithm_name` has an unreachable fallback

- `src/lib.rs:646-654` returns `"unknown"` for algorithms outside the known
  variants.
- Because options are parsed through `parse_algorithm`, this should not happen
  unless `similar::Algorithm` grows new variants and the compiler allows the
  wildcard to hide that change.
- Suggested fix: remove the wildcard if possible so new upstream variants force
  an explicit decision during compilation.

## Good parts worth keeping

- The Rust boundary is much healthier now: typed options, explicit JSON errors,
  no inline cleanup panic, and targeted regression tests.
- `ReportBuilder` makes the report/row/op contract far easier to inspect than
  the original one-function implementation.
- The raw Typst helpers are a good package design choice; `diffst` feels like a
  toolkit, not just a single prebuilt table.
- `scripts/smoke.sh` is a strong baseline because it exercises Rust, WASM, and
  every Typst example in one command.
