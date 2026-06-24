# Previous audit fix summary

The previous audit fix did a cleanup/refactor pass across the repo.

## Changes made

- Added `audit.md`.
- Refactored `src/lib.rs`:
  - strict typed option parsing instead of silent defaults
  - errors for malformed JSON and wrong option types
  - `ReportBuilder` to reduce duplicated row/op construction
  - safer semantic cleanup error handling instead of `unwrap`
  - trailing-newline metadata in `report.meta`
  - cheaper default line diff path
  - new Rust regression tests
- Cleaned up `lib.typ`:
  - centralized typography helpers
  - single-pass row counting
  - clearer hunk helper functions
  - debug panel now shows trailing-newline metadata
  - validation for negative `collapse-threshold`
- Updated `README.md`:
  - removed stale `examples/algorithms.typ`
  - documented the smoke script and trailing-newline metadata
- Added `scripts/smoke.sh`:
  - runs `cargo test`
  - builds WASM
  - compiles every Typst example

## Verification

Verified at the time with:

- `cargo fmt`
- `cargo test`
- `sh scripts/smoke.sh`
- `git diff --check`

