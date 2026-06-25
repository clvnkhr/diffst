use serde::{Deserialize, Serialize};
use similar::algorithms::{diff_slices, Capture, Compact, Replace};
use similar::{capture_diff_slices, Algorithm, DiffOp, DiffableStr};

#[cfg(target_arch = "wasm32")]
wasm_minimal_protocol::initiate_protocol!();

#[derive(Serialize)]
struct DiffReport<'a> {
    meta: DiffMeta,
    stats: DiffStats,
    ops: Vec<ReportOp>,
    rows: Vec<DiffRow<'a>>,
}

#[derive(Serialize)]
struct DiffMeta {
    algorithm: &'static str,
    inline: &'static str,
    ignore_whitespace: bool,
    show_whitespace: bool,
    unicode: bool,
    semantic_cleanup: bool,
    old_trailing_newline: bool,
    new_trailing_newline: bool,
    old_line_endings: &'static str,
    new_line_endings: &'static str,
    messages: Vec<String>,
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
struct DiffRow<'a> {
    kind: &'static str,
    old_no: Option<usize>,
    old: Option<&'a str>,
    old_spans: Option<Vec<InlineSpan>>,
    new_no: Option<usize>,
    new: Option<&'a str>,
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawOptions {
    #[serde(default)]
    ignore_whitespace: bool,
    #[serde(default)]
    show_whitespace: bool,
    #[serde(default = "default_algorithm")]
    algorithm: String,
    #[serde(default = "default_inline")]
    inline: String,
    #[serde(default = "default_unicode")]
    unicode: bool,
    #[serde(default)]
    semantic_cleanup: bool,
}

struct Options {
    ignore_whitespace: bool,
    show_whitespace: bool,
    algorithm: Algorithm,
    inline: InlineMode,
    unicode: bool,
    semantic_cleanup: bool,
}

impl Options {
    fn from_json(bytes: &[u8]) -> Result<Self, String> {
        let raw: RawOptions =
            serde_json::from_slice(bytes).map_err(|err| format!("invalid options JSON: {err}"))?;

        Ok(Self {
            ignore_whitespace: raw.ignore_whitespace,
            show_whitespace: raw.show_whitespace,
            algorithm: parse_algorithm(&raw.algorithm)?,
            inline: parse_inline_mode(&raw.inline)?,
            unicode: raw.unicode,
            semantic_cleanup: raw.semantic_cleanup,
        })
    }
}

fn default_algorithm() -> String {
    "myers".to_owned()
}

fn default_inline() -> String {
    "chars".to_owned()
}

fn default_unicode() -> bool {
    true
}

struct LineInput<'a> {
    text: &'a str,
    lines: Vec<String>,
    trailing_newline: bool,
    line_endings: &'static str,
}

impl<'a> LineInput<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text,
            lines: split_lines(text),
            trailing_newline: text.ends_with('\n'),
            line_endings: line_ending_style(text),
        }
    }
}

struct ReportBuilder<'a> {
    report: DiffReport<'a>,
    old_lines: &'a [String],
    new_lines: &'a [String],
    options: &'a Options,
}

impl<'a> ReportBuilder<'a> {
    fn new(old: &'a LineInput<'a>, new: &'a LineInput<'a>, options: &'a Options) -> Self {
        Self {
            report: DiffReport {
                meta: build_meta(old, new, options),
                stats: DiffStats {
                    old_lines: old.lines.len(),
                    new_lines: new.lines.len(),
                    additions: 0,
                    deletions: 0,
                    changed_blocks: 0,
                    equal_lines: 0,
                    similarity: 1.0,
                },
                ops: Vec::new(),
                rows: Vec::new(),
            },
            old_lines: &old.lines,
            new_lines: &new.lines,
            options,
        }
    }

    fn push_op(
        &mut self,
        kind: &'static str,
        old_start: usize,
        old_len: usize,
        new_start: usize,
        new_len: usize,
        row_start: usize,
    ) {
        self.report.ops.push(ReportOp {
            kind,
            old_start,
            old_len,
            new_start,
            new_len,
            row_start,
            row_len: self.report.rows.len() - row_start,
        });
    }

    fn push_equal(&mut self, old_index: usize, new_index: usize, len: usize) {
        let row_start = self.report.rows.len();
        self.report.stats.equal_lines += len;

        for offset in 0..len {
            let old_line = &self.old_lines[old_index + offset];
            let new_line = &self.new_lines[new_index + offset];
            self.report.rows.push(DiffRow {
                kind: "equal",
                old_no: Some(old_index + offset + 1),
                old: Some(old_line.as_str()),
                old_spans: equal_spans(old_line, self.options.show_whitespace),
                new_no: Some(new_index + offset + 1),
                new: Some(new_line.as_str()),
                new_spans: equal_spans(new_line, self.options.show_whitespace),
            });
        }

        self.push_op("equal", old_index + 1, len, new_index + 1, len, row_start);
    }

    fn push_delete(&mut self, old_index: usize, old_len: usize) {
        let row_start = self.report.rows.len();
        self.report.stats.changed_blocks += 1;
        self.report.stats.deletions += old_len;

        for offset in 0..old_len {
            let old_line = &self.old_lines[old_index + offset];
            self.report.rows.push(DiffRow {
                kind: "delete",
                old_no: Some(old_index + offset + 1),
                old: Some(old_line.as_str()),
                old_spans: Some(deleted_spans(old_line, self.options.show_whitespace)),
                new_no: None,
                new: None,
                new_spans: None,
            });
        }

        self.push_op("delete", old_index + 1, old_len, 0, 0, row_start);
    }

    fn push_insert(&mut self, new_index: usize, new_len: usize) {
        let row_start = self.report.rows.len();
        self.report.stats.changed_blocks += 1;
        self.report.stats.additions += new_len;

        for offset in 0..new_len {
            let new_line = &self.new_lines[new_index + offset];
            self.report.rows.push(DiffRow {
                kind: "insert",
                old_no: None,
                old: None,
                old_spans: None,
                new_no: Some(new_index + offset + 1),
                new: Some(new_line.as_str()),
                new_spans: Some(inserted_spans(new_line, self.options.show_whitespace)),
            });
        }

        self.push_op("insert", 0, 0, new_index + 1, new_len, row_start);
    }

    fn push_replace(
        &mut self,
        old_index: usize,
        old_len: usize,
        new_index: usize,
        new_len: usize,
    ) -> Result<(), String> {
        let row_start = self.report.rows.len();
        self.report.stats.changed_blocks += 1;
        self.report.stats.deletions += old_len;
        self.report.stats.additions += new_len;

        for offset in 0..old_len.max(new_len) {
            let old_line = (offset < old_len).then(|| self.old_lines[old_index + offset].as_str());
            let new_line = (offset < new_len).then(|| self.new_lines[new_index + offset].as_str());
            let (old_spans, new_spans) = replace_spans(old_line, new_line, self.options)?;

            self.report.rows.push(DiffRow {
                kind: "replace",
                old_no: (offset < old_len).then_some(old_index + offset + 1),
                old: (offset < old_len).then(|| self.old_lines[old_index + offset].as_str()),
                old_spans,
                new_no: (offset < new_len).then_some(new_index + offset + 1),
                new: (offset < new_len).then(|| self.new_lines[new_index + offset].as_str()),
                new_spans,
            });
        }

        self.push_op(
            "replace",
            old_index + 1,
            old_len,
            new_index + 1,
            new_len,
            row_start,
        );
        Ok(())
    }

    fn finish(mut self) -> DiffReport<'a> {
        self.report.stats.similarity = similarity_score(
            self.report.stats.equal_lines,
            self.report.stats.old_lines,
            self.report.stats.new_lines,
        );
        self.report
    }
}

impl InlineMode {
    fn name(self) -> &'static str {
        match self {
            InlineMode::Chars => "chars",
            InlineMode::Words => "words",
            InlineMode::None => "none",
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_minimal_protocol::wasm_func)]
pub fn diff(old: &[u8], new: &[u8], options: &[u8]) -> Vec<u8> {
    match diff_impl(old, new, options) {
        Ok(bytes) => bytes,
        Err(message) => serde_json::json!({ "error": message })
            .to_string()
            .into_bytes(),
    }
}

fn diff_impl(old: &[u8], new: &[u8], options: &[u8]) -> Result<Vec<u8>, String> {
    let old = std::str::from_utf8(old).map_err(|_| "old file is not valid UTF-8")?;
    let new = std::str::from_utf8(new).map_err(|_| "new file is not valid UTF-8")?;
    let options = Options::from_json(options)?;
    let old = LineInput::new(old);
    let new = LineInput::new(new);
    let ops = line_ops(
        options.algorithm,
        options.ignore_whitespace,
        &old.lines,
        &new.lines,
    );
    let mut builder = ReportBuilder::new(&old, &new, &options);

    for op in ops {
        match op {
            DiffOp::Equal {
                old_index,
                new_index,
                len,
            } => builder.push_equal(old_index, new_index, len),
            DiffOp::Delete {
                old_index, old_len, ..
            } => {
                builder.push_delete(old_index, old_len);
            }
            DiffOp::Insert {
                new_index, new_len, ..
            } => {
                builder.push_insert(new_index, new_len);
            }
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => builder.push_replace(old_index, old_len, new_index, new_len)?,
        }
    }

    serde_json::to_vec(&builder.finish()).map_err(|err| err.to_string())
}

fn build_meta(old: &LineInput<'_>, new: &LineInput<'_>, options: &Options) -> DiffMeta {
    let mut messages = Vec::new();
    messages.push(format!("algorithm: {}", algorithm_name(options.algorithm)));
    messages.push(format!("inline: {}", options.inline.name()));

    if options.unicode {
        messages.push("unicode inline tokenization enabled".to_owned());
    } else {
        messages.push("unicode inline tokenization disabled".to_owned());
    }

    if options.ignore_whitespace {
        messages.push("line matching ignores whitespace".to_owned());
    }

    if options.show_whitespace {
        messages.push("changed whitespace is rendered visibly".to_owned());
    }

    if options.semantic_cleanup {
        messages.push("semantic cleanup enabled for inline spans".to_owned());
    }

    if old.text != new.text && old.lines == new.lines {
        messages.push("files differ only by trailing newline".to_owned());
    }

    if old.line_endings != new.line_endings {
        messages.push(format!(
            "line endings differ: {} -> {}",
            old.line_endings, new.line_endings
        ));
    }

    DiffMeta {
        algorithm: algorithm_name(options.algorithm),
        inline: options.inline.name(),
        ignore_whitespace: options.ignore_whitespace,
        show_whitespace: options.show_whitespace,
        unicode: options.unicode,
        semantic_cleanup: options.semantic_cleanup,
        old_trailing_newline: old.trailing_newline,
        new_trailing_newline: new.trailing_newline,
        old_line_endings: old.line_endings,
        new_line_endings: new.line_endings,
        messages,
    }
}

fn line_ops(
    algorithm: Algorithm,
    ignore_whitespace: bool,
    old_lines: &[String],
    new_lines: &[String],
) -> Vec<DiffOp> {
    if ignore_whitespace {
        let old_keys = old_lines
            .iter()
            .map(|line| normalize_key(line))
            .collect::<Vec<_>>();
        let new_keys = new_lines
            .iter()
            .map(|line| normalize_key(line))
            .collect::<Vec<_>>();
        capture_diff_slices(algorithm, &old_keys, &new_keys)
    } else {
        capture_diff_slices(algorithm, old_lines, new_lines)
    }
}

fn similarity_score(equal_lines: usize, old_lines: usize, new_lines: usize) -> f64 {
    let total = old_lines + new_lines;
    if total == 0 {
        1.0
    } else {
        (2.0 * equal_lines as f64) / total as f64
    }
}

fn replace_spans(
    old_line: Option<&str>,
    new_line: Option<&str>,
    options: &Options,
) -> Result<(Option<Vec<InlineSpan>>, Option<Vec<InlineSpan>>), String> {
    match (old_line, new_line) {
        (Some(old_line), Some(new_line)) => inline_spans(
            old_line,
            new_line,
            options.show_whitespace,
            options.algorithm,
            options.inline,
            options.unicode,
            options.semantic_cleanup,
        ),
        (Some(old_line), None) => Ok((
            Some(deleted_spans(old_line, options.show_whitespace)),
            None,
        )),
        (None, Some(new_line)) => Ok((
            None,
            Some(inserted_spans(new_line, options.show_whitespace)),
        )),
        (None, None) => Ok((None, None)),
    }
}

fn deleted_spans(text: &str, show_whitespace: bool) -> Vec<InlineSpan> {
    display_spans(text, "delete", show_whitespace)
}

fn inserted_spans(text: &str, show_whitespace: bool) -> Vec<InlineSpan> {
    display_spans(text, "insert", show_whitespace)
}

fn equal_spans(text: &str, show_whitespace: bool) -> Option<Vec<InlineSpan>> {
    if !show_whitespace || !has_trailing_horizontal_whitespace(text) {
        return None;
    }

    Some(display_trailing_whitespace_spans(text))
}

fn inline_spans(
    old_line: &str,
    new_line: &str,
    show_whitespace: bool,
    algorithm: Algorithm,
    inline: InlineMode,
    unicode: bool,
    semantic_cleanup: bool,
) -> Result<(Option<Vec<InlineSpan>>, Option<Vec<InlineSpan>>), String> {
    if inline == InlineMode::None {
        return Ok((None, None));
    }

    let old_tokens = inline_tokens(old_line, inline, unicode);
    let new_tokens = inline_tokens(new_line, inline, unicode);
    let ops = if semantic_cleanup {
        capture_compact_diff_slices(algorithm, &old_tokens, &new_tokens)?
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
            DiffOp::Delete {
                old_index, old_len, ..
            } => {
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

    Ok((Some(old_spans), Some(new_spans)))
}

fn capture_compact_diff_slices<T>(
    algorithm: Algorithm,
    old: &[T],
    new: &[T],
) -> Result<Vec<DiffOp>, String>
where
    T: Eq + std::hash::Hash,
{
    let capture = Capture::new();
    let replace = Replace::new(capture);
    let mut compact = Compact::new(replace, old, new);
    diff_slices(algorithm, &mut compact, old, new)
        .map_err(|_| "inline semantic cleanup failed".to_owned())?;
    Ok(compact.into_inner().into_inner().into_ops())
}

fn inline_tokens(line: &str, inline: InlineMode, unicode: bool) -> Vec<String> {
    match inline {
        InlineMode::Chars if unicode => line
            .tokenize_graphemes()
            .into_iter()
            .map(ToOwned::to_owned)
            .collect(),
        InlineMode::Chars => line.chars().map(|ch| ch.to_string()).collect(),
        InlineMode::Words if unicode => line
            .tokenize_unicode_words()
            .into_iter()
            .map(ToOwned::to_owned)
            .collect(),
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

fn algorithm_name(algorithm: Algorithm) -> &'static str {
    match algorithm {
        Algorithm::Myers => "myers",
        Algorithm::Patience => "patience",
        Algorithm::Lcs => "lcs",
        Algorithm::Hunt => "hunt",
        Algorithm::Histogram => "histogram",
        _ => "unknown",
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
    for ch in tokens.flat_map(|token| token.chars()) {
        push_display_char(spans, kind, ch, show_whitespace);
    }
}

fn push_display_char(
    spans: &mut Vec<InlineSpan>,
    kind: &'static str,
    ch: char,
    show_whitespace: bool,
) {
    let displayed = display_char(ch, show_whitespace);
    let kind = if show_whitespace && is_whitespace_marker(displayed) {
        marker_kind(kind)
    } else {
        kind
    };

    push_text(spans, kind, displayed);
}

fn push_text(spans: &mut Vec<InlineSpan>, kind: &'static str, ch: char) {
    if let Some(last) = spans.last_mut() {
        if last.kind == kind {
            last.text.push(ch);
            return;
        }
    }

    spans.push(InlineSpan {
        kind,
        text: ch.to_string(),
    });
}

fn display_spans(text: &str, kind: &'static str, show_whitespace: bool) -> Vec<InlineSpan> {
    let mut spans = Vec::new();

    for ch in text.chars() {
        push_display_char(&mut spans, kind, ch, show_whitespace);
    }

    spans
}

fn display_trailing_whitespace_spans(text: &str) -> Vec<InlineSpan> {
    let start = trailing_horizontal_whitespace_start(text);
    let mut spans = Vec::new();

    for ch in text[..start].chars() {
        push_display_char(&mut spans, "equal", ch, false);
    }
    for ch in text[start..].chars() {
        push_display_char(&mut spans, "equal", ch, true);
    }

    spans
}

fn has_trailing_horizontal_whitespace(text: &str) -> bool {
    trailing_horizontal_whitespace_start(text) < text.len()
}

fn trailing_horizontal_whitespace_start(text: &str) -> usize {
    let mut start = text.len();

    for (index, ch) in text.char_indices().rev() {
        if ch == ' ' || ch == '\t' {
            start = index;
        } else {
            break;
        }
    }

    start
}

fn display_char(ch: char, show_whitespace: bool) -> char {
    if !show_whitespace {
        return ch;
    }

    match ch {
        ' ' => '·',
        '\t' => '→',
        '\n' => '↵',
        '\r' => '␍',
        _ => ch,
    }
}

fn is_whitespace_marker(ch: char) -> bool {
    matches!(ch, '·' | '→' | '↵' | '␍')
}

fn marker_kind(kind: &'static str) -> &'static str {
    match kind {
        "delete" => "delete-marker",
        "insert" => "insert-marker",
        _ => "equal-marker",
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

fn line_ending_style(text: &str) -> &'static str {
    let bytes = text.as_bytes();
    let mut index = 0;
    let mut lf = 0;
    let mut crlf = 0;
    let mut cr = 0;

    while index < bytes.len() {
        if bytes[index] == b'\r' {
            if bytes.get(index + 1) == Some(&b'\n') {
                crlf += 1;
                index += 2;
            } else {
                cr += 1;
                index += 1;
            }
        } else if bytes[index] == b'\n' {
            lf += 1;
            index += 1;
        } else {
            index += 1;
        }
    }

    match (lf > 0, crlf > 0, cr > 0) {
        (false, false, false) => "none",
        (true, false, false) => "lf",
        (false, true, false) => "crlf",
        (false, false, true) => "cr",
        _ => "mixed",
    }
}

fn normalize_key(line: &str) -> String {
    line.split_whitespace().collect::<Vec<_>>().join(" ")
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
            .any(|span| span["kind"] == "insert-marker"
                && span["text"].as_str().unwrap().contains('·')));
    }

    #[test]
    fn can_mark_trailing_whitespace_on_equal_lines() {
        let output = diff_impl(
            b"let x = 1  \n\ttrimmed\t\n",
            b"let x = 1  \n\ttrimmed\t\n",
            br#"{"show_whitespace":true}"#,
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
        let rows = value["rows"].as_array().unwrap();

        assert_eq!(rows[0]["old_spans"][0]["text"], "let x = 1");
        assert_eq!(rows[0]["old_spans"][1]["kind"], "equal-marker");
        assert_eq!(rows[0]["old_spans"][1]["text"], "··");
        assert_eq!(rows[1]["old_spans"][0]["text"], "\ttrimmed");
        assert_eq!(rows[1]["old_spans"][1]["kind"], "equal-marker");
        assert_eq!(rows[1]["old_spans"][1]["text"], "→");
        assert_eq!(rows[0]["new_spans"][1]["kind"], "equal-marker");
        assert_eq!(rows[0]["new_spans"][1]["text"], "··");
        assert_eq!(rows[1]["new_spans"][1]["kind"], "equal-marker");
        assert_eq!(rows[1]["new_spans"][1]["text"], "→");
    }

    #[test]
    fn can_mark_newline_chars_when_displaying_whitespace() {
        let spans = display_spans("a\nb\r\n", "equal", true);

        assert_eq!(spans[0].kind, "equal");
        assert_eq!(spans[0].text, "a");
        assert_eq!(spans[1].kind, "equal-marker");
        assert_eq!(spans[1].text, "↵");
        assert_eq!(spans[2].kind, "equal");
        assert_eq!(spans[2].text, "b");
        assert_eq!(spans[3].kind, "equal-marker");
        assert_eq!(spans[3].text, "␍↵");
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
    fn defaults_to_unicode_graphemes_for_inline_chars() {
        let output = diff_impl(
            "a🇦🇹b\n".as_bytes(),
            "a🇦🇱b\n".as_bytes(),
            br#"{"inline":"chars"}"#,
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();

        assert!(value["rows"][0]["old_spans"]
            .as_array()
            .unwrap()
            .iter()
            .any(|span| span["kind"] == "delete" && span["text"] == "🇦🇹"));
        assert!(value["rows"][0]["new_spans"]
            .as_array()
            .unwrap()
            .iter()
            .any(|span| span["kind"] == "insert" && span["text"] == "🇦🇱"));
    }

    #[test]
    fn can_disable_unicode_inline_tokenization() {
        let output = diff_impl(
            "a🇦🇹b\n".as_bytes(),
            "a🇦🇱b\n".as_bytes(),
            br#"{"inline":"chars","unicode":false}"#,
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();

        assert!(value["rows"][0]["old_spans"]
            .as_array()
            .unwrap()
            .iter()
            .any(|span| span["kind"] == "delete" && span["text"] == "🇹"));
        assert!(value["rows"][0]["new_spans"]
            .as_array()
            .unwrap()
            .iter()
            .any(|span| span["kind"] == "insert" && span["text"] == "🇱"));
    }

    #[test]
    fn can_disable_inline_spans_for_replacements() {
        let output =
            diff_impl(b"hello world\n", b"hello typst\n", br#"{"inline":"none"}"#).unwrap();
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
    fn reports_debug_metadata() {
        let output = diff_impl(
            b"let x = 1\n",
            b"let  x = 2\n",
            br#"{"algorithm":"patience","inline":"words","unicode":false,"ignore_whitespace":true,"show_whitespace":true,"semantic_cleanup":true}"#,
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
        let messages = value["meta"]["messages"].as_array().unwrap();

        assert_eq!(value["meta"]["algorithm"], "patience");
        assert_eq!(value["meta"]["inline"], "words");
        assert_eq!(value["meta"]["unicode"], false);
        assert_eq!(value["meta"]["ignore_whitespace"], true);
        assert_eq!(value["meta"]["show_whitespace"], true);
        assert_eq!(value["meta"]["semantic_cleanup"], true);
        assert!(messages
            .iter()
            .any(|message| message.as_str() == Some("line matching ignores whitespace")));
        assert!(messages
            .iter()
            .any(|message| message.as_str() == Some("semantic cleanup enabled for inline spans")));
    }

    #[test]
    fn reports_trailing_newline_metadata() {
        let output = diff_impl(b"a\n", b"a", br#"{}"#).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
        let messages = value["meta"]["messages"].as_array().unwrap();

        assert_eq!(value["meta"]["old_trailing_newline"], true);
        assert_eq!(value["meta"]["new_trailing_newline"], false);
        assert_eq!(value["meta"]["old_line_endings"], "lf");
        assert_eq!(value["meta"]["new_line_endings"], "none");
        assert!(messages
            .iter()
            .any(|message| message.as_str() == Some("files differ only by trailing newline")));
    }

    #[test]
    fn reports_line_ending_metadata() {
        let output = diff_impl(b"a\r\nb\r\n", b"a\nb\n", br#"{}"#).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
        let messages = value["meta"]["messages"].as_array().unwrap();

        assert_eq!(value["meta"]["old_line_endings"], "crlf");
        assert_eq!(value["meta"]["new_line_endings"], "lf");
        assert!(messages
            .iter()
            .any(|message| message.as_str() == Some("line endings differ: crlf -> lf")));
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

    #[test]
    fn rejects_malformed_options_json() {
        let err = diff_impl(b"a\n", b"b\n", br#"{"inline":"chars""#).unwrap_err();

        assert!(err.contains("invalid options JSON"));
    }

    #[test]
    fn rejects_wrong_option_types() {
        let err = diff_impl(b"a\n", b"b\n", br#"{"unicode":"false"}"#).unwrap_err();

        assert!(err.contains("invalid options JSON"));
    }

    #[test]
    fn rejects_unknown_option_keys() {
        let err = diff_impl(b"a\n", b"b\n", br#"{"semantic_cleaup":true}"#).unwrap_err();

        assert!(err.contains("invalid options JSON"));
        assert!(err.contains("unknown field"));
    }
}
