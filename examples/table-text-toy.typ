#set page(width: 240pt, height: auto, margin: 12pt)
#set text(size: 8pt)

#let mono(body) = text(font: "DejaVu Sans Mono", size: 7pt)[#body]

#let cell(body, fill: none) = table.cell(
  fill: fill,
  inset: (x: 4pt, y: 3pt),
)[#body]

= table text toy

#table(
  columns: (2em, 1fr),
  stroke: gray,
  inset: 0pt,
  table.header(
    cell(strong[No.], fill: luma(235)),
    cell(strong[Content], fill: luma(235)),
  ),
  cell([1]),
  cell(mono[
    This is normal prose with spaces. It should wrap naturally inside the table cell.
  ]),
  cell([2]),
  cell(mono[
    https://example.com/api/v1/documents/2026/06/24/reports/diffst-long-line-behavior/sections/alpha-beta-gamma-delta-epsilon-zeta-eta-theta-iota-kappa-lambda
  ]),
  cell([3]),
  cell(mono[
    sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
  ]),
  cell([4]),
  cell([
    #mono[before ]
    #highlight(fill: yellow)[#mono[highlighted text with spaces should still be able to wrap naturally]]
    #mono[ after]
  ]),
  cell([5]),
  cell([
    #mono[before ]
    #box(fill: yellow, inset: 1pt)[#mono[boxed highlighted text with spaces behaves like one unbreakable inline object]]
    #mono[ after]
  ]),
  cell([6]),
  cell(block(clip: true)[
    #mono[
      clipped block, width only: https://example.com/api/v1/documents/2026/06/24/reports/diffst-long-line-behavior/sections/alpha-beta-gamma-delta-epsilon-zeta-eta-theta-iota-kappa-lambdaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
    ]
  ]),
  cell([7]),
  cell(box(clip: true)[
    #mono[
      clipped box, width only: https://example.com/api/v1/documents/2026/06/24/reports/diffst-long-line-behavior/sections/alpha-beta-gamma-delta-epsilon-zeta-eta-theta-iota-kappa-lambdaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
    ]
  ]),
)
