#import "../lib.typ": diffst, minimal-table

#set page(width: 297mm, height: 210mm, margin: 14mm)
#set text(font: "New Computer Modern", size: 9pt)

#show: minimal-table

= minimal table

#diffst(
  "examples/paper-old.typ",
  "examples/paper-new.typ",
  display: "collapsed",
  context-lines: 2,
  inline: "words",
  semantic-cleanup: true,
)
