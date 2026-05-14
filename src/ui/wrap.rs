//! Word-wrap helpers operating on the typing target as a `&[char]`.
//!
//! Pure functions, no rendering dependencies.

/// Break `target` into `(start, end_exclusive)` ranges of at most `width`
/// chars, breaking on `\n` (hard) and at the last space before the limit
/// (soft). Always returns at least one range.
pub fn wrap_lines(target: &[char], width: usize) -> Vec<(usize, usize)> {
    let mut lines = Vec::new();
    let len = target.len();
    let mut start = 0;
    while start < len {
        let hard_end = (start + width).min(len);
        let newline = target[start..hard_end].iter().position(|c| *c == '\n');
        let end = if let Some(p) = newline {
            start + p + 1
        } else if hard_end < len {
            target[start..hard_end]
                .iter()
                .rposition(|c| *c == ' ')
                .map(|p| start + p + 1)
                .unwrap_or(hard_end)
        } else {
            hard_end
        };
        let end = if end == start {
            hard_end.max(start + 1)
        } else {
            end
        };
        lines.push((start, end));
        start = end;
    }
    if lines.is_empty() {
        lines.push((0, 0));
    }
    lines
}

/// Returns the index of the line containing the cursor.
pub fn find_line(lines: &[(usize, usize)], cursor: usize) -> usize {
    for (i, (_, e)) in lines.iter().enumerate() {
        if cursor < *e {
            return i;
        }
        if cursor == *e && i + 1 == lines.len() {
            return i;
        }
    }
    lines.len().saturating_sub(1)
}
