#import "@preview/elembic:1.1.1" as e

#let _engine = plugin("target/wasm32-unknown-unknown/release/diffst_wasm.wasm")

#let default-colors = (
  text: rgb("#111827"),
  line-no: rgb("#6b7280"),
  border: rgb("#cfd7e3"),
  header: rgb("#e8edf5"),
  equal: white,
  delete: rgb("#ffe3e6"),
  insert: rgb("#dcf7e4"),
  replace: rgb("#fff1bf"),
  inline-delete: rgb("#ff9aa5"),
  inline-insert: rgb("#83dea1"),
  inline-equal: none,
  delete-text: rgb("#7f1d1d"),
  insert-text: rgb("#14532d"),
  replace-text: rgb("#713f12"),
  collapsed: rgb("#f7f8fb"),
)

#let _color(colors, key) = colors.at(key, default: default-colors.at(key))

#let _cell(colors, fill, body, align: left) = table.cell(
  fill: fill,
  inset: (x: 4.5pt, y: 3pt),
  align: align,
)[#body]

#let _line-no(colors, value) = {
  if value == none {
    text(fill: _color(colors, "line-no"), size: 6.5pt)[]
  } else {
    text(fill: _color(colors, "line-no"), size: 6.5pt, font: "DejaVu Sans Mono")[#str(value)]
  }
}

#let _span-fill(colors, kind) = {
  if kind == "delete" {
    _color(colors, "inline-delete")
  } else if kind == "insert" {
    _color(colors, "inline-insert")
  } else {
    _color(colors, "inline-equal")
  }
}

#let _span-text-fill(colors, kind) = {
  if kind == "delete" {
    _color(colors, "delete-text")
  } else if kind == "insert" {
    _color(colors, "insert-text")
  } else {
    _color(colors, "text")
  }
}

#let _code-span(colors, span) = {
  let body = text(
    size: 6.8pt,
    font: "DejaVu Sans Mono",
    fill: _span-text-fill(colors, span.kind),
  )[#span.text]

  if span.kind == "equal" {
    body
  } else {
    box(
      fill: _span-fill(colors, span.kind),
      inset: (x: 0.7pt, y: 0.4pt),
      outset: (y: 0.2pt),
      radius: 1pt,
    )[#body]
  }
}

#let _code(colors, value, spans: none) = {
  if spans != none {
    for span in spans {
      _code-span(colors, span)
    }
  } else if value == none {
    text(size: 6.8pt, font: "DejaVu Sans Mono")[]
  } else {
    text(size: 6.8pt, font: "DejaVu Sans Mono", fill: _color(colors, "text"))[#value]
  }
}

#let _row-fill(colors, kind) = {
  if kind == "delete" {
    _color(colors, "delete")
  } else if kind == "insert" {
    _color(colors, "insert")
  } else if kind == "replace" {
    _color(colors, "replace")
  } else {
    _color(colors, "equal")
  }
}

#let _pill(fill, fg, body) = box(
  fill: fill,
  inset: (x: 5pt, y: 2pt),
  radius: 2pt,
)[#text(size: 7pt, fill: fg, weight: "bold")[#body]]

#let _flush_equal_run(run, threshold, context-lines) = {
  let keep = calc.max(0, context-lines)
  if run.len() > threshold and run.len() > keep * 2 {
    let keep = calc.min(keep, run.len())
    let hidden = run.len() - keep * 2
    run.slice(0, keep) + ((
      kind: "collapsed",
      hidden: hidden,
    ),) + run.slice(run.len() - keep)
  } else {
    run
  }
}

#let _with-collapse(rows, threshold, context-lines) = {
  let output = ()
  let equal-run = ()

  for row in rows {
    if row.kind == "equal" {
      equal-run.push(row)
    } else {
      output += _flush_equal_run(equal-run, threshold, context-lines)
      equal-run = ()
      output.push(row)
    }
  }

  output + _flush_equal_run(equal-run, threshold, context-lines)
}

#let _summary(colors, report) = block[
  #grid(
    columns: (1fr, auto, auto, auto, auto),
    gutter: 6pt,
    align: horizon,
    [
      #text(size: 8.5pt, weight: "bold")[#report.old]
      #h(3pt)
      #text(fill: _color(colors, "line-no"))[#math.mapsto]
      #h(3pt)
      #text(size: 8.5pt, weight: "bold")[#report.new]
      #linebreak()
      #text(size: 6.8pt, fill: _color(colors, "line-no"))[
        #report.stats.old_lines old lines, #report.stats.new_lines new lines
      ]
    ],
    _pill(_color(colors, "header"), _color(colors, "text"), str(calc.round(report.stats.similarity * 100)) + "% similar"),
    _pill(_color(colors, "insert"), _color(colors, "insert-text"), "+" + str(report.stats.additions)),
    _pill(_color(colors, "delete"), _color(colors, "delete-text"), "-" + str(report.stats.deletions)),
    _pill(_color(colors, "replace"), _color(colors, "replace-text"), str(report.stats.changed_blocks) + " changed blocks"),
  )
]

#let _diff-table(colors, rows) = table(
  columns: (2.4em, 1fr, 2.4em, 1fr),
  stroke: (x, y) => (
    paint: _color(colors, "border"),
    thickness: if y == 0 { 0.8pt } else { 0.45pt },
  ),
  inset: 0pt,
  _cell(colors, _color(colors, "header"), text(size: 6.5pt, weight: "bold")[Old], align: center),
  _cell(colors, _color(colors, "header"), text(size: 6.5pt, weight: "bold")[Content]),
  _cell(colors, _color(colors, "header"), text(size: 6.5pt, weight: "bold")[New], align: center),
  _cell(colors, _color(colors, "header"), text(size: 6.5pt, weight: "bold")[Content]),
  ..rows.map(row => {
    if row.kind == "collapsed" {
      (
        table.cell(colspan: 4, fill: _color(colors, "collapsed"), inset: (x: 4pt, y: 3pt), align: center)[
          #text(size: 6.5pt, fill: _color(colors, "line-no"))[#row.hidden unchanged lines hidden]
        ],
      )
    } else {
      let fill = _row-fill(colors, row.kind)
      (
        _cell(colors, fill, _line-no(colors, row.at("old_no", default: none)), align: right),
        _cell(colors, fill, _code(
          colors,
          row.at("old", default: none),
          spans: row.at("old_spans", default: none),
        )),
        _cell(colors, fill, _line-no(colors, row.at("new_no", default: none)), align: right),
        _cell(colors, fill, _code(
          colors,
          row.at("new", default: none),
          spans: row.at("new_spans", default: none),
        )),
      )
    }
  }).flatten(),
)

#let diffst-report(
  old,
  new,
  ignore-whitespace: false,
  show-whitespace: false,
  algorithm: "myers",
  inline: "chars",
  semantic-cleanup: false,
) = {
  let old-content = read(old)
  let new-content = read(new)
  let options = json.encode((
    ignore_whitespace: ignore-whitespace,
    show_whitespace: show-whitespace,
    algorithm: algorithm,
    inline: inline,
    semantic_cleanup: semantic-cleanup,
  ))
  let report = json(_engine.diff(bytes(old-content), bytes(new-content), bytes(options)))

  if "error" in report {
    panic(report.error)
  }

  report + (
    old: old,
    new: new,
  )
}

#let diffst-rows(
  report,
  display: "collapsed",
  collapse-threshold: 14,
  context-lines: 3,
) = {
  if context-lines < 0 {
    panic("context-lines must be greater than or equal to 0")
  }

  if display == "full" {
    report.rows
  } else if display == "collapsed" {
    _with-collapse(report.rows, collapse-threshold, context-lines)
  } else {
    panic("display must be \"full\" or \"collapsed\"")
  }
}

#let _previous_op(ops, op) = {
  let previous = none
  for candidate in ops {
    if candidate.row_start < op.row_start {
      previous = candidate
    }
  }
  previous
}

#let _finish_hunk(report, hunk) = {
  let rows = report.rows.slice(hunk.row_start, hunk.row_end)
  let old-nos = rows
    .filter(row => row.at("old_no", default: none) != none)
    .map(row => row.old_no)
  let new-nos = rows
    .filter(row => row.at("new_no", default: none) != none)
    .map(row => row.new_no)

  (
    ops: hunk.ops,
    rows: rows,
    row_start: hunk.row_start,
    row_end: hunk.row_end,
    context_before: hunk.context_before,
    context_after: hunk.context_after,
    old_start: if old-nos.len() == 0 { none } else { old-nos.first() },
    old_len: old-nos.len(),
    new_start: if new-nos.len() == 0 { none } else { new-nos.first() },
    new_len: new-nos.len(),
  )
}

#let diffst-hunks(report, context-lines: 3) = {
  if context-lines < 0 {
    panic("context-lines must be greater than or equal to 0")
  }

  let hunks = ()
  let current = none

  for op in report.ops {
    if op.kind == "equal" {
      if current != none {
        let prefix = op.row_len - calc.min(op.row_len, context-lines)
        current.ops.push(op)
        current.row_end = op.row_start + op.row_len
        current.context_after = calc.min(op.row_len, context-lines)

        if prefix > context-lines {
          current.row_end = op.row_start + context-lines
          hunks.push(_finish_hunk(report, current))
          current = none
        }
      }
    } else {
      if current == none {
        current = (
          ops: (),
          row_start: op.row_start,
          row_end: op.row_start,
          context_before: 0,
          context_after: 0,
        )

        let previous = _previous_op(report.ops, op)
        if previous != none and previous.kind == "equal" {
          current.ops.push(previous)
          current.context_before = calc.min(previous.row_len, context-lines)
          current.row_start = previous.row_start + previous.row_len - current.context_before
        }
      }

      current.ops.push(op)
      current.row_end = op.row_start + op.row_len
    }
  }

  if current != none {
    hunks.push(_finish_hunk(report, current))
  }

  hunks
}

#let diffst-summary(report, colors: (:)) = {
  let colors = default-colors + colors
  _summary(colors, report)
}

#let diffst-table(report, rows: auto, colors: (:)) = {
  let colors = default-colors + colors
  let rows = if rows == auto { report.rows } else { rows }
  _diff-table(colors, rows)
}

#let diffst-layout(
  report,
  colors: (:),
  display: "collapsed",
  collapse-threshold: 14,
  context-lines: 3,
  body: auto,
) = {
  let colors = default-colors + colors
  let rows = diffst-rows(
    report,
    display: display,
    collapse-threshold: collapse-threshold,
    context-lines: context-lines,
  )

  if body == auto {
    block(width: 100%)[
      #_summary(colors, report)
      #v(6pt)
      #_diff-table(colors, rows)
    ]
  } else {
    body(report, rows, colors)
  }
}

#let _display(it) = {
  let report = diffst-report(
    it.old,
    it.new,
    ignore-whitespace: it.at("ignore-whitespace"),
    show-whitespace: it.at("show-whitespace"),
    algorithm: it.algorithm,
    inline: it.inline,
    semantic-cleanup: it.at("semantic-cleanup"),
  )

  diffst-layout(
    report,
    colors: it.colors,
    display: it.display,
    collapse-threshold: it.at("collapse-threshold"),
    context-lines: it.at("context-lines"),
  )
}

#let diffst = e.element.declare(
  "diffst",
  prefix: "diffst,v1",
  doc: "Renders a side-by-side diff report for two text files.",
  display: _display,
  fields: (
    e.field("old", str, doc: "Path to the old file.", required: true),
    e.field("new", str, doc: "Path to the new file.", required: true),
    e.field("ignore-whitespace", bool, doc: "Ignore whitespace while diffing lines.", default: false),
    e.field("show-whitespace", bool, doc: "Render changed spaces and tabs visibly in inline highlights.", default: false),
    e.field("algorithm", str, doc: "Diff algorithm: \"myers\", \"patience\", \"lcs\", \"hunt\", or \"histogram\".", default: "myers"),
    e.field("inline", str, doc: "Inline highlighting mode: \"chars\", \"words\", or \"none\".", default: "chars"),
    e.field("semantic-cleanup", bool, doc: "Run similar's semantic cleanup pass on inline highlights.", default: false),
    e.field("display", str, doc: "Either \"collapsed\" or \"full\".", default: "collapsed"),
    e.field("collapse-threshold", int, doc: "Minimum unchanged run length before collapsed display hides the middle.", default: 14),
    e.field("context-lines", int, doc: "Unchanged lines to keep on each side of a collapsed region.", default: 3),
    e.field("colors", e.types.dict(e.types.any), doc: "Color overrides merged with `default-colors`.", default: (:)),
  ),
)
