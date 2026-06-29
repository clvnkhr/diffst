#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from textwrap import dedent
from typing import overload, cast


DOCUMENT_TEMPLATE = """
#import {import_path}: diffst-layout, diffst-report

#set page(width: 297mm, height: auto, margin: 12mm)

#let back-to-top = align(right)[#link(<top>)[Back to top]]

#let diffst-outline-report-stats(report) = {{
  let stats = report.stats
  [
    #stats.old_lines old lines, #stats.new_lines new lines
    #h(0.8em)
    #str(calc.round(stats.similarity * 100))% similar lines
    #h(0.8em)
    +#stats.additions
    #h(0.8em)
    -#stats.deletions
    #h(0.8em)
    #stats.changed_blocks changed blocks
  ]
}}

{definitions}

#let diffst-outline-summary(location) = {{
{outline_summary_cases}
  none
}}

#let diffst-outline-entry(entry) = context block(width: 100%)[
  #let summary = diffst-outline-summary(entry.element.location())
  #grid(
    columns: (auto, 1fr, auto),
    column-gutter: 0.8em,
    link(entry.element.location())[#entry.element.body],
    entry.fill,
    if summary != none {{
      text(size: 0.82em, fill: rgb("#6b7280"))[#summary]
    }},
  )
]

#title[Git diff {old_short} to {new_short} <top>]

#show outline.entry: diffst-outline-entry
#outline()

{body}
"""

TEXT_FILE_DEFINITION_TEMPLATE = """
#let {report_name} = diffst-report(
  {old_text},
  {new_text},
  old-label: {old_label},
  new-label: {new_label},
)
"""

TEXT_FILE_SECTION_TEMPLATE = """
{heading}

#diffst-layout({report_name})

#back-to-top
"""

BINARY_FILE_SECTION_TEMPLATE = """
{heading}

    `{status}` #{path} is binary or not UTF-8 text, so it was not rendered.

- Old size: {old_size}
- New size: {new_size}
- Delta: {size_delta}

#back-to-top
"""

OUTLINE_REPORT_CASE_TEMPLATE = """
  if location == locate(<{label}>) {{
    return diffst-outline-report-stats({report_name})
  }}
"""

OUTLINE_TEXT_CASE_TEMPLATE = """
  if location == locate(<{label}>) {{
    return [{summary}]
  }}
"""

NO_CHANGES_TEMPLATE = """
_No files changed._
"""


def typst_string(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def typst_json_value(value: str) -> str:
    return f"json(bytes({typst_string(json.dumps(value, ensure_ascii=False))}))"


def typst_raw(value: str) -> str:
    return f"raw({typst_string(value)}, block: false)"


def typst_file_heading(value: str, *, label: str) -> str:
    text = value.replace("\n", " ").strip() or "unknown"
    return f"#heading(level: 1)[#{typst_raw(text)}] <{label}>"


@dataclass(frozen=True)
class RenderedFile:
    definition: str
    section: str
    outline_case: str


@dataclass(frozen=True)
class ChangedFile:
    status: str
    old_path: str | None
    new_path: str | None

    @property
    def display_path(self) -> str:
        return self.new_path or self.old_path or "unknown"


def _git_error_message(err: subprocess.CalledProcessError) -> str:
    stderr = err.stderr
    if isinstance(stderr, bytes):
        return stderr.decode("utf-8", "replace").strip()
    return str(stderr).strip()


@overload
def run_git(args: list[str], output: type[str] = str) -> str: ...


@overload
def run_git(args: list[str], output: type[bytes]) -> bytes: ...


def run_git(args: list[str], output: type[str] | type[bytes] = str) -> str | bytes:
    if output not in (str, bytes):
        raise TypeError("git output type must be str or bytes")

    try:
        result = subprocess.run(
            ["git", *args],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=output is str,
        )
    except subprocess.CalledProcessError as err:
        message = _git_error_message(err)
        raise SystemExit(message.strip() or f"git {' '.join(args)} failed") from err
    if output is str:
        return cast(str, result.stdout)
    return cast(bytes, result.stdout)


def resolve_commit(rev: str) -> tuple[str, str]:
    full = run_git(["rev-parse", "--verify", f"{rev}^{{commit}}"]).strip()
    short = run_git(["rev-parse", "--short", full]).strip()
    return full, short


def changed_files(old_commit: str, new_commit: str) -> list[ChangedFile]:
    raw = run_git(
        ["diff", "--name-status", "--find-renames", "-z", old_commit, new_commit],
        bytes,
    )
    parts = raw.decode("utf-8", "surrogateescape").split("\0")
    if parts and parts[-1] == "":
        parts.pop()

    files: list[ChangedFile] = []
    index = 0
    while index < len(parts):
        status = parts[index]
        index += 1

        if status.startswith(("R", "C")):
            old_path = parts[index]
            new_path = parts[index + 1]
            index += 2
        else:
            path = parts[index]
            index += 1
            old_path = None if status == "A" else path
            new_path = None if status == "D" else path

        files.append(ChangedFile(status=status, old_path=old_path, new_path=new_path))

    return files


def git_show(commit: str, path: str | None) -> bytes:
    if path is None:
        return b""
    return run_git(["show", f"{commit}:{path}"], bytes)


def format_size(size: int) -> str:
    units = ("B", "KiB", "MiB", "GiB")
    value = float(size)
    for unit in units:
        if value < 1024 or unit == units[-1]:
            if unit == "B":
                return f"{size} B"
            return f"{value:.1f} {unit}"
        value /= 1024
    raise AssertionError("unreachable")


def format_delta(delta: int) -> str:
    sign = "+" if delta >= 0 else "-"
    return sign + format_size(abs(delta))


def size_summary(old_bytes: bytes, new_bytes: bytes) -> str:
    return (
        f"{format_size(len(old_bytes))} -> {format_size(len(new_bytes))} "
        f"({format_delta(len(new_bytes) - len(old_bytes))})"
    )


def render_template(template: str, **values: str) -> str:
    return dedent(template).strip("\n").format(**values)


def render_file(
    file: ChangedFile,
    *,
    index: int,
    old_commit: str,
    new_commit: str,
    old_short: str,
    new_short: str,
) -> RenderedFile:
    label = f"diffst-file-{index}"
    report_name = f"diffst_file_{index}_report"
    old_label_path = file.old_path or file.display_path
    new_label_path = file.new_path or file.display_path
    old_label = f"{old_label_path} @ {old_short}"
    new_label = f"{new_label_path} @ {new_short}"

    old_bytes = git_show(old_commit, file.old_path)
    new_bytes = git_show(new_commit, file.new_path)
    binary_summary = size_summary(old_bytes, new_bytes)

    try:
        old_text = old_bytes.decode("utf-8")
        new_text = new_bytes.decode("utf-8")
    except UnicodeDecodeError:
        return RenderedFile(
            definition="",
            section=render_template(
                BINARY_FILE_SECTION_TEMPLATE,
                heading=typst_file_heading(file.display_path, label=label),
                status=file.status,
                path=typst_raw(file.display_path),
                old_size=format_size(len(old_bytes)),
                new_size=format_size(len(new_bytes)),
                size_delta=format_delta(len(new_bytes) - len(old_bytes)),
            ),
            outline_case=render_template(
                OUTLINE_TEXT_CASE_TEMPLATE,
                label=label,
                summary="binary or non-UTF-8, " + binary_summary,
            ),
        )

    return RenderedFile(
        definition=render_template(
            TEXT_FILE_DEFINITION_TEMPLATE,
            report_name=report_name,
            old_text=typst_json_value(old_text),
            new_text=typst_json_value(new_text),
            old_label=typst_string(old_label),
            new_label=typst_string(new_label),
        ),
        section=render_template(
            TEXT_FILE_SECTION_TEMPLATE,
            heading=typst_file_heading(file.display_path, label=label),
            report_name=report_name,
        ),
        outline_case=render_template(
            OUTLINE_REPORT_CASE_TEMPLATE,
            label=label,
            report_name=report_name,
        ),
    )


def render_document(
    files: list[ChangedFile],
    *,
    old_commit: str,
    new_commit: str,
    old_short: str,
    new_short: str,
    import_path: str,
) -> str:
    rendered_files = [
        render_file(
            file,
            index=index,
            old_commit=old_commit,
            new_commit=new_commit,
            old_short=old_short,
            new_short=new_short,
        )
        for index, file in enumerate(files)
    ]

    definitions = "\n\n".join(
        file.definition for file in rendered_files if file.definition
    )
    outline_summary_cases = "\n".join(file.outline_case for file in rendered_files)
    body = (
        "\n\n".join(file.section for file in rendered_files)
        if rendered_files
        else render_template(NO_CHANGES_TEMPLATE)
    )
    return render_template(
        DOCUMENT_TEMPLATE,
        import_path=typst_string(import_path),
        definitions=definitions,
        outline_summary_cases=outline_summary_cases,
        old_short=old_short,
        new_short=new_short,
        body=body,
    )


def default_import_path(output: Path | None) -> str:
    if output is None:
        return "lib.typ"

    repo_root = Path(run_git(["rev-parse", "--show-toplevel"]).strip())
    lib_path = repo_root / "lib.typ"
    return os.path.relpath(lib_path, output.parent.resolve())


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate a Typst diff report for all files changed between two Git commits.",
    )
    parser.add_argument("old_commit", help="Old commit hash or revision.")
    parser.add_argument("new_commit", help="New commit hash or revision.")
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        help="Write the generated Typst document to this path.",
    )
    parser.add_argument(
        "--import-path",
        help="Typst import path for diffst. Defaults to lib.typ for stdout, or a path relative to --output.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    old_commit, old_short = resolve_commit(args.old_commit)
    new_commit, new_short = resolve_commit(args.new_commit)
    output = args.output
    import_path = args.import_path or default_import_path(output)

    document = render_document(
        changed_files(old_commit, new_commit),
        old_commit=old_commit,
        new_commit=new_commit,
        old_short=old_short,
        new_short=new_short,
        import_path=import_path,
    )

    if output is None:
        sys.stdout.write(document)
        return 0

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(document, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
