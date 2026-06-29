use super::*;

#[test]
fn reports_basic_line_changes() {
    let output = diff_impl(b"a\nb\nc\n", b"a\nbee\nc\nd\n", br"{}").unwrap();
    let value: serde_json::Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(value["stats"]["deletions"], 1);
    assert_eq!(value["stats"]["additions"], 2);
    assert_eq!(value["stats"]["equal_lines"], 2);
    assert_eq!(value["stats"]["similarity"], 4.0 / 7.0);
    assert_eq!(value["rows"].as_array().unwrap().len(), 4);
}

#[test]
fn reports_full_similarity_for_empty_files() {
    let output = diff_impl(b"", b"", br"{}").unwrap();
    let value: serde_json::Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(value["stats"]["similarity"], 1.0);
}

#[test]
fn reports_line_ops_with_row_ranges() {
    let output = diff_impl(b"a\nb\nc\n", b"a\nbee\nc\nd\n", br"{}").unwrap();
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
    let output = diff_impl(b"hello world\n", b"hello typst\n", br"{}").unwrap();
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
        .any(
            |span| span["kind"] == "insert-marker" && span["text"].as_str().unwrap().contains('·')
        ));
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
    let output = diff_impl(b"hello world\n", b"hello typst\n", br#"{"inline":"none"}"#).unwrap();
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
fn defaults_to_presentable_diff_options() {
    let output = diff_impl(b"old input\n", b"new input\n", br"{}").unwrap();
    let value: serde_json::Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(value["meta"]["algorithm"], "histogram");
    assert_eq!(value["meta"]["inline"], "words");
    assert_eq!(value["meta"]["semantic_cleanup"], true);
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
    let output = diff_impl(b"a\n", b"a", br"{}").unwrap();
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
    let output = diff_impl(b"a\r\nb\r\n", b"a\nb\n", br"{}").unwrap();
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
