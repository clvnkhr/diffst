#import "../lib.typ": diffst

#set page(width: 297mm, height: 210mm, margin: 11mm)
#set text(font: "New Computer Modern", size: 8.5pt)

= realistic diffst example

#diffst(
  "examples/paper-old.typ",
  "examples/paper-new.typ",
  inline: "words",
  show-whitespace: true,
  display: "collapsed",
  collapse-threshold: 6,
)
