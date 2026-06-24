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
cargo build --release --target wasm32-unknown-unknown
typst compile --root . examples/basic.typ
typst compile --root . examples/realistic.typ
typst compile --root . examples/custom-colors.typ
typst compile --root . examples/manual-layout.typ
```

## Options

```typst
#diffst(
  "old.typ",
  "new.typ",
  ignore-whitespace: true,
  show-whitespace: true,
  display: "collapsed", // or "full"
)
```

`show-whitespace` makes changed spaces and tabs visible inside inline highlights.

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
  diffst-rows,
  diffst-summary,
  diffst-table,
)

#let report = diffst-report("old.typ", "new.typ", show-whitespace: true)
#let rows = diffst-rows(report, display: "collapsed", collapse-threshold: 8)

#grid(
  columns: (1fr, auto),
  [#diffst-summary(report)],
  [Reviewed by CK],
)

#v(8pt)
#diffst-table(report, rows: rows)
```

`diffst-layout(report, body: (report, rows, colors) => ..)` is available when
you want to keep the default row filtering but replace the final arrangement.
