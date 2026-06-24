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

#let minimal-colors = (
  text: black,
  line-no: luma(45%),
  border: black,
  header: white,
  equal: white,
  delete: luma(98%),
  insert: luma(99%),
  replace: luma(97%),
  inline-delete: rgb("#f2b6bf"),
  inline-insert: rgb("#a8d5b3"),
  inline-equal: none,
  delete-text: black,
  insert-text: black,
  replace-text: black,
  collapsed: white,
)

#let default-table-style = (
  columns: (2.4em, 1fr, 2.4em, 1fr),
  rules: "default",
  stroke-width: (
    header: 0.8pt,
    body: 0.45pt,
  ),
)

#let minimal-table-style = default-table-style + (
  rules: "minimal",
  stroke-width: 0.6pt,
)

#let _color(colors, key) = colors.at(key, default: default-colors.at(key))
#let _line-size = 6.5pt
#let _code-size = 6.8pt
#let _label-size = 8.5pt
#let _pill-size = 7pt

#let _mono(body, fill: auto, size: _code-size) = {
  let args = if fill == auto { (:) } else { (fill: fill) }
  text(size: size, ..args)[#raw(str(body), block: false)]
}

#let _muted(colors, body, size: _code-size) = {
  text(size: size, fill: _color(colors, "line-no"))[#body]
}

#let _strong(body, size: _label-size) = {
  text(size: size, weight: "bold")[#body]
}

#let _cell(colors, fill, body, align: left, stroke: auto) = {
  let args = if stroke == auto { (:) } else { (stroke: stroke) }
  table.cell(
    fill: fill,
    inset: (x: 4.5pt, y: 3pt),
    align: align,
    ..args,
  )[#body]
}

#let _line-no(colors, value) = {
  if value == none {
    _muted(colors, "", size: _line-size)
  } else {
    _mono(str(value), fill: _color(colors, "line-no"), size: _line-size)
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
  let body = _mono(span.text, fill: _span-text-fill(colors, span.kind))

  if span.kind == "equal" {
    body
  } else {
    highlight(
      fill: _span-fill(colors, span.kind),
    )[#body]
  }
}

#let _code(colors, value, spans: none) = {
  block(clip: true)[
    #if spans != none {
      for span in spans {
        _code-span(colors, span)
      }
    } else if value == none {
      _mono("")
    } else {
      _mono(value, fill: _color(colors, "text"))
    }
  ]
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

#let diffst-pill(fill, fg, body) = box(
  fill: fill,
  inset: (x: 5pt, y: 2pt),
  radius: 2pt,
)[#text(size: _pill-size, fill: fg, weight: "bold")[#body]]

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

#let diffst-labels-raw(report) = (
  old: report.old,
  new: report.new,
)

#let diffst-line-counts-raw(report) = (
  old: report.stats.old_lines,
  new: report.stats.new_lines,
)

#let _stat-info(report, stat) = {
  if stat == "similarity" {
    (
      value: calc.round(report.stats.similarity * 100),
      summary: (
        fill: "collapsed",
        fg: "text",
        label: value => str(value) + "% similar lines",
      ),
    )
  } else if stat == "additions" {
    (
      value: report.stats.additions,
      summary: (
        fill: "insert",
        fg: "insert-text",
        label: value => "+" + str(value),
      ),
    )
  } else if stat == "deletions" {
    (
      value: report.stats.deletions,
      summary: (
        fill: "delete",
        fg: "delete-text",
        label: value => "-" + str(value),
      ),
    )
  } else if stat == "changed-blocks" {
    (
      value: report.stats.changed_blocks,
      summary: (
        fill: "replace",
        fg: "replace-text",
        label: value => str(value) + " changed blocks",
      ),
    )
  } else if stat == "equal-lines" {
    (value: report.stats.equal_lines, summary: none)
  } else if stat == "old-lines" {
    (value: report.stats.old_lines, summary: none)
  } else if stat == "new-lines" {
    (value: report.stats.new_lines, summary: none)
  } else {
    panic("unknown raw stat: " + stat)
  }
}

#let diffst-stat-raw(report, stat) = _stat-info(report, stat).value

#let diffst-stats-raw(
  report,
  stats: ("similarity", "additions", "deletions", "changed-blocks"),
) = {
  stats.map(stat => (
    key: stat,
    value: diffst-stat-raw(report, stat),
  ))
}

#let diffst-row-counts-raw(rows) = {
  let equal = 0
  let insert = 0
  let delete = 0
  let replace = 0
  let collapsed = 0
  let hidden = 0

  for row in rows {
    if row.kind == "equal" {
      equal += 1
    } else if row.kind == "insert" {
      insert += 1
    } else if row.kind == "delete" {
      delete += 1
    } else if row.kind == "replace" {
      replace += 1
    } else if row.kind == "collapsed" {
      collapsed += 1
      hidden += row.hidden
    }
  }

  (
    rows: rows.len(),
    equal: equal,
    insert: insert,
    delete: delete,
    replace: replace,
    collapsed: collapsed,
    hidden: hidden,
  )
}

#let diffst-hunk-raw(hunk) = (
  row-start: hunk.row_start,
  row-end: hunk.row_end,
  rows: hunk.rows.len(),
  ops: hunk.ops.len(),
  old-start: hunk.old_start,
  old-len: hunk.old_len,
  new-start: hunk.new_start,
  new-len: hunk.new_len,
  context-before: hunk.context_before,
  context-after: hunk.context_after,
)

#let diffst-summary-label(report, colors: (:)) = {
  let colors = default-colors + colors
  [
    #_strong(report.old)
    #h(3pt)
    #text(fill: _color(colors, "line-no"))[#math.mapsto]
    #h(3pt)
    #_strong(report.new)
  ]
}

#let diffst-summary-lines(report, colors: (:)) = {
  let colors = default-colors + colors
  _muted(colors, [
    #report.stats.old_lines old lines, #report.stats.new_lines new lines
  ])
}

#let diffst-summary-title(report, colors: (:)) = [
  #diffst-summary-label(report, colors: colors)
  #linebreak()
  #diffst-summary-lines(report, colors: colors)
]

#let diffst-summary-stat(report, stat, colors: (:)) = {
  let colors = default-colors + colors
  let info = _stat-info(report, stat)
  if info.summary == none {
    panic("unknown summary stat: " + stat)
  }

  diffst-pill(
    _color(colors, info.summary.fill),
    _color(colors, info.summary.fg),
    (info.summary.label)(info.value),
  )
}

#let diffst-summary-stats(
  report,
  stats: ("similarity", "additions", "deletions", "changed-blocks"),
  colors: (:),
) = {
  stats.map(stat => diffst-summary-stat(report, stat, colors: colors))
}

#let _summary(
  colors,
  report,
  title: auto,
  stats: ("similarity", "additions", "deletions", "changed-blocks"),
) = {
  let title = if title == auto {
    diffst-summary-title(report, colors: colors)
  } else {
    title
  }

  block[
    #grid(
      columns: (1fr,) + stats.map(_ => auto),
      gutter: 6pt,
      align: horizon,
      title,
      ..diffst-summary-stats(report, stats: stats, colors: colors),
    )
  ]
}

#let _trailing-newline-rows(colors, report) = {
  if report == none or report.meta.old_trailing_newline == report.meta.new_trailing_newline {
    ()
  } else {
    let old-state = if report.meta.old_trailing_newline {
      "old file ends with a newline"
    } else {
      "old file has no trailing newline"
    }
    let new-state = if report.meta.new_trailing_newline {
      "new file ends with a newline"
    } else {
      "new file has no trailing newline"
    }

    (
      table.cell(colspan: 4, fill: _color(colors, "collapsed"), inset: (x: 4pt, y: 3pt), align: center)[
        #_muted(colors, [#old-state; #new-state], size: _line-size)
      ],
    )
  }
}

#let _resolve-table-style(table-style) = {
  if type(table-style) == str {
    if table-style == "default" {
      default-table-style
    } else if table-style == "minimal" {
      minimal-table-style
    } else {
      panic("table-style must be \"default\", \"minimal\", or a table style dictionary")
    }
  } else {
    default-table-style + table-style
  }
}

#let _table-stroke(colors, table-style) = {
  if table-style.rules == "default" {
    (x, y) => (
      paint: _color(colors, "border"),
      thickness: if y == 0 { table-style.stroke-width.header } else { table-style.stroke-width.body },
    )
  } else if table-style.rules == "minimal" {
    none
  } else {
    panic("table-style.rules must be \"default\" or \"minimal\"")
  }
}

#let _minimal-cell-stroke(colors, table-style, column, header: false) = {
  let rule = table-style.stroke-width + _color(colors, "border")
  if header and column == 1 {
    (right: rule, bottom: rule)
  } else if header {
    (bottom: rule)
  } else if column == 1 {
    (right: rule)
  } else {
    none
  }
}

#let _cell-stroke(colors, table-style, column, header: false) = {
  if table-style.rules == "minimal" {
    _minimal-cell-stroke(colors, table-style, column, header: header)
  } else {
    auto
  }
}

#let _diff-table(colors, rows, report: none, table-style: default-table-style) = {
  let table-style = _resolve-table-style(table-style)
  table(
  columns: table-style.columns,
  stroke: _table-stroke(colors, table-style),
  inset: 0pt,
  table.header(
    repeat: true,
    _cell(colors, _color(colors, "header"), _strong([Old], size: _line-size), align: center, stroke: _cell-stroke(colors, table-style, 0, header: true)),
    _cell(colors, _color(colors, "header"), _strong([Content], size: _line-size), stroke: _cell-stroke(colors, table-style, 1, header: true)),
    _cell(colors, _color(colors, "header"), _strong([New], size: _line-size), align: center, stroke: _cell-stroke(colors, table-style, 2, header: true)),
    _cell(colors, _color(colors, "header"), _strong([Content], size: _line-size), stroke: _cell-stroke(colors, table-style, 3, header: true)),
  ),
  ..rows.map(row => {
    if row.kind == "collapsed" {
      (
        table.cell(colspan: 4, fill: _color(colors, "collapsed"), inset: (x: 4pt, y: 3pt), align: center)[
          #_muted(colors, [#row.hidden unchanged lines hidden], size: _line-size)
        ],
      )
    } else {
      let fill = _row-fill(colors, row.kind)
      (
        _cell(colors, fill, _line-no(colors, row.at("old_no", default: none)), align: right, stroke: _cell-stroke(colors, table-style, 0)),
        _cell(colors, fill, _code(
          colors,
          row.at("old", default: none),
          spans: row.at("old_spans", default: none),
        ), stroke: _cell-stroke(colors, table-style, 1)),
        _cell(colors, fill, _line-no(colors, row.at("new_no", default: none)), align: right, stroke: _cell-stroke(colors, table-style, 2)),
        _cell(colors, fill, _code(
          colors,
          row.at("new", default: none),
          spans: row.at("new_spans", default: none),
        ), stroke: _cell-stroke(colors, table-style, 3)),
      )
    }
  }).flatten(),
  .._trailing-newline-rows(colors, report),
  )
}

#let diffst-report(
  old,
  new,
  ignore-whitespace: false,
  show-whitespace: false,
  algorithm: "myers",
  inline: "chars",
  unicode: true,
  semantic-cleanup: false,
) = {
  let old-content = read(old)
  let new-content = read(new)
  let options = json.encode((
    ignore_whitespace: ignore-whitespace,
    show_whitespace: show-whitespace,
    algorithm: algorithm,
    inline: inline,
    unicode: unicode,
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
  if collapse-threshold < 0 {
    panic("collapse-threshold must be greater than or equal to 0")
  }

  if display == "full" {
    report.rows
  } else if display == "collapsed" {
    _with-collapse(report.rows, collapse-threshold, context-lines)
  } else {
    panic("display must be \"full\" or \"collapsed\"")
  }
}

#let _empty_hunk(row-start) = (
  ops: (),
  row_start: row-start,
  row_end: row-start,
  context_before: 0,
  context_after: 0,
)

#let _start_hunk(previous, op, context-lines) = {
  let hunk = _empty_hunk(op.row_start)

  if previous != none and previous.kind == "equal" {
    hunk.ops.push(previous)
    hunk.context_before = calc.min(previous.row_len, context-lines)
    hunk.row_start = previous.row_start + previous.row_len - hunk.context_before
  }

  hunk
}

#let _equal_gap_exceeds_context(op, context-lines) = {
  let hidden-prefix = op.row_len - calc.min(op.row_len, context-lines)
  hidden-prefix > context-lines
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
  let previous = none

  for op in report.ops {
    if op.kind == "equal" {
      if current != none {
        current.ops.push(op)
        current.row_end = op.row_start + op.row_len
        current.context_after = calc.min(op.row_len, context-lines)

        if _equal_gap_exceeds_context(op, context-lines) {
          current.row_end = op.row_start + context-lines
          hunks.push(_finish_hunk(report, current))
          current = none
        }
      }
    } else {
      if current == none {
        current = _start_hunk(previous, op, context-lines)
      }

      current.ops.push(op)
      current.row_end = op.row_start + op.row_len
    }

    previous = op
  }

  if current != none {
    hunks.push(_finish_hunk(report, current))
  }

  hunks
}

#let diffst-hunks-raw(report, context-lines: 3) = {
  diffst-hunks(report, context-lines: context-lines).map(diffst-hunk-raw)
}

#let diffst-debug-raw(report, rows: auto, context-lines: 3) = {
  let rows = if rows == auto { report.rows } else { rows }
  let hunks = diffst-hunks-raw(report, context-lines: context-lines)
  (
    meta: report.meta,
    stats: report.stats,
    rows: diffst-row-counts-raw(rows),
    ops: report.ops.len(),
    hunks: hunks.len(),
    messages: report.meta.messages,
  )
}

#let _debug-value(value) = {
  if value == true {
    "true"
  } else if value == false {
    "false"
  } else if value == none {
    "none"
  } else {
    str(value)
  }
}

#let _debug-row(colors, label, value) = (
  table.cell(fill: _color(colors, "collapsed"), inset: (x: 4pt, y: 2.5pt))[
    #_strong(label, size: _code-size)
  ],
  table.cell(inset: (x: 4pt, y: 2.5pt))[
    #text(size: _code-size)[#value]
  ],
)

#let diffst-debug(
  report,
  rows: auto,
  colors: (:),
  context-lines: 3,
  max-messages: 8,
) = {
  let colors = default-colors + colors
  let raw = diffst-debug-raw(report, rows: rows, context-lines: context-lines)
  let row-counts = raw.rows
  let messages = raw.messages.slice(0, calc.min(max-messages, raw.messages.len()))

  block[
    #grid(
      columns: (1fr, auto),
      gutter: 6pt,
      align: horizon,
      [
        #_strong([diffst debug])
        #linebreak()
        #_muted(colors, [
          #report.old #math.mapsto #report.new
        ])
      ],
      diffst-pill(
        _color(colors, "collapsed"),
        _color(colors, "text"),
        str(raw.ops) + " ops / " + str(row-counts.rows) + " rows",
      ),
    )
    #v(5pt)
    #table(
      columns: (auto, 1fr),
      stroke: _color(colors, "border"),
      inset: (x: 4pt, y: 2.5pt),
      .._debug-row(colors, "algorithm", raw.meta.algorithm),
      .._debug-row(colors, "inline", raw.meta.inline),
      .._debug-row(colors, "unicode", _debug-value(raw.meta.unicode)),
      .._debug-row(colors, "ignore whitespace", _debug-value(raw.meta.ignore_whitespace)),
      .._debug-row(colors, "show whitespace", _debug-value(raw.meta.show_whitespace)),
      .._debug-row(colors, "semantic cleanup", _debug-value(raw.meta.semantic_cleanup)),
      .._debug-row(colors, "old trailing newline", _debug-value(raw.meta.old_trailing_newline)),
      .._debug-row(colors, "new trailing newline", _debug-value(raw.meta.new_trailing_newline)),
      .._debug-row(colors, "old line endings", raw.meta.old_line_endings),
      .._debug-row(colors, "new line endings", raw.meta.new_line_endings),
      .._debug-row(colors, "old/new lines", str(raw.stats.old_lines) + " / " + str(raw.stats.new_lines)),
      .._debug-row(colors, "equal lines", str(raw.stats.equal_lines)),
      .._debug-row(colors, "line similarity", str(calc.round(raw.stats.similarity * 100)) + "%"),
      .._debug-row(colors, "visible rows", str(row-counts.rows)),
      .._debug-row(colors, "hidden rows", str(row-counts.hidden)),
      .._debug-row(colors, "hunks", str(raw.hunks)),
    )
    #if messages.len() > 0 [
      #v(5pt)
      #_strong([messages], size: _pill-size)
      #v(2pt)
      #for message in messages [
        #_muted(colors, [- #message])
        #linebreak()
      ]
    ]
  ]
}

#let diffst-summary(
  report,
  colors: (:),
  title: auto,
  stats: ("similarity", "additions", "deletions", "changed-blocks"),
  body: auto,
) = {
  let colors = default-colors + colors
  if body == auto {
    _summary(colors, report, title: title, stats: stats)
  } else {
    body(report, colors)
  }
}

#let diffst-table(report, rows: auto, colors: (:), table-style: default-table-style) = {
  let colors = default-colors + colors
  let rows = if rows == auto { report.rows } else { rows }
  _diff-table(colors, rows, report: report, table-style: table-style)
}

#let diffst-layout(
  report,
  colors: (:),
  display: "collapsed",
  collapse-threshold: 14,
  context-lines: 3,
  table-style: default-table-style,
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
      #_diff-table(colors, rows, report: report, table-style: table-style)
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
    unicode: it.at("unicode"),
    semantic-cleanup: it.at("semantic-cleanup"),
  )

  diffst-layout(
    report,
    colors: it.colors,
    display: it.display,
    collapse-threshold: it.at("collapse-threshold"),
    context-lines: it.at("context-lines"),
    table-style: it.at("table-style"),
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
    e.field("unicode", bool, doc: "Use Unicode-aware inline tokenization for graphemes and word boundaries.", default: true),
    e.field("semantic-cleanup", bool, doc: "Run similar's semantic cleanup pass on inline highlights.", default: false),
    e.field("display", str, doc: "Either \"collapsed\" or \"full\".", default: "collapsed"),
    e.field("collapse-threshold", int, doc: "Minimum unchanged run length before collapsed display hides the middle.", default: 14),
    e.field("context-lines", int, doc: "Unchanged lines to keep on each side of a collapsed region.", default: 3),
    e.field("table-style", e.types.any, doc: "Table style dictionary, or \"default\"/\"minimal\".", default: default-table-style),
    e.field("colors", e.types.dict(e.types.any), doc: "Color overrides merged with `default-colors`.", default: (:)),
  ),
)

#let minimal-table(body) = {
  show: e.set_(diffst, colors: minimal-colors, table-style: minimal-table-style)
  body
}
