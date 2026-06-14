use crate::models::{QuizAnswerControl, QuizAnswerKind, QuizAnswerOption};
use crate::moodle::html::strip_html;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
struct InputTag {
    attrs: BTreeMap<String, String>,
    label: String,
}

pub fn parse_question_controls(html: &str) -> Vec<QuizAnswerControl> {
    let mut controls = Vec::new();
    let inputs = parse_inputs(html);
    let mut radio_groups: BTreeMap<String, Vec<QuizAnswerOption>> = BTreeMap::new();
    let mut check_groups: BTreeMap<String, Vec<QuizAnswerOption>> = BTreeMap::new();

    for input in inputs {
        let name = match input.attrs.get("name").cloned().filter(|s| !s.is_empty()) {
            Some(name) => name,
            None => continue,
        };
        if name.contains(":flagged") {
            continue;
        }
        let kind = input
            .attrs
            .get("type")
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_else(|| "text".to_owned());
        let value = input.attrs.get("value").cloned().unwrap_or_default();
        let selected = input.attrs.contains_key("checked");
        match kind.as_str() {
            "hidden" => controls.push(QuizAnswerControl {
                name,
                kind: QuizAnswerKind::Hidden,
                options: Vec::new(),
                value,
            }),
            "radio" => radio_groups
                .entry(name)
                .or_default()
                .push(QuizAnswerOption {
                    label: fallback_label(&input.label, &value),
                    value,
                    selected,
                }),
            "checkbox" => check_groups
                .entry(name)
                .or_default()
                .push(QuizAnswerOption {
                    label: fallback_label(&input.label, &value),
                    value,
                    selected,
                }),
            "text" | "number" => controls.push(QuizAnswerControl {
                name,
                kind: QuizAnswerKind::Text,
                options: Vec::new(),
                value,
            }),
            "submit" | "button" => {}
            _ => controls.push(QuizAnswerControl {
                name,
                kind: QuizAnswerKind::Unsupported,
                options: Vec::new(),
                value: kind,
            }),
        }
    }

    for (name, options) in radio_groups {
        controls.push(QuizAnswerControl {
            name,
            kind: QuizAnswerKind::SingleChoice,
            options,
            value: String::new(),
        });
    }
    for (name, options) in check_groups {
        controls.push(QuizAnswerControl {
            name,
            kind: QuizAnswerKind::MultiChoice,
            options,
            value: String::new(),
        });
    }
    controls.extend(parse_selects(html));
    controls
}

pub fn build_answer_params(controls: &[QuizAnswerControl]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for control in controls {
        match control.kind {
            QuizAnswerKind::Hidden | QuizAnswerKind::Text => {
                out.push((control.name.clone(), control.value.clone()));
            }
            QuizAnswerKind::SingleChoice => {
                if let Some(option) = control.options.iter().find(|o| o.selected) {
                    out.push((control.name.clone(), option.value.clone()));
                }
            }
            QuizAnswerKind::MultiChoice => {
                for option in control.options.iter().filter(|o| o.selected) {
                    out.push((control.name.clone(), option.value.clone()));
                }
            }
            QuizAnswerKind::Unsupported => {}
        }
    }
    out
}

fn parse_inputs(html: &str) -> Vec<InputTag> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    while let Some(start_rel) = html[offset..].find("<input") {
        let start = offset + start_rel;
        let Some(end_rel) = html[start..].find('>') else {
            break;
        };
        let end = start + end_rel + 1;
        let tag = &html[start..end];
        out.push(InputTag {
            attrs: parse_attrs(tag),
            label: nearby_label(html, end),
        });
        offset = end;
    }
    out
}

fn parse_selects(html: &str) -> Vec<QuizAnswerControl> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    while let Some(start_rel) = html[offset..].find("<select") {
        let start = offset + start_rel;
        let Some(open_end_rel) = html[start..].find('>') else {
            break;
        };
        let open_end = start + open_end_rel + 1;
        let Some(close_rel) = html[open_end..].find("</select>") else {
            break;
        };
        let close = open_end + close_rel;
        let attrs = parse_attrs(&html[start..open_end]);
        let Some(name) = attrs.get("name").cloned().filter(|s| !s.is_empty()) else {
            offset = close + "</select>".len();
            continue;
        };
        let mut options = Vec::new();
        let mut inner_offset = open_end;
        while inner_offset < close {
            let Some(opt_rel) = html[inner_offset..close].find("<option") else {
                break;
            };
            let opt_start = inner_offset + opt_rel;
            let Some(opt_open_end_rel) = html[opt_start..close].find('>') else {
                break;
            };
            let opt_open_end = opt_start + opt_open_end_rel + 1;
            let Some(opt_close_rel) = html[opt_open_end..close].find("</option>") else {
                break;
            };
            let opt_close = opt_open_end + opt_close_rel;
            let opt_attrs = parse_attrs(&html[opt_start..opt_open_end]);
            let value = opt_attrs.get("value").cloned().unwrap_or_default();
            options.push(QuizAnswerOption {
                label: fallback_label(&strip_html(&html[opt_open_end..opt_close]), &value),
                value,
                selected: opt_attrs.contains_key("selected"),
            });
            inner_offset = opt_close + "</option>".len();
        }
        out.push(QuizAnswerControl {
            name,
            kind: QuizAnswerKind::SingleChoice,
            options,
            value: String::new(),
        });
        offset = close + "</select>".len();
    }
    out
}

fn parse_attrs(tag: &str) -> BTreeMap<String, String> {
    let mut attrs = BTreeMap::new();
    let mut chars = tag.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_whitespace() {
            let mut name = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() || c == '=' || c == '>' || c == '/' {
                    break;
                }
                name.push(c);
                chars.next();
            }
            if name.is_empty() {
                continue;
            }
            while chars.peek().is_some_and(|c| c.is_whitespace()) {
                chars.next();
            }
            let mut value = String::new();
            if chars.peek() == Some(&'=') {
                chars.next();
                while chars.peek().is_some_and(|c| c.is_whitespace()) {
                    chars.next();
                }
                if let Some(quote @ ('"' | '\'')) = chars.peek().copied() {
                    chars.next();
                    while let Some(c) = chars.next() {
                        if c == quote {
                            break;
                        }
                        value.push(c);
                    }
                } else {
                    while let Some(&c) = chars.peek() {
                        if c.is_whitespace() || c == '>' {
                            break;
                        }
                        value.push(c);
                        chars.next();
                    }
                }
            }
            attrs.insert(name.to_ascii_lowercase(), html_unescape(&value));
        }
    }
    attrs
}

fn nearby_label(html: &str, after_tag: usize) -> String {
    let tail = &html[after_tag..html.len().min(after_tag + 400)];
    if let Some(start) = tail.find("<label") {
        if let Some(open_end_rel) = tail[start..].find('>') {
            let open_end = start + open_end_rel + 1;
            if let Some(close_rel) = tail[open_end..].find("</label>") {
                return strip_html(&tail[open_end..open_end + close_rel]);
            }
        }
    }
    String::new()
}

fn fallback_label(label: &str, value: &str) -> String {
    if label.trim().is_empty() {
        value.to_owned()
    } else {
        label.trim().to_owned()
    }
}

fn html_unescape(value: &str) -> String {
    crate::moodle::html::decode_html_entities(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_radio_and_hidden_controls() {
        let controls = parse_question_controls(
            r#"<input type="hidden" name="q1:1_:sequencecheck" value="1">
            <input type="radio" name="q1:1_answer" value="0" checked><label>True</label>
            <input type="radio" name="q1:1_answer" value="1"><label>False</label>"#,
        );
        assert!(controls.iter().any(|c| c.kind == QuizAnswerKind::Hidden));
        let radio = controls
            .iter()
            .find(|c| c.kind == QuizAnswerKind::SingleChoice)
            .unwrap();
        assert_eq!(radio.name, "q1:1_answer");
        assert_eq!(radio.options.len(), 2);
        assert!(radio.options[0].selected);
    }

    #[test]
    fn builds_answer_params_without_unsupported_controls() {
        let controls = vec![
            QuizAnswerControl {
                name: "a".into(),
                kind: QuizAnswerKind::Text,
                options: Vec::new(),
                value: "42".into(),
            },
            QuizAnswerControl {
                name: "b".into(),
                kind: QuizAnswerKind::Unsupported,
                options: Vec::new(),
                value: "file".into(),
            },
        ];
        assert_eq!(
            build_answer_params(&controls),
            vec![("a".into(), "42".into())]
        );
    }
}
