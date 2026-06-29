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

type SpanPair = (Option<Vec<InlineSpan>>, Option<Vec<InlineSpan>>);

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
struct RawOptions<'a> {
    #[serde(default)]
    ignore_whitespace: bool,
    #[serde(default)]
    show_whitespace: bool,
    #[serde(default = "default_algorithm")]
    algorithm: &'a str,
    #[serde(default = "default_inline")]
    inline: &'a str,
    #[serde(default = "default_unicode")]
    unicode: bool,
    #[serde(default = "default_true")]
    semantic_cleanup: bool,
}

#[derive(Clone, Copy)]
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
            algorithm: parse_algorithm(raw.algorithm)?,
            inline: parse_inline_mode(raw.inline)?,
            unicode: raw.unicode,
            semantic_cleanup: raw.semantic_cleanup,
        })
    }
}

fn default_algorithm() -> &'static str {
    "histogram"
}

fn default_inline() -> &'static str {
    "words"
}

fn default_unicode() -> bool {
    true
}

fn default_true() -> bool {
    true
}

struct LineInput<'a> {
    text: &'a str,
    lines: Vec<&'a str>,
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
    old_lines: &'a [&'a str],
    new_lines: &'a [&'a str],
    options: Options,
}

impl<'a> ReportBuilder<'a> {
    fn new(old: &'a LineInput<'a>, new: &'a LineInput<'a>, options: Options) -> Self {
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
                rows: Vec::with_capacity(old.lines.len().max(new.lines.len())),
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
            let old_line = self.old_lines[old_index + offset];
            let new_line = self.new_lines[new_index + offset];
            self.report.rows.push(DiffRow {
                kind: "equal",
                old_no: Some(old_index + offset + 1),
                old: Some(old_line),
                old_spans: equal_spans(old_line, self.options.show_whitespace),
                new_no: Some(new_index + offset + 1),
                new: Some(new_line),
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
            let old_line = self.old_lines[old_index + offset];
            self.report.rows.push(DiffRow {
                kind: "delete",
                old_no: Some(old_index + offset + 1),
                old: Some(old_line),
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
            let new_line = self.new_lines[new_index + offset];
            self.report.rows.push(DiffRow {
                kind: "insert",
                old_no: None,
                old: None,
                old_spans: None,
                new_no: Some(new_index + offset + 1),
                new: Some(new_line),
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
            let old_line = (offset < old_len).then(|| self.old_lines[old_index + offset]);
            let new_line = (offset < new_len).then(|| self.new_lines[new_index + offset]);
            let (old_spans, new_spans) = replace_spans(old_line, new_line, self.options)?;

            self.report.rows.push(DiffRow {
                kind: "replace",
                old_no: (offset < old_len).then_some(old_index + offset + 1),
                old: old_line,
                old_spans,
                new_no: (offset < new_len).then_some(new_index + offset + 1),
                new: new_line,
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
pub fn diff(old: &[u8], new: &[u8], options: &[u8]) -> Result<Vec<u8>, String> {
    diff_impl(old, new, options)
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
    let mut builder = ReportBuilder::new(&old, &new, options);

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

fn build_meta(old: &LineInput<'_>, new: &LineInput<'_>, options: Options) -> DiffMeta {
    let algorithm = algorithm_name(options.algorithm);
    let inline = options.inline.name();
    let mut messages = Vec::with_capacity(8);
    messages.push(format!("algorithm: {algorithm}"));
    messages.push(format!("inline: {inline}"));

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
        algorithm,
        inline,
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
    old_lines: &[&str],
    new_lines: &[&str],
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
    options: Options,
) -> Result<SpanPair, String> {
    match (old_line, new_line) {
        (Some(old_line), Some(new_line)) => inline_spans(old_line, new_line, options),
        (Some(old_line), None) => {
            Ok((Some(deleted_spans(old_line, options.show_whitespace)), None))
        }
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

fn inline_spans(old_line: &str, new_line: &str, options: Options) -> Result<SpanPair, String> {
    match (options.inline, options.unicode) {
        (InlineMode::None, _) => Ok((None, None)),
        (InlineMode::Chars, true) => {
            let old_tokens = old_line.tokenize_graphemes();
            let new_tokens = new_line.tokenize_graphemes();
            inline_spans_for_tokens(&old_tokens, &new_tokens, options)
        }
        (InlineMode::Chars, false) => {
            let old_tokens = char_tokens(old_line);
            let new_tokens = char_tokens(new_line);
            inline_spans_for_tokens(&old_tokens, &new_tokens, options)
        }
        (InlineMode::Words, true) => {
            let old_tokens = old_line.tokenize_unicode_words();
            let new_tokens = new_line.tokenize_unicode_words();
            inline_spans_for_tokens(&old_tokens, &new_tokens, options)
        }
        (InlineMode::Words, false) => {
            let old_tokens = word_tokens(old_line);
            let new_tokens = word_tokens(new_line);
            inline_spans_for_tokens(&old_tokens, &new_tokens, options)
        }
    }
}

fn inline_spans_for_tokens<T>(
    old_tokens: &[T],
    new_tokens: &[T],
    options: Options,
) -> Result<SpanPair, String>
where
    T: AsRef<str> + Eq + std::hash::Hash,
{
    let ops = if options.semantic_cleanup {
        capture_compact_diff_slices(options.algorithm, old_tokens, new_tokens)?
    } else {
        capture_diff_slices(options.algorithm, old_tokens, new_tokens)
    };
    let mut old_spans = Vec::with_capacity(ops.len());
    let mut new_spans = Vec::with_capacity(ops.len());

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
                    options.show_whitespace,
                );
            }
            DiffOp::Insert {
                new_index, new_len, ..
            } => {
                push_span(
                    &mut new_spans,
                    "insert",
                    new_tokens[new_index..new_index + new_len].iter(),
                    options.show_whitespace,
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
                    options.show_whitespace,
                );
                push_span(
                    &mut new_spans,
                    "insert",
                    new_tokens[new_index..new_index + new_len].iter(),
                    options.show_whitespace,
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

fn char_tokens(line: &str) -> Vec<String> {
    line.chars().map(|ch| ch.to_string()).collect()
}

fn word_tokens(line: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut start = 0;
    let mut current_kind = None;

    for (index, ch) in line.char_indices() {
        let kind = token_kind(ch);
        if current_kind.is_some_and(|active| active != kind) {
            tokens.push(&line[start..index]);
            start = index;
        }

        current_kind = Some(kind);
    }

    if start < line.len() {
        tokens.push(&line[start..]);
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

fn push_span<'a, T>(
    spans: &mut Vec<InlineSpan>,
    kind: &'static str,
    tokens: impl Iterator<Item = &'a T>,
    show_whitespace: bool,
) where
    T: AsRef<str> + 'a,
{
    if !show_whitespace {
        for token in tokens {
            push_text(spans, kind, token.as_ref());
        }
        return;
    }

    for ch in tokens.flat_map(|token| token.as_ref().chars()) {
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

    push_char(spans, kind, displayed);
}

fn push_text(spans: &mut Vec<InlineSpan>, kind: &'static str, text: &str) {
    if let Some(last) = spans.last_mut() {
        if last.kind == kind {
            last.text.push_str(text);
            return;
        }
    }

    spans.push(InlineSpan {
        kind,
        text: text.to_owned(),
    });
}

fn display_spans(text: &str, kind: &'static str, show_whitespace: bool) -> Vec<InlineSpan> {
    if !show_whitespace {
        return vec![InlineSpan {
            kind,
            text: text.to_owned(),
        }];
    }

    let mut spans = Vec::with_capacity(1);

    for ch in text.chars() {
        push_display_char(&mut spans, kind, ch, show_whitespace);
    }

    spans
}

fn push_char(spans: &mut Vec<InlineSpan>, kind: &'static str, ch: char) {
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

fn display_trailing_whitespace_spans(text: &str) -> Vec<InlineSpan> {
    let start = trailing_horizontal_whitespace_start(text);
    let mut spans = Vec::with_capacity(2);

    if start > 0 {
        push_text(&mut spans, "equal", &text[..start]);
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

fn split_lines(text: &str) -> Vec<&str> {
    let mut lines = text
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
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
    let mut parts = line.split_whitespace();
    let Some(first) = parts.next() else {
        return String::new();
    };

    let mut normalized = String::with_capacity(line.len());
    normalized.push_str(first);

    for part in parts {
        normalized.push(' ');
        normalized.push_str(part);
    }

    normalized
}

#[cfg(test)]
mod tests;
