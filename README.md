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
- `long-lines.typ`
- `unicode.typ`
- `debug.typ`
- `semantic-cleanup.typ`
- `ignore-whitespace.typ`, `show-whitespace.typ`, `trailing-whitespace.typ`,
  and `trailing-newline.typ`
- `display-collapsed.typ` and `display-full.typ`
- `context-lines.typ`
- `collapse-threshold.typ`
- `table-layout.typ`
- `hunks.typ`

`examples/custom-colors.typ` shows color overrides,
`examples/minimal-table.typ` shows a print-friendly minimal table, and
`examples/show-rules.typ` shows Typst show rules for styling the rendered
blocks, tables, cells, and fonts around a report. `examples/partial-report.typ`
shows how to reuse one report and render selected line ranges.

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
  table-style: "default", // or "minimal", default-table-style, minimal-table-style, ...
  table-layout: "split", // or "single"
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

`show-whitespace` makes changed spaces and tabs visible inside inline
highlights, and marks trailing spaces or tabs on otherwise unchanged lines. The
marker drawings are vector overlays, so copying from the PDF keeps the
underlying whitespace instead of copying marker glyphs.

`context-lines` controls how many unchanged lines are kept before and after a
collapsed region. `collapse-threshold` controls how long an unchanged run must
be before it is collapsed.

`table-style` controls the table columns and rules. The default style draws a
light grid; `"minimal"` keeps only the center separator and the rule under the
header. For a document-wide minimal style, use `#show: minimal-table`.

`table-layout` controls the table structure. `"split"` is the default and uses
synchronized side-by-side tables so old/new content columns are easier to select
separately in PDFs. `"single"` renders the original one-node Typst `table`,
which can be useful for show rules or custom layouts that need to target a
single table.

`default-table-style` and `minimal-table-style` are exported dictionaries, so
table structure and stroke widths can be changed without rewriting the renderer.

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

`deadline-ms` is intentionally not exposed. `similar` can use real deadlines
when a clock is available, but Typst plugins do not currently provide the host
clock imports needed for a reliable WASM wall-clock cutoff.

The summary includes an `x% similar lines` score. It is based on exactly matched
lines, so a prose example with small edits on every line can show
`0% similar lines` even when the lines are visually similar. In manual layouts, use
`report.stats.similarity` for a `0.0` to `1.0` ratio and
`report.stats.equal_lines` for the matched-line count.

The rendered diff adds a marker row when only the final trailing newline
differs. Raw trailing-newline and line-ending details are exposed through
`report.meta`.

Long unbroken code spans are clipped to the table cell instead of being broken
with inserted characters. This keeps copied text clean while preventing a single
token from drawing outside the report.

The rendered diff uses separate synchronized tables for old line numbers, old
content, new line numbers, and new content. This makes it easier to select and
copy one side of the diff independently. PDF viewers may still insert newline
characters when copying visually wrapped lines; that is a PDF text-extraction
limitation rather than an inserted character in the rendered source.

Avoid broad `#show table: it => block(..)[#it]` wrappers around diff reports.
`diffst` measures internal table prototypes to synchronize row heights, so
wrapping every table adds that wrapper's padding to each measured row. Prefer
`#show table: set table(..)` and `#show table.cell: ...` rules for table styling.

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
`delete-text`, `insert-text`, `replace-text`, `marker`, and `collapsed`.

For printed papers or reports, `minimal-colors` and `minimal-table` provide a
more minimal style. The minimal table keeps only the middle separator and the
rule under the header, while keeping colored inline highlights for changed
words or characters.

```typst
#import "lib.typ": diffst, minimal-table

#show: minimal-table

#diffst("old.typ", "new.typ")
```

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
  diffst-single-table,
  diffst-table,
  minimal-table-style,
  minimal-colors,
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
      #similarity% similar lines\
      #linebreak()
      #diffst-summary-stat(report, "changed-blocks")
    ]
  ],
)

#v(8pt)
#diffst-debug(report, rows: rows)

#v(8pt)
#diffst-table(report, rows: rows)

#v(8pt)
#diffst-table(report, rows: rows, colors: minimal-colors, table-style: minimal-table-style)

#v(8pt)
#diffst-single-table(report, rows: rows)
```

`report.ops` exposes the raw line-level diff operations returned by the WASM
engine. Each op includes its kind, old/new line ranges, and corresponding row
range. `diffst-hunks(report, context-lines: 2)` groups those ops into hunk
dictionaries with `ops`, `rows`, `old_start`, `old_len`, `new_start`, and
`new_len` fields for custom layouts.

Use `range: (10, 18)` on `diffst-rows`, `diffst-table`,
`diffst-single-table`, or `diffst-layout` to render an inclusive 1-based line
range. By default it keeps rows whose old or new line number is in range; set
`range-side: "old"` or `range-side: "new"` to anchor the range to one file.
Range filtering happens before collapsed/full display rows are computed.

`diffst-layout(report, table-layout: "single", body: (report, rows, colors) => ..)`
is available when you want to keep the default row filtering but replace the
final arrangement.

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
  vertical spacing, and the diff table. Its `table-layout` option chooses the
  split-table or single-table renderer. Its `range` and `range-side` options
  can render only a slice of the report. If `body` is supplied, it calls
  `body(report, rows, colors)` instead.
- `diffst-summary(..)` returns a `block` containing a `grid`. It is composed
  from `diffst-summary-title(..)` and `diffst-summary-stats(..)`. The file
  labels and line counts are text, and the stats are small filled `box` pills
  created by `diffst-pill(..)`.
- `diffst-table(..)` returns synchronized side-by-side tables for the old line
  numbers, old content, new line numbers, and new content. They visually read as
  one diff table while keeping the content columns easier to select separately.
  The column sizes and rule widths come from `table-style`, which
  may be `default-table-style`, `minimal-table-style`, a dictionary derived from
  one of them, or the compatibility strings `"default"` and `"minimal"`.
- `diffst-single-table(..)` returns the original single Typst `table` version
  for custom layouts that need one table node instead of separately selectable
  synchronized tables. It accepts the same `rows`, `colors`, and `table-style`
  arguments as `diffst-table(..)`.
- `diffst-table(..)` and `diffst-single-table(..)` can compute their own rows
  with `display`, `collapse-threshold`, `context-lines`, `range`, and
  `range-side`, or render an explicit `rows` array from `diffst-rows(..)`.
- `minimal-table` is a show rule that sets `colors: minimal-colors` and
  `table-style: minimal-table-style` for all `diffst` elements in its scope.
- Code cells use Typst inline `raw` text, so the document's raw-text styling
  controls the monospace font. Inline highlights are `highlight` elements around
  that raw text.
- `diffst-rows(..)` returns row dictionaries for table helpers; it can apply
  collapsed/full display filtering and line-range filtering without emitting
  content.
- `diffst-hunks(..)` returns hunk dictionaries with `ops` and `rows`; it does
  not emit content.
- `diffst-debug(..)` returns a `block` containing a `grid`, a two-column
  `table`, and text debug messages.
- The `*-raw(..)` helpers return labels, numbers, counts, or numeric hunk
  summaries; they do not emit content.
- `diffst-report(..)` returns data from the WASM engine plus `old` and `new`
  labels; it does not emit content.
