# diffst

`diffst` renders presentable side-by-side diff reports in Typst.

```typst
#import "lib.typ": diffst

#diffst("old.typ", "new.typ")
```

The package reads two text files, sends their contents to a Rust WebAssembly
plugin, and renders the structured diff as Typst content.

## Quick Start

```typst
#import "lib.typ": diffst

#diffst(
  "draft-old.typ",
  "draft-new.typ",
  inline: "words",
)
```

By default, `diffst` collapses long unchanged regions. Use `display: "full"` to
show every line.

```typst
#diffst(
  "old.typ",
  "new.typ",
  display: "full",
)
```

## Main Options

```typst
#diffst(
  "old.typ",
  "new.typ",
  algorithm: "histogram",
  inline: "words",
  unicode: true,
  semantic-cleanup: true,
  ignore-whitespace: false,
  show-whitespace: false,
  display: "collapsed",
  context-lines: 3,
  collapse-threshold: 14,
  table-layout: "split",
  table-style: "default",
)
```

- `algorithm`: `"histogram"`, `"myers"`, `"patience"`, `"lcs"`, or `"hunt"`.
  Histogram is the default because it tends to produce readable changed blocks
  for prose and reordered code. Myers is the classic baseline.
- `inline`: `"words"`, `"chars"`, or `"none"`. Word mode is the default because
  it reads better in reports; character mode is available for very fine-grained
  edits.
- `unicode`: when `true`, inline diffs use grapheme clusters and Unicode word
  boundaries.
- `semantic-cleanup`: uses `similar`'s compact inline diff adapter to make
  inline highlights less fragmented and easier to read. It is on by default for
  presentation; set it to `false` when you want more literal token-by-token
  inline spans.
- `ignore-whitespace`: compares lines while ignoring whitespace differences.
- `show-whitespace`: makes changed spaces and tabs visible, and marks trailing
  spaces or tabs on otherwise unchanged lines. Marker drawings are vector
  overlays, so PDF copy/paste keeps the underlying whitespace.
- `display`: `"full"` or `"collapsed"`.
- `context-lines`: unchanged lines to keep around collapsed regions.
- `collapse-threshold`: unchanged run length required before collapsing.
- `table-layout`: `"split"` or `"single"`. Split is the default and uses
  synchronized tables so old/new content columns are easier to copy separately.
  Single renders one Typst `table`.
- `table-style`: `"default"`, `"minimal"`, `default-table-style`,
  `minimal-table-style`, or a derived style dictionary.

## Styling

Override colors per report or document-wide through Elembic:

```typst
#import "@preview/elembic:1.1.1" as e
#import "lib.typ": diffst, default-colors

#show: e.set_(diffst, colors: default-colors + (
  replace: rgb("#e0f2fe"),
  inline-delete: rgb("#f0abfc"),
  inline-insert: rgb("#67e8f9"),
))

#diffst("old.typ", "new.typ")
```

Available color keys:

`text`, `line-no`, `border`, `header`, `equal`, `delete`, `insert`, `replace`,
`inline-delete`, `inline-insert`, `inline-equal`, `delete-text`, `insert-text`,
`replace-text`, `marker`, and `collapsed`.

For printed papers or compact reports, use the minimal style:

```typst
#import "lib.typ": diffst, minimal-table

#show: minimal-table

#diffst("old.typ", "new.typ")
```

`minimal-table` sets `minimal-colors` and `minimal-table-style`. The minimal
table keeps only the middle separator and the rule under the header while
retaining colored inline highlights.

Table style dictionaries control columns and stroke widths:

```typst
#import "lib.typ": diffst, default-table-style

#diffst(
  "old.typ",
  "new.typ",
  table-style: default-table-style + (
    columns: (2em, 1fr, 2em, 1fr),
    stroke-width: (header: 0.7pt, body: 0.35pt),
  ),
)
```

Prefer `#show table: set table(..)` and `#show table.cell: ...` for table
styling. Broad wrappers such as `#show table: it => block(..)[#it]` can affect
internal measurement tables used to synchronize row heights.

## Composition API

For custom reports, build from data upward:

```typst
#import "lib.typ": (
  diffst-report,
  diffst-summary,
  diffst-table,
  diffst-single-table,
)

#let report = diffst-report(
  "paper-old.typ",
  "paper-new.typ",
  inline: "words",
  semantic-cleanup: true,
)

#diffst-summary(report)

#v(8pt)

#diffst-table(report, range: (10, 18))

#v(8pt)

#diffst-single-table(report, range: (26, 34), range-side: "new")
```

The main composition layers are:

- `diffst-report(old, new, ..)` returns structured diff data and metadata.
- `diffst-summary(report, ..)` renders the file labels, line counts, and stat
  pills.
- `diffst-table(report, ..)` renders the default split-table diff.
- `diffst-single-table(report, ..)` renders the one-table version.
- `diffst-layout(report, ..)` renders the default full report: summary, spacing,
  and table.

`range: (start, end)` filters to an inclusive 1-based line range before
`display` is applied. By default, rows are kept when either the old or new line
number is in range. Use `range-side: "old"` or `range-side: "new"` to anchor the
range to one file.

Use `diffst-layout(..., body: (report, rows, colors) => ..)` when you want the
default row filtering, range handling, and color resolution, but a custom final
arrangement:

```typst
#diffst-layout(
  report,
  range: (10, 18),
  body: (report, rows, colors) => [
    #diffst-summary(report, colors: colors)
    #v(4pt)
    #diffst-single-table(report, rows: rows, colors: colors)
  ],
)
```

## Lower-Level Data Helpers

Use these when you want to compute your own layout:

- `diffst-rows(report, display: .., range: ..)` returns renderable row
  dictionaries without emitting content.
- `diffst-hunks(report, context-lines: ..)` returns hunk dictionaries with
  `ops`, `rows`, `old_start`, `old_len`, `new_start`, and `new_len`.
- `diffst-debug(report, rows: ..)` renders a compact debug panel.
- `diffst-debug-raw(report, rows: ..)` returns debug data.
- `diffst-labels-raw(report)` returns `(old, new)`.
- `diffst-line-counts-raw(report)` returns old/new line counts.
- `diffst-stat-raw(report, "similarity")` returns one numeric stat.
- `diffst-stats-raw(report, stats: (..))` returns `(key, value)` dictionaries.
- `diffst-row-counts-raw(rows)` returns visible row counts by kind.
- `diffst-hunk-raw(hunk)` returns numeric hunk ranges and context sizes.
- `diffst-hunks-raw(report, context-lines: ..)` returns raw hunk summaries.

Supported stat keys are `"similarity"`, `"additions"`, `"deletions"`,
`"changed-blocks"`, `"equal-lines"`, `"old-lines"`, and `"new-lines"`.

## Copy/Paste Notes

The default split layout uses separate synchronized tables for old line numbers,
old content, new line numbers, and new content. This makes it easier to select
one side of the diff in PDF viewers.

Long unbroken code spans are clipped to the table cell instead of being broken
with inserted characters. Visually wrapped lines may still copy with inserted
newlines depending on the PDF viewer; that is a PDF text-extraction behavior,
not an inserted character in the Typst source.

## Metadata

`report.meta` includes:

`algorithm`, `inline`, `unicode`, `ignore_whitespace`, `show_whitespace`,
`semantic_cleanup`, `old_trailing_newline`, `new_trailing_newline`,
`old_line_endings`, `new_line_endings`, and `messages`.

Line ending values are `"lf"`, `"crlf"`, `"cr"`, `"mixed"`, or `"none"`.
When only the final trailing newline differs, the rendered diff adds a note row;
raw newline and line-ending details remain available through `report.meta`.

The summary's `x% similar lines` score is based on exactly matched lines. A
prose diff with small edits on every line can therefore show `0% similar lines`
even when the lines look visually similar.

## Examples

Focused option examples live in `examples/options/`:

- algorithms: `algorithm-myers.typ`, `algorithm-patience.typ`,
  `algorithm-histogram.typ`, `algorithm-lcs.typ`, `algorithm-hunt.typ`
- inline modes: `inline-chars.typ`, `inline-words.typ`, `inline-none.typ`
- display: `display-full.typ`, `display-collapsed.typ`,
  `collapse-threshold.typ`, `context-lines.typ`
- whitespace: `ignore-whitespace.typ`, `show-whitespace.typ`,
  `trailing-whitespace.typ`, `trailing-newline.typ`
- other: `unicode.typ`, `semantic-cleanup.typ`, `long-lines.typ`,
  `table-layout.typ`, `debug.typ`, `hunks.typ`

Larger examples:

- `examples/basic.typ`
- `examples/realistic.typ`
- `examples/custom-colors.typ`
- `examples/minimal-table.typ`
- `examples/show-rules.typ`
- `examples/manual-layout.typ`
- `examples/partial-report.typ`

## Build

```sh
rustup target add wasm32-unknown-unknown
sh scripts/smoke.sh
```

The smoke script runs `cargo test`, builds the WASM plugin, and compiles every
example to `${TMPDIR:-/tmp}/diffst-smoke`.

The package loads `plugin.wasm` from the repository root. `scripts/smoke.sh`
refreshes that package-local artifact from the release build before compiling
examples.

`deadline-ms` is intentionally not exposed. The `similar` crate can use real
deadlines when a clock is available, but Typst plugins do not currently provide
the host clock imports needed for a reliable WASM wall-clock cutoff.
