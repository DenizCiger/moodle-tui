pub fn decode_html_entities(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'&' {
            if let Some(end) = value[i + 1..].find(';') {
                let entity = &value[i + 1..i + 1 + end];
                if let Some(replacement) = resolve_entity(entity) {
                    out.push_str(&replacement);
                    i += end + 2;
                    continue;
                }
            }
        }
        let ch = value[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn resolve_entity(entity: &str) -> Option<String> {
    if let Some(rest) = entity.strip_prefix('#') {
        let (radix, digits) = if let Some(hex) = rest.strip_prefix(['x', 'X']) {
            (16, hex)
        } else {
            (10, rest)
        };
        let code = u32::from_str_radix(digits, radix).ok()?;
        if code == 0 {
            return None;
        }
        return char::from_u32(code).map(|c| c.to_string());
    }
    Some(
        match entity.to_ascii_lowercase().as_str() {
            "amp" => "&",
            "lt" => "<",
            "gt" => ">",
            "quot" => "\"",
            "apos" => "'",
            "nbsp" => " ",
            _ => return None,
        }
        .to_owned(),
    )
}

pub fn strip_html(value: &str) -> String {
    let decoded = decode_html_entities(value);
    let mut out = String::with_capacity(decoded.len());
    let mut in_tag = false;
    for ch in decoded.chars() {
        match (in_tag, ch) {
            (false, '<') => in_tag = true,
            (true, '>') => {
                in_tag = false;
                out.push(' ');
            }
            (false, _) => out.push(ch),
            _ => {}
        }
    }
    let mut collapsed = String::with_capacity(out.len());
    let mut prev_space = false;
    for ch in out.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                collapsed.push(' ');
                prev_space = true;
            }
        } else {
            collapsed.push(ch);
            prev_space = false;
        }
    }
    collapsed.trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_named_and_numeric() {
        assert_eq!(decode_html_entities("Tom &amp; Jerry"), "Tom & Jerry");
        assert_eq!(decode_html_entities("&#65;"), "A");
        assert_eq!(decode_html_entities("&#x4e2d;"), "中");
        assert_eq!(decode_html_entities("plain"), "plain");
        assert_eq!(decode_html_entities("&unknown;"), "&unknown;");
    }

    #[test]
    fn strips_tags_and_collapses_whitespace() {
        assert_eq!(strip_html("<p>Hello <b>world</b></p>"), "Hello world");
        assert_eq!(strip_html("a<br/>b\n\nc"), "a b c");
        assert_eq!(strip_html("Tom &amp; <i>Jerry</i>"), "Tom & Jerry");
    }
}
