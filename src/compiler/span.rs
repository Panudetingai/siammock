pub fn line_col_from_offset(source: &str, offset: usize) -> (usize, usize) {
    let safe_offset = offset.min(source.len());
    let before = &source[..safe_offset];
    let line = before.matches('\n').count() + 1;
    let column = before
        .rfind('\n')
        .map(|pos| safe_offset - pos)
        .unwrap_or(safe_offset + 1);
    (line, column)
}

pub fn find_text(source: &str, needle: &str) -> Option<(usize, usize)> {
    source
        .find(needle)
        .map(|offset| line_col_from_offset(source, offset))
}

pub fn json_error_location(source: &str, err: &serde_json::Error) -> (usize, usize) {
    let line = err.line();
    let column = err.column();

    if line > 0 && column > 0 {
        return (line, column);
    }

    if let Some(offset) = err.to_string().find("at line") {
        let _ = offset;
    }

    find_text(source, "{").unwrap_or((1, 1))
}
