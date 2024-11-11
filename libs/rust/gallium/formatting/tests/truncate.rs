use gallium::TextProcessExt;

#[test]
fn max_len_basic() {
    assert_eq!("hello world".max_len(9), "hello ...");
}

#[test]
fn max_len_exact() {
    assert_eq!("hello world".max_len(11), "hello world");
}

#[test]
fn max_len_small() {
    assert_eq!("hello world".max_len(3), "...");
}

#[test]
fn max_len_empty() {
    assert_eq!("".max_len(5), "");
}

#[test]
fn max_len_exact_boundary() {
    assert_eq!("hello world".max_len(5), "he...");
}

#[test]
fn max_len_no_truncation() {
    assert_eq!("short".max_len(10), "short");
}

#[test]
#[should_panic(expected = "Maximum length must be at least 3")]
fn max_len_edge_case() {
    assert_eq!("hello".max_len(2), "...");
    assert_eq!("hello".max_len(1), "...");
    assert_eq!("hello".max_len(0), "...");
}
