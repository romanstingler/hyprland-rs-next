//! Lua literal encoding.

/// Escape a string for a double-quoted Lua literal.
#[must_use]
pub(crate) fn escape_str(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\000"),
            c if (c as u32) < 0x20 || c == '\u{7f}' => {
                out.push_str(&format!("\\{:03}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

/// Encode a string as a double-quoted Lua literal.
#[must_use]
pub(crate) fn quote(value: &str) -> String {
    format!("\"{}\"", escape_str(value))
}

/// Encode a table key using bracket syntax.
#[must_use]
pub(crate) fn table_key(key: &str) -> String {
    format!("[{}]", quote(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_quotes_and_backslashes() {
        assert_eq!(escape_str(r#"a\"b"#), r#"a\\\"b"#);
        assert_eq!(quote("x"), r#""x""#);
    }

    #[test]
    fn escapes_control_characters() {
        assert_eq!(escape_str("a\nb\r\tc\0"), r"a\nb\r\tc\000");
        assert_eq!(escape_str("\u{7f}"), r"\127");
    }

    #[test]
    fn table_keys_are_always_bracketed() {
        assert_eq!(table_key("general"), r#"["general"]"#);
        assert_eq!(table_key(r#"a"b"#), r#"["a\"b"]"#);
    }
}
