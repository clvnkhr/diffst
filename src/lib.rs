use serde::Serialize;
use similar::{capture_diff_slices, capture_diff_slices_by_key, Algorithm, DiffOp};

#[cfg(target_arch = "wasm32")]
wasm_minimal_protocol::initiate_protocol!();

#[derive(Serialize)]
struct DiffReport {
    stats: DiffStats,
    rows: Vec<DiffRow>,
}

#[derive(Serialize)]
struct DiffStats {
    old_lines: usize,
    new_lines: usize,
    additions: usize,
    deletions: usize,
    changed_blocks: usize,
}

#[derive(Serialize)]
struct DiffRow {
    kind: &'static str,
    old_no: Option<usize>,
    old: Option<String>,
    old_spans: Option<Vec<InlineSpan>>,
    new_no: Option<usize>,
    new: Option<String>,
    new_spans: Option<Vec<InlineSpan>>,
}

#[derive(Serialize)]
struct InlineSpan {
    kind: &'static str,
    text: String,
}

#[cfg_attr(target_arch = "wasm32", wasm_minimal_protocol::wasm_func)]
pub fn diff(old: &[u8], new: &[u8], options: &[u8]) -> Vec<u8> {
    match diff_impl(old, new, options) {
        Ok(bytes) => bytes,
        Err(message) => serde_json::json!({ "error": message }).to_string().into_bytes(),
    }
}

fn diff_impl(old: &[u8], new: &[u8], options: &[u8]) -> Result<Vec<u8>, String> {
    let old = std::str::from_utf8(old).map_err(|_| "old file is not valid UTF-8")?;
    let new = std::str::from_utf8(new).map_err(|_| "new file is not valid UTF-8")?;
    let options: serde_json::Value =
        serde_json::from_slice(options).unwrap_or_else(|_| serde_json::json!({}));
    let ignore_whitespace = options
        .get("ignore_whitespace")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let show_whitespace = options
        .get("show_whitespace")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let algorithm = options
        .get("algorithm")
        .and_then(|value| value.as_str())
        .map(parse_algorithm)
        .transpose()?
        .unwrap_or_default();

    let old_lines = split_lines(old);
    let new_lines = split_lines(new);
    let ops = capture_diff_slices_by_key(
        algorithm,
        &old_lines,
        &new_lines,
        |line| normalize_key(line, ignore_whitespace),
    );

    let mut report = DiffReport {
        stats: DiffStats {
            old_lines: old_lines.len(),
            new_lines: new_lines.len(),
            additions: 0,
            deletions: 0,
            changed_blocks: 0,
        },
        rows: Vec::new(),
    };

    for op in ops {
        match op {
            DiffOp::Equal {
                old_index,
                new_index,
                len,
            } => {
                for offset in 0..len {
                    report.rows.push(DiffRow {
                        kind: "equal",
                        old_no: Some(old_index + offset + 1),
                        old: Some(old_lines[old_index + offset].clone()),
                        old_spans: None,
                        new_no: Some(new_index + offset + 1),
                        new: Some(new_lines[new_index + offset].clone()),
                        new_spans: None,
                    });
                }
            }
            DiffOp::Delete { old_index, old_len, .. } => {
                report.stats.changed_blocks += 1;
                report.stats.deletions += old_len;
                for offset in 0..old_len {
                    report.rows.push(DiffRow {
                        kind: "delete",
                        old_no: Some(old_index + offset + 1),
                        old: Some(old_lines[old_index + offset].clone()),
                        old_spans: Some(vec![InlineSpan {
                            kind: "delete",
                            text: display_text(&old_lines[old_index + offset], show_whitespace),
                        }]),
                        new_no: None,
                        new: None,
                        new_spans: None,
                    });
                }
            }
            DiffOp::Insert {
                new_index, new_len, ..
            } => {
                report.stats.changed_blocks += 1;
                report.stats.additions += new_len;
                for offset in 0..new_len {
                    report.rows.push(DiffRow {
                        kind: "insert",
                        old_no: None,
                        old: None,
                        old_spans: None,
                        new_no: Some(new_index + offset + 1),
                        new: Some(new_lines[new_index + offset].clone()),
                        new_spans: Some(vec![InlineSpan {
                            kind: "insert",
                            text: display_text(&new_lines[new_index + offset], show_whitespace),
                        }]),
                    });
                }
            }
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                report.stats.changed_blocks += 1;
                report.stats.deletions += old_len;
                report.stats.additions += new_len;
                let len = old_len.max(new_len);
                for offset in 0..len {
                    let old_line = (offset < old_len).then(|| old_lines[old_index + offset].as_str());
                    let new_line = (offset < new_len).then(|| new_lines[new_index + offset].as_str());
                    let (old_spans, new_spans) = match (old_line, new_line) {
                        (Some(old_line), Some(new_line)) => {
                            inline_spans(old_line, new_line, show_whitespace, algorithm)
                        }
                        (Some(old_line), None) => (
                            Some(vec![InlineSpan {
                                kind: "delete",
                                text: display_text(old_line, show_whitespace),
                            }]),
                            None,
                        ),
                        (None, Some(new_line)) => (
                            None,
                            Some(vec![InlineSpan {
                                kind: "insert",
                                text: display_text(new_line, show_whitespace),
                            }]),
                        ),
                        (None, None) => (None, None),
                    };

                    report.rows.push(DiffRow {
                        kind: "replace",
                        old_no: (offset < old_len).then_some(old_index + offset + 1),
                        old: (offset < old_len).then(|| old_lines[old_index + offset].clone()),
                        old_spans,
                        new_no: (offset < new_len).then_some(new_index + offset + 1),
                        new: (offset < new_len).then(|| new_lines[new_index + offset].clone()),
                        new_spans,
                    });
                }
            }
        }
    }

    serde_json::to_vec(&report).map_err(|err| err.to_string())
}

fn inline_spans(
    old_line: &str,
    new_line: &str,
    show_whitespace: bool,
    algorithm: Algorithm,
) -> (Option<Vec<InlineSpan>>, Option<Vec<InlineSpan>>) {
    let old_chars = old_line.chars().collect::<Vec<_>>();
    let new_chars = new_line.chars().collect::<Vec<_>>();
    let ops = capture_diff_slices(algorithm, &old_chars, &new_chars);
    let mut old_spans = Vec::new();
    let mut new_spans = Vec::new();

    for op in ops {
        match op {
            DiffOp::Equal {
                old_index,
                new_index,
                len,
            } => {
                push_span(
                    &mut old_spans,
                    "equal",
                    old_chars[old_index..old_index + len].iter(),
                    false,
                );
                push_span(
                    &mut new_spans,
                    "equal",
                    new_chars[new_index..new_index + len].iter(),
                    false,
                );
            }
            DiffOp::Delete { old_index, old_len, .. } => {
                push_span(
                    &mut old_spans,
                    "delete",
                    old_chars[old_index..old_index + old_len].iter(),
                    show_whitespace,
                );
            }
            DiffOp::Insert {
                new_index, new_len, ..
            } => {
                push_span(
                    &mut new_spans,
                    "insert",
                    new_chars[new_index..new_index + new_len].iter(),
                    show_whitespace,
                );
            }
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                push_span(
                    &mut old_spans,
                    "delete",
                    old_chars[old_index..old_index + old_len].iter(),
                    show_whitespace,
                );
                push_span(
                    &mut new_spans,
                    "insert",
                    new_chars[new_index..new_index + new_len].iter(),
                    show_whitespace,
                );
            }
        }
    }

    (Some(old_spans), Some(new_spans))
}

fn parse_algorithm(value: &str) -> Result<Algorithm, String> {
    match value {
        "myers" => Ok(Algorithm::Myers),
        "patience" => Ok(Algorithm::Patience),
        "lcs" => Ok(Algorithm::Lcs),
        "hunt" => Ok(Algorithm::Hunt),
        "histogram" => Ok(Algorithm::Histogram),
        _ => Err(format!(
            "algorithm must be one of: myers, patience, lcs, hunt, histogram; got {value:?}"
        )),
    }
}

fn push_span<'a>(
    spans: &mut Vec<InlineSpan>,
    kind: &'static str,
    chars: impl Iterator<Item = &'a char>,
    show_whitespace: bool,
) {
    let text = chars
        .map(|ch| display_char(*ch, show_whitespace))
        .collect::<String>();

    if text.is_empty() {
        return;
    }

    if let Some(last) = spans.last_mut() {
        if last.kind == kind {
            last.text.push_str(&text);
            return;
        }
    }

    spans.push(InlineSpan { kind, text });
}

fn display_text(text: &str, show_whitespace: bool) -> String {
    text.chars()
        .map(|ch| display_char(ch, show_whitespace))
        .collect()
}

fn display_char(ch: char, show_whitespace: bool) -> char {
    if !show_whitespace {
        return ch;
    }

    match ch {
        ' ' => '·',
        '\t' => '→',
        _ => ch,
    }
}

fn split_lines(text: &str) -> Vec<String> {
    let mut lines = text
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line).to_owned())
        .collect::<Vec<_>>();

    if text.ends_with('\n') {
        lines.pop();
    }

    lines
}

fn normalize_key(line: &str, ignore_whitespace: bool) -> String {
    if ignore_whitespace {
        line.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        line.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_basic_line_changes() {
        let output = diff_impl(b"a\nb\nc\n", b"a\nbee\nc\nd\n", br#"{}"#).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();

        assert_eq!(value["stats"]["deletions"], 1);
        assert_eq!(value["stats"]["additions"], 2);
        assert_eq!(value["rows"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn can_ignore_whitespace() {
        let output = diff_impl(
            b"let x = 1\n",
            b"let   x = 1\n",
            br#"{"ignore_whitespace":true}"#,
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();

        assert_eq!(value["stats"]["deletions"], 0);
        assert_eq!(value["stats"]["additions"], 0);
    }

    #[test]
    fn reports_inline_spans_for_replacements() {
        let output = diff_impl(b"hello world\n", b"hello typst\n", br#"{}"#).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
        let row = &value["rows"][0];

        assert_eq!(row["kind"], "replace");
        assert!(row["old_spans"]
            .as_array()
            .unwrap()
            .iter()
            .any(|span| span["kind"] == "delete"));
        assert!(row["new_spans"]
            .as_array()
            .unwrap()
            .iter()
            .any(|span| span["kind"] == "insert"));
    }

    #[test]
    fn can_make_changed_whitespace_visible() {
        let output = diff_impl(
            b"let x = 1\n",
            b"let x  = 1\n",
            br#"{"show_whitespace":true}"#,
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();

        assert!(value["rows"][0]["new_spans"]
            .as_array()
            .unwrap()
            .iter()
            .any(|span| span["text"].as_str().unwrap().contains('·')));
    }

    #[test]
    fn accepts_all_supported_algorithms() {
        for algorithm in ["myers", "patience", "lcs", "hunt", "histogram"] {
            let options = format!(r#"{{"algorithm":"{algorithm}"}}"#);
            let output = diff_impl(b"a\nb\nc\n", b"a\nbee\nc\n", options.as_bytes()).unwrap();
            let value: serde_json::Value = serde_json::from_slice(&output).unwrap();

            assert_eq!(value["stats"]["deletions"], 1);
            assert_eq!(value["stats"]["additions"], 1);
        }
    }

    #[test]
    fn rejects_unknown_algorithm() {
        let err = diff_impl(b"a\n", b"b\n", br#"{"algorithm":"nope"}"#).unwrap_err();

        assert!(err.contains("algorithm must be one of"));
    }
}
