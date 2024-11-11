#![deny(clippy::all, clippy::pedantic)]

use std::borrow::Cow;

/// Extension trait for [`String`] and [`str`] with text processing utilities.
#[allow(clippy::module_name_repetitions)]
pub trait TextProcessExt {
    /// Remove Discord markdown from the String.
    ///
    /// This function removes **all** characters that can be used to format
    /// markdown. It does not check if it is valid markdown.
    ///
    /// ```
    /// # use gallium::TextProcessExt;
    /// assert_eq!("**markdown will be removed**".remove_markdown(), "markdown will be removed");
    /// assert_eq!("no other text will be removed".remove_markdown(), "no other text will be removed");
    /// ```
    fn remove_markdown(&self) -> Cow<str>;

    /// Truncate text if it exceed a maximum size.
    ///
    /// Truncated characters will be replaced with `...` (without exceeding the
    /// maximum size). Maximum size must be larger at least 3 characters.
    ///
    /// ```
    /// # use crate::gallium::TextProcessExt;
    /// assert_eq!("this will be truncated".to_owned().max_len(10), "this wi...".to_owned());
    /// assert_eq!("this not".to_owned().max_len(10), "this not".to_owned());
    /// ```
    fn max_len(&self, max: usize) -> Cow<str>;
}

impl TextProcessExt for str {
    fn remove_markdown(&self) -> Cow<str> {
        let mut result = String::with_capacity(self.len());
        for c in self.chars() {
            match c {
                '*' | '\\' | '_' | '~' | '|' | '`' => {}
                _ => result.push(c),
            }
        }
        Cow::Owned(result)
    }

    fn max_len(&self, max: usize) -> Cow<str> {
        // Ensure max is at least 3
        assert!(max >= 3, "Maximum length must be at least 3");

        let len = self.len();
        if len <= max {
            Cow::Borrowed(self)
        } else {
            let visible_len = max - 3;
            let truncated = &self[..visible_len];
            Cow::Owned(format!("{truncated}..."))
        }
    }
}
