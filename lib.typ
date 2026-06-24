#let _engine = plugin("target/wasm32-unknown-unknown/release/diffst_wasm.wasm")

#let _colors = (
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

#let _cell(fill, body, align: left) = table.cell(
  fill: fill,
  inset: (x: 4.5pt, y: 3pt),
  align: align,
)[#body]

#let _line-no(value) = {
  if value == none {
    text(fill: _colors.line-no, size: 6.5pt)[]
  } else {
    text(fill: _colors.line-no, size: 6.5pt, font: "DejaVu Sans Mono")[#str(value)]
  }
}

#let _span-fill(kind) = {
  if kind == "delete" {
    _colors.inline-delete
  } else if kind == "insert" {
    _colors.inline-insert
  } else {
    _colors.inline-equal
  }
}

#let _span-text-fill(kind) = {
  if kind == "delete" {
    _colors.delete-text
  } else if kind == "insert" {
    _colors.insert-text
  } else {
    _colors.text
  }
}

#let _code-span(span) = {
  let body = text(
    size: 6.8pt,
    font: "DejaVu Sans Mono",
    fill: _span-text-fill(span.kind),
  )[#span.text]

  if span.kind == "equal" {
    body
  } else {
    box(
      fill: _span-fill(span.kind),
      inset: (x: 0.7pt, y: 0.4pt),
      outset: (y: 0.2pt),
      radius: 1pt,
    )[#body]
  }
}

#let _code(value, spans: none) = {
  if spans != none {
    for span in spans {
      _code-span(span)
    }
  } else if value == none {
    text(size: 6.8pt, font: "DejaVu Sans Mono")[]
  } else {
    text(size: 6.8pt, font: "DejaVu Sans Mono", fill: _colors.text)[#value]
  }
}

#let _row-fill(kind) = {
  if kind == "delete" {
    _colors.delete
  } else if kind == "insert" {
    _colors.insert
  } else if kind == "replace" {
    _colors.replace
  } else {
    _colors.equal
  }
}

#let _pill(fill, fg, body) = box(
  fill: fill,
  inset: (x: 5pt, y: 2pt),
  radius: 2pt,
)[#text(size: 7pt, fill: fg, weight: "bold")[#body]]

#let _flush_equal_run(run, threshold) = {
  if run.len() > threshold {
    let keep = calc.min(3, run.len())
    let hidden = run.len() - keep * 2
    run.slice(0, keep) + ((
      kind: "collapsed",
      hidden: hidden,
    ),) + run.slice(run.len() - keep)
  } else {
    run
  }
}

#let _with-collapse(rows, threshold) = {
  let output = ()
  let equal-run = ()

  for row in rows {
    if row.kind == "equal" {
      equal-run.push(row)
    } else {
      output += _flush_equal_run(equal-run, threshold)
      equal-run = ()
      output.push(row)
    }
  }

  output + _flush_equal_run(equal-run, threshold)
}

#let _summary(stats, old, new) = block[
  #grid(
    columns: (1fr, auto, auto, auto),
    gutter: 6pt,
    align: horizon,
    [
      #text(size: 8.5pt, weight: "bold")[#old]
      #h(3pt)
      #text(fill: _colors.line-no)[->]
      #h(3pt)
      #text(size: 8.5pt, weight: "bold")[#new]
      #linebreak()
      #text(size: 6.8pt, fill: _colors.line-no)[
        #stats.old_lines old lines, #stats.new_lines new lines
      ]
    ],
    _pill(_colors.insert, _colors.insert-text, "+" + str(stats.additions)),
    _pill(_colors.delete, _colors.delete-text, "-" + str(stats.deletions)),
    _pill(_colors.replace, _colors.replace-text, str(stats.changed_blocks) + " changed blocks"),
  )
]

#let _diff-table(rows) = table(
  columns: (2.4em, 1fr, 2.4em, 1fr),
  stroke: (x, y) => (
    paint: _colors.border,
    thickness: if y == 0 { 0.8pt } else { 0.45pt },
  ),
  inset: 0pt,
  _cell(_colors.header, text(size: 6.5pt, weight: "bold")[Old], align: center),
  _cell(_colors.header, text(size: 6.5pt, weight: "bold")[Content]),
  _cell(_colors.header, text(size: 6.5pt, weight: "bold")[New], align: center),
  _cell(_colors.header, text(size: 6.5pt, weight: "bold")[Content]),
  ..rows.map(row => {
    if row.kind == "collapsed" {
      (
        table.cell(colspan: 4, fill: _colors.collapsed, inset: (x: 4pt, y: 3pt), align: center)[
          #text(size: 6.5pt, fill: _colors.line-no)[#row.hidden unchanged lines hidden]
        ],
      )
    } else {
      let fill = _row-fill(row.kind)
      (
        _cell(fill, _line-no(row.at("old_no", default: none)), align: right),
        _cell(fill, _code(
          row.at("old", default: none),
          spans: row.at("old_spans", default: none),
        )),
        _cell(fill, _line-no(row.at("new_no", default: none)), align: right),
        _cell(fill, _code(
          row.at("new", default: none),
          spans: row.at("new_spans", default: none),
        )),
      )
    }
  }).flatten(),
)

#let diffst(
  old,
  new,
  ignore-whitespace: false,
  show-whitespace: false,
  display: "collapsed",
  collapse-threshold: 14,
) = {
  let old-content = read(old)
  let new-content = read(new)
  let options = json.encode((
    ignore_whitespace: ignore-whitespace,
    show_whitespace: show-whitespace,
  ))
  let report = json(_engine.diff(bytes(old-content), bytes(new-content), bytes(options)))

  if "error" in report {
    panic(report.error)
  }

  let rows = if display == "full" {
    report.rows
  } else if display == "collapsed" {
    _with-collapse(report.rows, collapse-threshold)
  } else {
    panic("display must be \"full\" or \"collapsed\"")
  }

  block(width: 100%)[
    #_summary(report.stats, old, new)
    #v(6pt)
    #_diff-table(rows)
  ]
}
