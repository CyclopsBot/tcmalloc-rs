use gallium::TextProcessExt;

#[test]
fn remove_markdown_basic() {
    let text = "*italics* _italics_ **bold** ~~strikethrough~~ `code` ||spoiler||";
    let expected = "italics italics bold strikethrough code spoiler";
    assert_eq!(text.remove_markdown(), expected);
}

#[test]
fn remove_markdown_complex() {
    let text = "**bold ||spoiler|| `code`**";
    let expected = "bold spoiler code";
    assert_eq!(text.remove_markdown(), expected);
}

#[test]
fn remove_markdown_invalid() {
    let text = "*italics* _italics_ ***bold** ~~strikethrough~~~ `code` ||spoiler||";
    let expected = "italics italics bold strikethrough code spoiler";
    assert_eq!(text.remove_markdown(), expected);
}

#[test]
fn remove_markdown_empty() {
    let text = "";
    let expected = "";
    assert_eq!(text.remove_markdown(), expected);
}

#[test]
fn remove_markdown_no_markdown() {
    let text = "no markdown here";
    let expected = "no markdown here";
    assert_eq!(text.remove_markdown(), expected);
}

#[test]
fn remove_markdown_only_markdown() {
    let text = "****";
    let expected = "";
    assert_eq!(text.remove_markdown(), expected);
}

#[test]
fn remove_markdown_escaped_characters() {
    let text = "\\*escaped\\* \\_escaped\\_ \\~escaped\\~ \\`escaped\\`";
    let expected = "escaped escaped escaped escaped";
    assert_eq!(text.remove_markdown(), expected);
}

#[test]
fn remove_markdown_edge_case() {
    let text = "*a**b__c~d`e|f~`";
    let expected = "abcdef";
    assert_eq!(text.remove_markdown(), expected);
}

#[test]
fn remove_markdown_nested_formatting() {
    let text = "**bold _italic_ **nested bold** _italic again_**";
    let expected = "bold italic nested bold italic again";
    assert_eq!(text.remove_markdown(), expected);
}

#[test]
fn remove_markdown_multiple_consecutive() {
    let text = "***bold and italic*** __underlined__ ~strikethrough~";
    let expected = "bold and italic underlined strikethrough";
    assert_eq!(text.remove_markdown(), expected);
}

#[test]
fn remove_markdown_mixed_formatting() {
    let text = "**bold** _italic_ ***bold italic*** `code` ~~strikethrough~~";
    let expected = "bold italic bold italic code strikethrough";
    assert_eq!(text.remove_markdown(), expected);
}
