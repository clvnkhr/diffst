#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

#let code-size = 7pt
#let old-fill = rgb("#ffe3e6")
#let new-fill = rgb("#dcf7e4")
#let same-fill = rgb("#ffffff")
#let border = rgb("#cbd5e1")
#let rule = 0.45pt + border
#let inset = (x: 4pt, y: 3pt)
#let no-w = 10mm
#let content-w = 126.5mm

#let code(body) = text(size: code-size)[#raw(body, block: false)]
#let line-no(n) = text(size: code-size, fill: rgb("#64748b"))[#raw(str(n), block: false)]
#let long-copy-line = "let summary = \"This long copy test line is deliberately wide enough to wrap in a constrained box, and the question is whether copying from the PDF preserves spaces or inserts line breaks at visual wraps.\""

#let rows = (
  (kind: "equal", old-no: 1, old: "let title = \"Draft\"", new-no: 1, new: "let title = \"Draft\""),
  (kind: "replace", old-no: 2, old: "let status = \"rough\"", new-no: 2, new: "let status = \"ready\""),
  (kind: "replace", old-no: 3, old: "let spacing = \"old\"", new-no: 3, new: "let spacing = \"new\""),
  (kind: "delete", old-no: 4, old: "let removed = true", new-no: none, new: ""),
  (kind: "insert", old-no: none, old: "", new-no: 4, new: "let added = true"),
  (
    kind: "replace",
    old-no: 5,
    old: "let summary = \"This old sentence is deliberately long enough to wrap inside the old content column, and it keeps going so the row height has to be measured rather than guessed.\"",
    new-no: 5,
    new: "let summary = \"This new sentence is deliberately long enough to wrap inside the new content column, and it keeps going with a different ending so synced row heights have to survive wrapping.\"",
  ),
  (kind: "equal", old-no: 6, old: "render(document)", new-no: 6, new: "render(document)"),
)

#let fill(kind, side) = {
  if kind == "equal" {
    same-fill
  } else if side == "old" and (kind == "delete" or kind == "replace") {
    old-fill
  } else if side == "new" and (kind == "insert" or kind == "replace") {
    new-fill
  } else {
    same-fill
  }
}

#let single-table(rows) = table(
  columns: (2.4em, 1fr, 2.4em, 1fr),
  stroke: border,
  inset: (x: 4pt, y: 3pt),
  table.header([Old], [Content], [New], [Content]),
  ..rows.map(row => (
    table.cell(fill: fill(row.kind, "old"), align: right)[#if row.old-no != none { line-no(row.old-no) }],
    table.cell(fill: fill(row.kind, "old"))[#code(row.old)],
    table.cell(fill: fill(row.kind, "new"), align: right)[#if row.new-no != none { line-no(row.new-no) }],
    table.cell(fill: fill(row.kind, "new"))[#code(row.new)],
  )).flatten(),
)

#let half-table(rows, side) = table(
  columns: (2.4em, 1fr),
  stroke: border,
  inset: (x: 4pt, y: 3pt),
  table.header(if side == "old" { [Old] } else { [New] }, [Content]),
  ..rows.map(row => {
    let no = if side == "old" { row.old-no } else { row.new-no }
    let content = if side == "old" { row.old } else { row.new }
    (
      table.cell(fill: fill(row.kind, side), align: right)[#if no != none { line-no(no) }],
      table.cell(fill: fill(row.kind, side))[#code(content)],
    )
  }).flatten(),
)

#let measured-cell(width, body) = box(width: width)[
  #pad(..inset)[#body]
]

#let fixed-cell(width, height, fill, body, align: left, stroke: auto) = {
  let args = if stroke == auto { (:) } else { (stroke: stroke) }
  table.cell(
    fill: fill,
    inset: 0pt,
    align: align,
    ..args,
  )[
    #box(width: width, height: height)[
      #pad(..inset)[#body]
    ]
  ]
}

#let full-row-prototype(row) = table(
  columns: (no-w, content-w, no-w, content-w),
  stroke: none,
  inset: 0pt,
  fixed-cell(no-w, auto, fill(row.kind, "old"), if row.old-no == none { [] } else { line-no(row.old-no) }, align: right),
  fixed-cell(content-w, auto, fill(row.kind, "old"), code(row.old)),
  fixed-cell(no-w, auto, fill(row.kind, "new"), if row.new-no == none { [] } else { line-no(row.new-no) }, align: right),
  fixed-cell(content-w, auto, fill(row.kind, "new"), code(row.new)),
)

#let split-stroke(index, header: false) = {
  (
    left: if index == 0 { rule } else { none },
    right: rule,
    top: if header { rule } else { none },
    bottom: rule,
  )
}

#let split-column-table(rows, title, side, part, index, heights) = table(
  columns: if part == "no" { (no-w,) } else { (content-w,) },
  stroke: none,
  inset: 0pt,
  table.header(
    fixed-cell(
      if part == "no" { no-w } else { content-w },
      heights.header,
      same-fill,
      title,
      align: if part == "no" { center } else { left },
      stroke: split-stroke(index, header: true),
    ),
  ),
  ..rows.enumerate().map(((index, row)) => {
    let is-no = part == "no"
    let width = if is-no { no-w } else { content-w }
    let content = if side == "old" and is-no {
      if row.old-no == none { [] } else { line-no(row.old-no) }
    } else if side == "new" and is-no {
      if row.new-no == none { [] } else { line-no(row.new-no) }
    } else if side == "old" {
      code(row.old)
    } else {
      code(row.new)
    }

    fixed-cell(
      width,
      heights.rows.at(index),
      fill(row.kind, side),
      content,
      align: if is-no { right } else { left },
      stroke: split-stroke(if side == "old" and is-no { 0 } else if side == "old" { 1 } else if is-no { 2 } else { 3 }),
    )
  }),
)

#let synced-split-tables(rows) = context {
  let header-prototype = table(
    columns: (no-w, content-w, no-w, content-w),
    stroke: none,
    inset: 0pt,
    fixed-cell(no-w, auto, same-fill, [Old], align: center),
    fixed-cell(content-w, auto, same-fill, [Content]),
    fixed-cell(no-w, auto, same-fill, [New], align: center),
    fixed-cell(content-w, auto, same-fill, [Content]),
  )
  let heights = (
    header: measure(header-prototype).height,
    rows: rows.map(row => measure(full-row-prototype(row)).height),
  )

  grid(
    columns: (no-w, content-w, no-w, content-w),
    gutter: 0pt,
    split-column-table(rows, [Old], "old", "no", 0, heights),
    split-column-table(rows, [Content], "old", "content", 1, heights),
    split-column-table(rows, [New], "new", "no", 2, heights),
    split-column-table(rows, [Content], "new", "content", 3, heights),
  )
}

= Selection Toy

Try dragging across only the old content column, then only the new content
column. The first table is the current-style single table. The later layouts
use independent tables.

== One table

#single-table(rows)

#pagebreak()

== Two independent tables

#grid(
  columns: (1fr, 1fr),
  gutter: 8pt,
  half-table(rows, "old"),
  half-table(rows, "new"),
)

#v(10pt)

== Two tables with a narrow gutter

#grid(
  columns: (1fr, 1fr),
  gutter: 2pt,
  half-table(rows, "old"),
  half-table(rows, "new"),
)

#pagebreak()

== Four independent tables, measured row heights, zero gutter

#synced-split-tables(rows)

#pagebreak()

== Copy behavior for wrapped text

Try copying each row below. They all use the same constrained width.

#table(
  columns: (35mm, 115mm),
  stroke: border,
  inset: (x: 4pt, y: 4pt),
  table.header([variant], [content]),

  [plain text],
  [#box(width: 80mm)[#text(size: code-size)[#long-copy-line]]],

  [inline raw],
  [#box(width: 80mm)[#raw(long-copy-line, block: false)]],

  [inline raw in text],
  [#box(width: 80mm)[#text(size: code-size)[#raw(long-copy-line, block: false)]]],

  [block raw],
  [#box(width: 80mm)[#raw(long-copy-line, block: true)]],

  [block raw clipped],
  [#box(width: 80mm, clip: true)[#raw(long-copy-line, block: true)]],

  [inline raw clipped],
  [#text(top-edge: "bounds", bottom-edge: "bounds")[
    #box(width: 80mm, clip: true)[#raw(long-copy-line, block: false)]
  ]],

  [manual spaces],
  [#box(width: 80mm)[
    #raw("let summary = \"This long copy test line has visible chunks\"", block: false)
    #raw(" and this second raw span continues it.\"", block: false)
  ]],
)
