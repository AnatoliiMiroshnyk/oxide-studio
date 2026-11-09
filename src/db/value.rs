use rusqlite::types::ValueRef;

const MAX_TEXT_CHARS: usize = 16_384;

pub(crate) fn display_value(value: ValueRef<'_>) -> String {
    match value {
        ValueRef::Null => "NULL".to_owned(),
        ValueRef::Integer(number) => number.to_string(),
        ValueRef::Real(number) => number.to_string(),
        ValueRef::Text(bytes) => {
            let decoded = String::from_utf8_lossy(bytes);
            let mut chars = decoded.chars();
            let mut text: String = chars.by_ref().take(MAX_TEXT_CHARS).collect();
            if chars.next().is_some() {
                text.push_str("… [cell truncated]");
            }
            text
        }
        ValueRef::Blob(bytes) => format!("[BLOB · {} bytes]", bytes.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_values_are_capped_for_display() {
        let bytes = vec![b'x'; MAX_TEXT_CHARS + 5];
        let rendered = display_value(ValueRef::Text(&bytes));
        assert!(rendered.starts_with(&"x".repeat(MAX_TEXT_CHARS)));
        assert!(rendered.ends_with("[cell truncated]"));
    }

    #[test]
    fn blobs_are_summarized_without_copying_payload() {
        let bytes = [1_u8, 2, 3, 4];
        assert_eq!(display_value(ValueRef::Blob(&bytes)), "[BLOB · 4 bytes]");
    }
}
