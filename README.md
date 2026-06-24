# diffst

`diffst` is a Typst package for presentable side-by-side document diff reports.

```typst
#import "lib.typ": diffst

#diffst("old.typ", "new.typ")
```

The Typst package reads both files, passes their contents to a Rust WebAssembly
plugin, and renders the structured diff as a side-by-side report.

## Build

```sh
rustup target add wasm32-unknown-unknown
sh scripts/smoke.sh
```

The smoke script runs `cargo test`, builds the WASM plugin, and compiles every
`examples/**/*.typ` file to `${TMPDIR:-/tmp}/diffst-smoke`.

## Examples

The `examples/options/` directory contains focused examples where each file
turns on one option:

- `algorithm-myers.typ`, `algorithm-patience.typ`, `algorithm-histogram.typ`,
  `algorithm-lcs.typ`, and `algorithm-hunt.typ`
- `inline-chars.typ`, `inline-words.typ`, and `inline-none.typ`
- `unicode.typ`
- `debug.typ`
- `semantic-cleanup.typ`
- `ignore-whitespace.typ` and `show-whitespace.typ`
- `display-collapsed.typ` and `display-full.typ`
- `context-lines.typ`
- `collapse-threshold.typ`
- `hunks.typ`

`examples/custom-colors.typ` shows color overrides, and
`examples/show-rules.typ` shows Typst show rules for styling the rendered
blocks, tables, cells, and fonts around a report.

## Options

```typst
#diffst(
  "old.typ",
  "new.typ",
  algorithm: "patience",
  inline: "words",
  unicode: true,
  semantic-cleanup: true,
  ignore-whitespace: true,
  show-whitespace: true,
  display: "collapsed", // or "full"
  context-lines: 3,
)
```

`algorithm` can be `"myers"`, `"patience"`, `"lcs"`, `"hunt"`, or
`"histogram"`. Myers is the default. Patience and histogram can be more readable
for reordered prose or code, while LCS and Hunt are useful when you want to
compare the underlying algorithms.

`inline` controls how replaced lines are highlighted: `"chars"` highlights
character-level edits, `"words"` highlights word/punctuation chunks, and
`"none"` keeps only the changed-row background.

`unicode` controls inline tokenization quality and defaults to `true`. With
`inline: "chars"`, Unicode mode diffs grapheme clusters instead of raw Unicode
scalar values, so emoji sequences and combining marks stay together. With
`inline: "words"`, it uses Unicode word boundaries, which is better for
research papers and multilingual prose.

`semantic-cleanup` runs an extra cleanup pass on inline highlights to shift
highlight boundaries toward more readable chunks.

`show-whitespace` makes changed spaces and tabs visible inside inline highlights.

`context-lines` controls how many unchanged lines are kept before and after a
collapsed region. `collapse-threshold` controls how long an unchanged run must
be before it is collapsed.

`deadline-ms` is intentionally not exposed. `similar` can use real deadlines
when a clock is available, but Typst plugins do not currently provide the host
clock imports needed for a reliable WASM wall-clock cutoff.

The summary includes a line similarity score. It is based on exactly matched
lines, so a prose example with small edits on every line can show `0%` even
when the lines are visually similar. In manual layouts, use
`report.stats.similarity` for a `0.0` to `1.0` ratio and
`report.stats.equal_lines` for the matched-line count.

The default table compares rendered line content and adds a note when only the
final trailing newline differs. Raw newline and line-ending details are exposed
through `report.meta`.

## Colors

`diffst` is an Elembic element, so colors can be changed for one report or set
document-wide.

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

Available keys are `text`, `line-no`, `border`, `header`, `equal`, `delete`,
`insert`, `replace`, `inline-delete`, `inline-insert`, `inline-equal`,
`delete-text`, `insert-text`, `replace-text`, and `collapsed`.

## Manual Layouts

The default `#diffst(..)` call is built from smaller functions that can be
arranged manually.

```typst
#import "lib.typ": (
  diffst-report,
  diffst-debug,
  diffst-debug-raw,
  diffst-hunks,
  diffst-labels-raw,
  diffst-line-counts-raw,
  diffst-rows,
  diffst-stat-raw,
  diffst-summary,
  diffst-summary-stat,
  diffst-summary-title,
  diffst-table,
)

#let report = diffst-report("old.typ", "new.typ", show-whitespace: true)
#let hunks = diffst-hunks(report, context-lines: 2)
#let labels = diffst-labels-raw(report)
#let lines = diffst-line-counts-raw(report)
#let similarity = diffst-stat-raw(report, "similarity")
#let rows = diffst-rows(
  report,
  display: "collapsed",
  collapse-threshold: 8,
  context-lines: 2,
)

#grid(
  columns: (1fr, auto),
  [#diffst-summary(report, stats: ("similarity", "additions"))],
  [
    #align(right)[
      #labels.old -> #labels.new\
      #lines.old old lines, #lines.new new lines\
      #similarity% similar\
      #linebreak()
      #diffst-summary-stat(report, "changed-blocks")
    ]
  ],
)

#v(8pt)
#diffst-debug(report, rows: rows)

#v(8pt)
#diffst-table(report, rows: rows)
```

`report.ops` exposes the raw line-level diff operations returned by the WASM
engine. Each op includes its kind, old/new line ranges, and corresponding row
range. `diffst-hunks(report, context-lines: 2)` groups those ops into hunk
dictionaries with `ops`, `rows`, `old_start`, `old_len`, `new_start`, and
`new_len` fields for custom layouts.

`diffst-layout(report, body: (report, rows, colors) => ..)` is available when
you want to keep the default row filtering but replace the final arrangement.

`report.meta` exposes the resolved debug metadata from the WASM engine:
`algorithm`, `inline`, `unicode`, `ignore_whitespace`, `show_whitespace`,
`semantic_cleanup`, `old_trailing_newline`, `new_trailing_newline`,
`old_line_endings`, `new_line_endings`, and a `messages` array. Line ending
values are `"lf"`, `"crlf"`, `"cr"`, `"mixed"`, or `"none"`. Use
`diffst-debug(report, rows: ..)` to render those diagnostics in the document, or
`diffst-debug-raw(report, rows: ..)` to receive the same diagnostics as data for
a custom smoke panel.

For layouts that want numbers and labels instead of prebuilt content, diffst
also exposes raw helpers:

- `diffst-labels-raw(report)` returns `(old, new)`.
- `diffst-line-counts-raw(report)` returns `(old, new)` line counts.
- `diffst-stat-raw(report, "similarity")` returns one number. Supported raw
  stats are `"similarity"`, `"additions"`, `"deletions"`, `"changed-blocks"`,
  `"equal-lines"`, `"old-lines"`, and `"new-lines"`.
- `diffst-stats-raw(report, stats: (..))` returns an array of
  `(key, value)` dictionaries.
- `diffst-row-counts-raw(rows)` returns visible row counts by kind plus hidden
  collapsed rows.
- `diffst-hunk-raw(hunk)` returns numeric hunk ranges and context sizes.
- `diffst-hunks-raw(report, context-lines: ..)` returns raw hunk summaries.
- `diffst-debug-raw(report, rows: ..)` returns metadata, stats, row counts,
  op/hunk counts, and debug messages.

The summary is also split into smaller pieces:

- `diffst-summary-title(report, colors: ..)` combines the file label and line
  counts.
- `diffst-summary-label(report, colors: ..)` renders only the file labels.
- `diffst-summary-lines(report, colors: ..)` renders only the old/new line
  counts.
- `diffst-summary-stat(report, "similarity", colors: ..)` renders one stat
  pill. Supported stats are `"similarity"`, `"additions"`, `"deletions"`, and
  `"changed-blocks"`.
- `diffst-summary-stats(report, stats: (..), colors: ..)` returns an array of
  stat pills that can be spread into your own grid or stack.
- `diffst-pill(fill, fg, body)` is the small filled `box` primitive used by the
  default stats.
- `diffst-summary(report, title: .., stats: (..), body: ..)` keeps the default
  wrapper but lets you replace the title, choose stats, or provide a custom
  summary function with `body: (report, colors) => ..`.
- `diffst-debug(report, rows: ..)` renders the debug metadata and messages as
  a compact block plus table.

## Rendering Structure

The default renderer is ordinary Typst content, so custom layouts can wrap or
replace each layer.

- `#diffst(..)` is an Elembic element. Its display function calls
  `diffst-report(..)` and `diffst-layout(..)`.
- `diffst-layout(..)` returns a `block(width: 100%)` containing the summary,
  vertical spacing, and the diff table. If `body` is supplied, it calls
  `body(report, rows, colors)` instead.
- `diffst-summary(..)` returns a `block` containing a `grid`. It is composed
  from `diffst-summary-title(..)` and `diffst-summary-stats(..)`. The file
  labels and line counts are text, and the stats are small filled `box` pills
  created by `diffst-pill(..)`.
- `diffst-table(..)` returns a `table` with four columns: old line number, old
  content, new line number, and new content. Header, line number, content, and
  collapsed rows are `table.cell`s.
- Inline highlights inside code cells are `box` elements around monospace
  `text`. Unchanged text is plain monospace `text`.
- `diffst-rows(..)` returns row dictionaries for `diffst-table(..)`; it does
  not emit content.
- `diffst-hunks(..)` returns hunk dictionaries with `ops` and `rows`; it does
  not emit content.
- `diffst-debug(..)` returns a `block` containing a `grid`, a two-column
  `table`, and text debug messages.
- The `*-raw(..)` helpers return labels, numbers, counts, or numeric hunk
  summaries; they do not emit content.
- `diffst-report(..)` returns data from the WASM engine plus `old` and `new`
  labels; it does not emit content.
