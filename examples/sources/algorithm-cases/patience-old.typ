= Release Review Notes

== Overview

The report generator should keep the important anchors visible when a document
is reorganized. The first draft groups each feature by implementation order.

== Parser

The parser reads UTF-8 text and splits it into lines.
It records the source path for later display.
It reports invalid input before rendering begins.

== Renderer

The renderer builds a table with old and new columns.
It highlights inserted and deleted rows.
It uses muted colors to keep the report printable.

== Reviewer Checklist

- confirm that moved sections are still readable
- confirm that line numbers remain stable
- confirm that inline changes are visible
