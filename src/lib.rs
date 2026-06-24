use serde::Serialize;
use similar::algorithms::{diff_slices, Capture, Compact, Replace};
use similar::{capture_diff_slices, capture_diff_slices_by_key, Algorithm, DiffOp};

#[cfg(target_arch = "wasm32")]
wasm_minimal_protocol::initiate_protocol!();

#[derive(Serialize)]
struct DiffReport {
    stats: DiffStats,
    ops: Vec<ReportOp>,
    rows: Vec<DiffRow>,
}

#[derive(Serialize)]
struct DiffStats {
    old_lines: usize,
    new_lines: usize,
    additions: usize,
    deletions: usize,
    changed_blocks: usize,
    equal_lines: usize,
    similarity: f64,
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

#[derive(Serialize)]
struct ReportOp {
    kind: &'static str,
    old_start: usize,
    old_len: usize,
    new_start: usize,
    new_len: usize,
    row_start: usize,
    row_len: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum InlineMode {
    Chars,
    Words,
    None,
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
    let inline = options
        .get("inline")
        .and_then(|value| value.as_str())
        .map(parse_inline_mode)
        .transpose()?
        .unwrap_or(InlineMode::Chars);
    let semantic_cleanup = options
        .get("semantic_cleanup")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);

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
            equal_lines: 0,
            similarity: 1.0,
        },
        ops: Vec::new(),
        rows: Vec::new(),
    };

    for op in ops {
        let row_start = report.rows.len();
        match op {
            DiffOp::Equal {
                old_index,
                new_index,
                len,
            } => {
                report.stats.equal_lines += len;
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
                report.ops.push(ReportOp {
                    kind: "equal",
                    old_start: old_index + 1,
                    old_len: len,
                    new_start: new_index + 1,
                    new_len: len,
                    row_start,
                    row_len: report.rows.len() - row_start,
                });
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
                report.ops.push(ReportOp {
                    kind: "delete",
                    old_start: old_index + 1,
                    old_len,
                    new_start: 0,
                    new_len: 0,
                    row_start,
                    row_len: report.rows.len() - row_start,
                });
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
                report.ops.push(ReportOp {
                    kind: "insert",
                    old_start: 0,
                    old_len: 0,
                    new_start: new_index + 1,
                    new_len,
                    row_start,
                    row_len: report.rows.len() - row_start,
                });
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
                            inline_spans(
                                old_line,
                                new_line,
                                show_whitespace,
                                algorithm,
                                inline,
                                semantic_cleanup,
                            )
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
                report.ops.push(ReportOp {
                    kind: "replace",
                    old_start: old_index + 1,
                    old_len,
                    new_start: new_index + 1,
                    new_len,
                    row_start,
                    row_len: report.rows.len() - row_start,
                });
            }
        }
    }

    report.stats.similarity = similarity_score(
        report.stats.equal_lines,
        report.stats.old_lines,
        report.stats.new_lines,
    );

    serde_json::to_vec(&report).map_err(|err| err.to_string())
}

fn similarity_score(equal_lines: usize, old_lines: usize, new_lines: usize) -> f64 {
    let total = old_lines + new_lines;
    if total == 0 {
        1.0
    } else {
        (2.0 * equal_lines as f64) / total as f64
    }
}

fn inline_spans(
    old_line: &str,
    new_line: &str,
    show_whitespace: bool,
    algorithm: Algorithm,
    inline: InlineMode,
    semantic_cleanup: bool,
) -> (Option<Vec<InlineSpan>>, Option<Vec<InlineSpan>>) {
    if inline == InlineMode::None {
        return (None, None);
    }

    let old_tokens = inline_tokens(old_line, inline);
    let new_tokens = inline_tokens(new_line, inline);
    let ops = if semantic_cleanup {
        capture_compact_diff_slices(algorithm, &old_tokens, &new_tokens)
    } else {
        capture_diff_slices(algorithm, &old_tokens, &new_tokens)
    };
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
                    old_tokens[old_index..old_index + len].iter(),
                    false,
                );
                push_span(
                    &mut new_spans,
                    "equal",
                    new_tokens[new_index..new_index + len].iter(),
                    false,
                );
            }
            DiffOp::Delete { old_index, old_len, .. } => {
                push_span(
                    &mut old_spans,
                    "delete",
                    old_tokens[old_index..old_index + old_len].iter(),
                    show_whitespace,
                );
            }
            DiffOp::Insert {
                new_index, new_len, ..
            } => {
                push_span(
                    &mut new_spans,
                    "insert",
                    new_tokens[new_index..new_index + new_len].iter(),
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
                    old_tokens[old_index..old_index + old_len].iter(),
                    show_whitespace,
                );
                push_span(
                    &mut new_spans,
                    "insert",
                    new_tokens[new_index..new_index + new_len].iter(),
                    show_whitespace,
                );
            }
        }
    }

    (Some(old_spans), Some(new_spans))
}

fn capture_compact_diff_slices<T>(algorithm: Algorithm, old: &[T], new: &[T]) -> Vec<DiffOp>
where
    T: Eq + std::hash::Hash,
{
    let capture = Capture::new();
    let replace = Replace::new(capture);
    let mut compact = Compact::new(replace, old, new);
    diff_slices(algorithm, &mut compact, old, new).unwrap();
    compact.into_inner().into_inner().into_ops()
}

fn inline_tokens(line: &str, inline: InlineMode) -> Vec<String> {
    match inline {
        InlineMode::Chars => line.chars().map(|ch| ch.to_string()).collect(),
        InlineMode::Words => word_tokens(line),
        InlineMode::None => Vec::new(),
    }
}

fn word_tokens(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut current_kind = None;

    for ch in line.chars() {
        let kind = token_kind(ch);
        if current_kind.is_some_and(|active| active != kind) {
            tokens.push(current);
            current = String::new();
        }

        current.push(ch);
        current_kind = Some(kind);
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn token_kind(ch: char) -> u8 {
    if ch.is_whitespace() {
        0
    } else if ch.is_alphanumeric() || ch == '_' {
        1
    } else {
        2
    }
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

fn parse_inline_mode(value: &str) -> Result<InlineMode, String> {
    match value {
        "chars" => Ok(InlineMode::Chars),
        "words" => Ok(InlineMode::Words),
        "none" => Ok(InlineMode::None),
        _ => Err(format!(
            "inline must be one of: chars, words, none; got {value:?}"
        )),
    }
}

fn push_span<'a>(
    spans: &mut Vec<InlineSpan>,
    kind: &'static str,
    tokens: impl Iterator<Item = &'a String>,
    show_whitespace: bool,
) {
    let text = tokens
        .flat_map(|token| token.chars())
        .map(|ch| display_char(ch, show_whitespace))
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
        assert_eq!(value["stats"]["equal_lines"], 2);
        assert_eq!(value["stats"]["similarity"], 4.0 / 7.0);
        assert_eq!(value["rows"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn reports_full_similarity_for_empty_files() {
        let output = diff_impl(b"", b"", br#"{}"#).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();

        assert_eq!(value["stats"]["similarity"], 1.0);
    }

    #[test]
    fn reports_line_ops_with_row_ranges() {
        let output = diff_impl(b"a\nb\nc\n", b"a\nbee\nc\nd\n", br#"{}"#).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
        let ops = value["ops"].as_array().unwrap();

        assert_eq!(ops[0]["kind"], "equal");
        assert_eq!(ops[1]["kind"], "replace");
        assert_eq!(ops[2]["kind"], "equal");
        assert_eq!(ops[3]["kind"], "insert");
        assert_eq!(ops[1]["row_start"], 1);
        assert_eq!(ops[1]["row_len"], 1);
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
    fn can_highlight_inline_words() {
        let output = diff_impl(
            b"The quick brown fox\n",
            b"The quick amber fox\n",
            br#"{"inline":"words"}"#,
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();

        assert!(value["rows"][0]["old_spans"]
            .as_array()
            .unwrap()
            .iter()
            .any(|span| span["kind"] == "delete" && span["text"] == "brown"));
        assert!(value["rows"][0]["new_spans"]
            .as_array()
            .unwrap()
            .iter()
            .any(|span| span["kind"] == "insert" && span["text"] == "amber"));
    }

    #[test]
    fn can_disable_inline_spans_for_replacements() {
        let output = diff_impl(
            b"hello world\n",
            b"hello typst\n",
            br#"{"inline":"none"}"#,
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();

        assert_eq!(value["rows"][0]["kind"], "replace");
        assert!(value["rows"][0]["old_spans"].is_null());
        assert!(value["rows"][0]["new_spans"].is_null());
    }

    #[test]
    fn accepts_semantic_cleanup_for_inline_spans() {
        let output = diff_impl(
            b"let value = compute(old_input)\n",
            b"let value = compute(new_input)\n",
            br#"{"inline":"words","semantic_cleanup":true}"#,
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();

        assert_eq!(value["rows"][0]["kind"], "replace");
        assert!(value["rows"][0]["old_spans"].is_array());
        assert!(value["rows"][0]["new_spans"].is_array());
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

    #[test]
    fn rejects_unknown_inline_mode() {
        let err = diff_impl(b"a\n", b"b\n", br#"{"inline":"letters"}"#).unwrap_err();

        assert!(err.contains("inline must be one of"));
    }
}
