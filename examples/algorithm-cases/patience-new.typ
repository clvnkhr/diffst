= Release Review Notes

== Overview

The report generator should keep the important anchors visible when a document
is reorganized. The second draft groups each feature by review priority.

== Parser

The parser reads UTF-8 text and splits it into lines.
It records the source path and display label for later use.
It reports invalid input before rendering begins.

== Reviewer Checklist

- confirm that moved sections are still readable
- confirm that line numbers remain stable
- confirm that inline changes are visible
- confirm that collapsed context does not hide intent

== Renderer

The renderer builds a table with old and new columns.
It highlights inserted, deleted, and replaced rows.
It uses muted colors to keep the report printable.
