= Incremental Summaries for Patch Review

#let sample-size = 240

== Abstract

Reviewers often need to understand a change before they can decide whether the
implementation is correct. We study a lightweight report format that combines
line-level diffs with short document structure cues. The goal is to keep the
report close to the source while still making the changed regions easy to scan.

== Introduction

Modern code review tools are optimized for interactive browsing. They are less
convenient when a team wants a stable artifact for a design review, a research
appendix, or an audit trail. A printable diff must be compact, but it also needs
to preserve the details that make a patch reviewable.

The first prototype renders every line in a two-column table. This is simple to
implement and straightforward to inspect, but long unchanged regions consume too
much space. It also treats a one-character edit the same way as a rewritten
paragraph, which makes small edits harder to see.

== Method

We parse both documents as UTF-8 text and split the content into lines. A Myers
diff finds equal, inserted, deleted, and replaced line ranges. Each range is
converted into rows that carry line numbers from the old and new documents.

For replaced lines we apply a second diff over characters. Inline spans are
emitted with a tag so the renderer can emphasize the exact deleted or inserted
characters. The renderer remains responsible for page layout and colors.

```rust
fn normalize_line(line: &str, ignore_ws: bool) -> String {
    if ignore_ws {
        line.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        line.to_owned()
    }
}
```

The document renderer uses a table with four columns: old line number, old
content, new line number, and new content. The implementation currently favors a
wide landscape page because code lines need horizontal space.

=== Report Requirements

- show additions and deletions with distinct colors
- preserve line numbers from both inputs
- support a mode that hides large unchanged regions
- produce a report that can be committed as a PDF

== Evaluation

We evaluated the prototype on five small patches and three paper drafts. The
patches contained configuration changes, documentation edits, and simple Rust
functions. The paper drafts contained paragraph rewrites and table updates.

#table(
  columns: (1fr, auto, auto),
  [Case], [Lines], [Changed],
  [config patch], [88], [12],
  [paper draft], [420], [36],
  [library module], [214], [31],
)

The table output was readable for short examples. For larger drafts, unchanged
context dominated the report. Inline changes were especially useful for prose
because they made a changed adjective or numeric value visible without forcing
the reader to compare the full sentence manually.

== Limitations

The prototype assumes that the files are valid UTF-8. It does not understand
syntax, citations, or rendered Typst structure. The report is also sensitive to
manual line wrapping, so a reflowed paragraph can appear as a large replacement.

== Conclusion

A Typst-native diff report is practical for small code reviews and research
drafts. The next version should improve the visual hierarchy and expose options
for whitespace handling.

