#import "../lib.typ": diffst

#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 8.5pt)

#let case(title, old, new, algorithm, note) = {
  heading(level: 2)[#title]
  note

  heading(level: 3)[Myers baseline]
  diffst(
    old,
    new,
    algorithm: "myers",
    collapse-threshold: 4,
  )

  pagebreak(weak: true)

  heading(level: 3)[#algorithm]
  diffst(
    old,
    new,
    algorithm: algorithm,
    collapse-threshold: 4,
  )

  pagebreak(weak: true)
}

= Algorithm showcase

These fixtures are synthetic, but shaped after common real review problems:
reordered sections, code with repeated scaffolding, and duplicate-heavy logs.
Each case shows Myers first as the baseline, followed by another algorithm on
the same input pair.

#case(
  [Patience: reordered sections with unique anchors],
  "examples/algorithm-cases/patience-old.typ",
  "examples/algorithm-cases/patience-new.typ",
  "patience",
  [
    Patience diff tends to be easier to read when distinctive lines, such as
    section headings, can anchor moved or reorganized blocks.
  ],
)

#case(
  [Histogram: repeated code scaffolding],
  "examples/algorithm-cases/histogram-old.typ",
  "examples/algorithm-cases/histogram-new.typ",
  "histogram",
  [
    Histogram diff is useful for code with many repeated low-information lines,
    such as braces, trace setup, and repeated route handlers.
  ],
)

#case(
  [Hunt: duplicate-heavy sequences],
  "examples/algorithm-cases/duplicates-old.typ",
  "examples/algorithm-cases/duplicates-new.typ",
  "hunt",
  [
    Hunt is a useful comparison point for sequences with many repeated lines,
    such as logs, checklists, and experiment timelines.
  ],
)

#case(
  [LCS: duplicate-heavy sequences],
  "examples/algorithm-cases/duplicates-old.typ",
  "examples/algorithm-cases/duplicates-new.typ",
  "lcs",
  [
    LCS is another baseline-style algorithm to compare against Myers on
    duplicate-heavy input.
  ],
)
