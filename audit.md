# diffst audit

Fresh audit after the cleanup/refactor pass. This focuses on what still feels
risky, awkward, under-tested, or release-blocking in the current working tree.

## Fixed in the latest pass

- Item 5: `diffst-hunks` now carries the previous op through the loop instead
  of scanning all earlier ops each time a hunk starts. `examples/options/hunks.typ`
  adds compile-time assertions for hunk count and context.
- Item 7: `scripts/smoke.sh` now recreates the smoke output directory before
  compiling examples.
- Item 8: raw and summary stat helpers now share `_stat-info`, a single
  descriptor source for values and summary rendering metadata.
- Item 10: Rust report rows now borrow line strings during serialization instead
  of cloning every row's old/new text.

## Findings

### 1. Package still depends on a development-only WASM path

- `lib.typ:3` hardcodes `target/wasm32-unknown-unknown/release/diffst_wasm.wasm`.
- This works for local development and for `scripts/smoke.sh`, but it is not a
  packageable layout. A clean user checkout or Typst package install will not
  have `target/...` unless they build Rust first.
- Suggested fix: decide on the release layout now. For distribution, commit or
  generate the WASM artifact into a stable package path such as `plugin.wasm`,
  then load that from `lib.typ`.

### 2. Unknown WASM option keys are still silently ignored

- `src/lib.rs:75-89` deserializes into `RawOptions`, but the struct does not use
  `#[serde(deny_unknown_fields)]`.
- Wrong value types now fail, which is good, but misspelled keys still disappear:
  `{"semantic_cleaup": true}` would run with cleanup disabled and no warning.
- Suggested fix: add `#[serde(deny_unknown_fields)]` to `RawOptions` and add a
  regression test for a misspelled key.

### 3. Trailing-newline differences are metadata-only

- `src/lib.rs:390-402` records `old_trailing_newline` and
  `new_trailing_newline`, and `lib.typ:599-600` shows those fields in the debug
  panel.
- The main table still renders `a\n` versus `a` as a fully equal row with `100%`
  similarity. For users looking only at the report, the diff can claim equality
  while the files differ.
- Suggested fix: either render a small table row/message for trailing-newline
  differences, or explicitly document that the default table is line-content
  oriented and newline fidelity lives in `report.meta`.

### 4. Line-ending fidelity is still lossy

- `src/lib.rs:711-721` strips `\r` from every split line, and trailing-newline
  metadata only checks `text.ends_with('\n')`.
- That normalizes CRLF and LF, and it can also hide a literal carriage return at
  the end of a line. This may be acceptable for presentational document diffs,
  but it should be a conscious contract.
- Suggested fix: add metadata for line-ending style or document normalization
  behavior clearly. If patch-like fidelity matters, preserve raw line
  terminators separately from rendered line content.

### 5. Smoke tests compile examples but do not assert rendered content

- `scripts/smoke.sh:10-12` compiles every example to a PDF, which is a useful
  syntax/integration check.
- It will not catch semantic regressions like collapsed rows hiding too much,
  wrong hunk ranges, trailing-newline metadata disappearing, or visual overlap.
- Suggested fix: add data-level tests for `diffst-debug-raw`, `diffst-hunks-raw`,
  and `diffst-row-counts-raw`; optionally export a few stable examples to SVG or
  PNG for snapshot comparison.

### 6. Theme customization stops at colors

- `lib.typ:23-40` centralizes typography constants, which is cleaner, but font
  family and sizes are still private constants.
- Users can override colors through `default-colors`, but cannot tune mono font,
  code size, line number size, or cell padding without replacing bigger layout
  pieces.
- Suggested fix: introduce a small `default-theme` dictionary, or expose
  typography/padding fields on the Elembic element if styling flexibility is a
  package goal.

### 7. Error messages expose Rust/serde wording directly

- `Options::from_json` in `src/lib.rs:101-103` wraps serde errors as
  `invalid options JSON: ...`.
- That is much better than silent fallback, but type errors from serde can be
  wordy and not always phrased in package terms.
- Suggested fix: keep serde for validation, but add targeted tests for the most
  common user mistakes and decide whether the raw messages are acceptable.

### 8. `algorithm_name` has an unreachable fallback

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
