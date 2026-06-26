# diffst implementation notes

This package has two halves:

- `src/lib.rs` is the Rust/WASM diff engine.
- `lib.typ` is the Typst API and renderer, with small internal modules in
  `typst/`.

The boundary between them is JSON. Typst reads the old and new files, sends
their bytes plus encoded options into the WASM plugin, receives a structured
report, and turns that report into Typst content.

## Build artifact

The internal report module loads the plugin from `plugin.wasm`:

```typst
#let _engine = plugin("../plugin.wasm")
```

During development, `scripts/smoke.sh` runs the Rust tests, builds
`target/wasm32-unknown-unknown/release/diffst_wasm.wasm`, copies it to
`plugin.wasm`, and compiles the examples. The package itself should only depend
on `plugin.wasm`; `target/...` is just the local Rust build output.

## Public Typst layers

The top-level `#diffst(..)` call is a small file convenience wrapper. It reads
the two paths, then calls `diffst-content(..)`, which creates the Elembic
element. The explicit wrappers exist so Typst/LSP can see the named parameters
while Elembic still gets the hidden data it needs for show rules.

The normal flow is:

1. `diffst(old-path, new-path, ..)` reads the files.
2. `diffst-content(old-text, new-text, ..)` creates the Elembic element.
3. Elembic calls `_display`.
4. `_display` calls `diffst-report(..)`.
5. `diffst-report(..)` in `typst/report.typ` sends the old/new text to
   `_engine.diff(..)`.
6. `_display` passes the report to `diffst-layout(..)`.
7. `diffst-layout(..)` computes visible rows and renders the summary plus a
   table.

The composable public helpers sit at different levels:

- `diffst(old-path, new-path, ..)` reads files and renders the default report.
- `diffst-content(old-text, new-text, ..)` renders the default report from
  already-read strings.
- `diffst-report(old-text, new-text, ..)` returns data only.
- `diffst-rows(report, ..)` applies range filtering and collapsed/full display.
- `diffst-table(report, ..)` renders the split-table view.
- `diffst-single-table(report, ..)` renders one Typst `table`.
- `diffst-summary(report, ..)` renders the heading/stats block.
- `diffst-layout(report, ..)` renders the default summary plus table, or calls
  `body(report, rows, colors)` for custom final layout.
- `diffst-hunks(report, ..)` groups raw operations into hunk dictionaries.

`diffst` is intentionally the only public layer that reads paths. The lower
layers accept strings and separate labels, so callers can use already-read
content without pretending it is a file path.

## Rust report shape

The Rust side serializes a `DiffReport`:

```text
{
  meta,
  stats,
  ops,
  rows,
}
```

`meta` records resolved options and diagnostics such as trailing-newline and
line-ending information. `stats` records line counts, additions, deletions,
changed blocks, equal lines, and the `similarity` ratio. `ops` records the raw
line-level operations from `similar`, including old/new ranges and the matching
row range. `rows` is the render-friendly representation used by Typst tables.

Each row has:

- `kind`: `"equal"`, `"delete"`, `"insert"`, or `"replace"`.
- old and new line numbers, when present.
- old and new text, when present.
- optional old/new inline spans.

Inline spans use a small vocabulary: `"equal"`, `"delete"`, `"insert"`, plus
`"equal-marker"`, `"delete-marker"`, and `"insert-marker"` for visible
whitespace markers.

## Rust diff pipeline

`diff(old, new, options)` is the WASM entrypoint. It returns
`Result<Vec<u8>, String>`, so `wasm-minimal-protocol` carries failures through
the plugin protocol while successful calls return report JSON.

`diff_impl` does the real work:

1. Decode old/new bytes as UTF-8.
2. Deserialize options with `serde` and `deny_unknown_fields`.
3. Split inputs into logical lines and record line-ending metadata.
4. Compute line operations with `similar`.
5. Use `ReportBuilder` to turn operations into stats, ops, and render rows.
6. Serialize the final report to JSON.

When `ignore_whitespace` is enabled, line matching uses normalized keys from
`normalize_key`, but rendered rows still contain the original text.

Replacement rows may also get inline spans. `inline: "words"` tokenizes by
Unicode word boundaries by default; `inline: "chars"` tokenizes by grapheme
clusters by default. Setting `unicode: false` switches to simpler Rust
character / ASCII-ish word grouping. When `semantic_cleanup` is true, inline
ops go through `similar`'s compact diff adapter.

Whitespace display is handled in Rust by changing span text and span kind:
spaces become `·`, tabs become `→`, LF becomes `↵`, and CR becomes `␍` in the
JSON. Typst then draws those marker spans as vector shapes over real raw
spaces, so PDF copy/paste keeps the underlying whitespace instead of copying
marker glyphs.

## Typst row processing

`diffst-rows(report, ..)` starts from `report.rows`.

If `range` is provided, `_range-rows` keeps rows whose old or new line number is
inside the inclusive 1-based range. `range-side: "old"` or `"new"` restricts
the match to one side.

After range filtering, `display` decides whether rows are returned as-is or
collapsed. Collapsed display replaces the middle of long equal runs with a
synthetic `"collapsed"` row. The synthetic row records how many unchanged rows
were hidden.

Hunks are computed separately from `report.ops`. `diffst-hunks` walks the op
stream, starts a hunk around changed ops, keeps configurable equal-line context,
and finishes a hunk when an equal gap is larger than the context window.

## Typst rendering

Rows are rendered into table cells with `_row-part`, `_code`, `_line-no`, and
the color helpers.

Code content uses inline `raw` text. Long unbroken content is wrapped in
`box(clip: true)` so it cannot draw outside the cell. This avoids inserting
break characters into the source text, though PDF viewers may still insert
newlines when copying visually wrapped text.

There are two table renderers:

- `diffst-table` / `_diff-table` renders four synchronized tables: old line
  numbers, old content, new line numbers, and new content.
- `diffst-single-table` / `_single-table` renders one Typst `table` with all
  four columns.

The split renderer exists because selecting a single content column in PDFs is
easier when each column is its own table. To keep the four tables visually
aligned, the renderer first creates hidden prototype rows, measures their
heights, then applies those heights to the visible column tables.

Collapsed rows and trailing-newline notes are rendered as full-width blocks
between table chunks. The split-table path chunks normal rows around collapsed
rows so those note blocks can span the whole visual table.

## Styling model

Colors are dictionaries. `default-colors` is merged with user overrides, and
`minimal-colors` is a print-friendly alternative.

Table structure is controlled by style dictionaries. `default-table-style` and
`minimal-table-style` define columns, rule style, and stroke widths. The
compatibility strings `"default"` and `"minimal"` are normalized internally.

`minimal-table` is a show rule that sets both `colors: minimal-colors` and
`table-style: minimal-table-style` for `diffst` elements in its scope.

Code cells intentionally rely on Typst raw text for the mono font. The package
sets sizes internally, but it does not hardcode a mono font family.

## Tests and smoke checks

Rust unit tests cover option validation, algorithms, inline highlighting,
whitespace markers, trailing-newline metadata, line endings, defaults, and row
operation ranges.

`scripts/smoke.sh` is the end-to-end check. It:

1. Runs `cargo test`.
2. Builds the release WASM target.
3. Copies the build output to `plugin.wasm`.
4. Compiles every `examples/**/*.typ` file to a temporary PDF directory.

This catches Rust regressions, WASM/plugin boundary problems, and Typst syntax
or rendering integration failures.
