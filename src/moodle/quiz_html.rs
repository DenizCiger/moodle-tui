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
                .extend(option_from_input(
                    &input.attrs["name"],
                    &input.label,
                    &value,
                    selected,
                )),
            "checkbox" => check_groups
                .entry(checkbox_group_name(&name))
                .or_default()
                .extend(option_from_input(&name, &input.label, &value, selected)),
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
    controls.extend(parse_textareas(html));
    controls
}

pub fn question_text_from_html(html: &str) -> String {
    if let Some(qtext) = inner_html_by_class(html, "qtext") {
        let text = strip_html(&qtext);
        if !text.trim().is_empty() {
            return text;
        }
    }

    let mut remaining = html.to_owned();
    let lower = html.to_ascii_lowercase();
    let first_answer_control = [
        "<div class=\"answer\"",
        "<div class=\"ablock\"",
        "<input",
        "<select",
        "<textarea",
    ]
    .iter()
    .filter_map(|needle| lower.find(needle))
    .min();
    if let Some(idx) = first_answer_control {
        remaining.truncate(idx);
    }
    let text = strip_html(&remaining);
    if text.trim().is_empty() {
        strip_html(html)
    } else {
        text
    }
}

fn inner_html_by_class(html: &str, class_name: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let class_marker = format!("class=\"{class_name}\"");
    let class_idx = lower.find(&class_marker)?;
    let tag_start = lower[..class_idx].rfind('<')?;
    let open_end = lower[tag_start..].find('>')? + tag_start;
    let tag = &lower[tag_start + 1..open_end];
    let tag_name = tag.split_whitespace().next()?;
    let close_tag = format!("</{tag_name}>");
    let close_idx = lower[open_end + 1..].find(&close_tag)? + open_end + 1;
    Some(html[open_end + 1..close_idx].to_owned())
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
                    out.push((
                        option.name.clone().unwrap_or_else(|| control.name.clone()),
                        option.value.clone(),
                    ));
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
        let attrs = parse_attrs(tag);
        let label = input_label(html, end, &attrs);
        out.push(InputTag { attrs, label });
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
                name: None,
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

fn parse_textareas(html: &str) -> Vec<QuizAnswerControl> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    let lower = html.to_ascii_lowercase();
    while let Some(start_rel) = lower[offset..].find("<textarea") {
        let start = offset + start_rel;
        let Some(open_end_rel) = lower[start..].find('>') else {
            break;
        };
        let open_end = start + open_end_rel + 1;
        let Some(close_rel) = lower[open_end..].find("</textarea>") else {
            break;
        };
        let close = open_end + close_rel;
        let attrs = parse_attrs(&html[start..open_end]);
        if let Some(name) = attrs.get("name").cloned().filter(|s| !s.is_empty()) {
            out.push(QuizAnswerControl {
                name,
                kind: QuizAnswerKind::Text,
                options: Vec::new(),
                value: strip_html(&html[open_end..close]),
            });
        }
        offset = close + "</textarea>".len();
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

fn input_label(html: &str, after_tag: usize, attrs: &BTreeMap<String, String>) -> String {
    if let Some(label_id) = attrs.get("aria-labelledby").or_else(|| attrs.get("id")) {
        if let Some(label) = element_text_by_id(html, label_id) {
            return label;
        }
    }
    nearby_label(html, after_tag)
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

fn element_text_by_id(html: &str, id: &str) -> Option<String> {
    let needle = format!("id=\"{id}\"");
    let start = html.find(&needle)?;
    let tag_start = html[..start].rfind('<')?;
    let open_end = html[tag_start..].find('>')? + tag_start;
    let tag = &html[tag_start + 1..open_end];
    let tag_name = tag.split_whitespace().next()?.trim_matches('/');
    let close_tag = format!("</{tag_name}>");
    let close = html[open_end + 1..].find(&close_tag)? + open_end + 1;
    let text = strip_html(&html[open_end + 1..close]);
    if text.trim().is_empty() {
        None
    } else {
        Some(text)
    }
}

fn fallback_label(label: &str, value: &str) -> String {
    if label.trim().is_empty() {
        value.to_owned()
    } else {
        label.trim().to_owned()
    }
}

fn checkbox_group_name(name: &str) -> String {
    if let Some(idx) = name.rfind("_choice") {
        let suffix = &name[idx + "_choice".len()..];
        if !suffix.is_empty() && suffix.chars().all(|ch| ch.is_ascii_digit()) {
            return name[..idx].to_owned();
        }
    }
    name.to_owned()
}

fn option_from_input(
    name: &str,
    label: &str,
    value: &str,
    selected: bool,
) -> Option<QuizAnswerOption> {
    let label = fallback_label(label, value);
    let normalized = label.trim().to_ascii_lowercase();
    if normalized == "clear my choice" {
        return None;
    }
    Some(QuizAnswerOption {
        name: Some(name.to_owned()),
        label,
        value: value.to_owned(),
        selected,
    })
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
    fn skips_moodle_clear_choice_radio_control() {
        let controls = parse_question_controls(
            r#"<input type="radio" name="q1:1_answer" value="0"><label>Assignment</label>
            <input type="radio" name="q1:1_answer" value="-1" checked><label>Clear my choice</label>"#,
        );
        let radio = controls
            .iter()
            .find(|c| c.kind == QuizAnswerKind::SingleChoice)
            .unwrap();
        assert_eq!(radio.options.len(), 1);
        assert_eq!(radio.options[0].label, "Assignment");
    }

    #[test]
    fn groups_moodle_multichoice_checkboxes_by_question() {
        let controls = parse_question_controls(
            r#"<input type="hidden" name="q7:4_choice0" value="0" />
            <input type="checkbox" name="q7:4_choice0" value="1" id="q7:4_choice0" aria-labelledby="q7:4_choice0_label" />
            <div id="q7:4_choice0_label"><span>a. </span><div>Assignment</div></div>
            <input type="hidden" name="q7:4_choice1" value="0" />
            <input type="checkbox" name="q7:4_choice1" value="1" id="q7:4_choice1" aria-labelledby="q7:4_choice1_label" checked />
            <div id="q7:4_choice1_label"><span>b. </span><div>Quiz</div></div>"#,
        );
        let checks: Vec<_> = controls
            .iter()
            .filter(|c| c.kind == QuizAnswerKind::MultiChoice)
            .collect();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].name, "q7:4");
        assert_eq!(checks[0].options.len(), 2);
        assert_eq!(checks[0].options[0].name.as_deref(), Some("q7:4_choice0"));
        assert_eq!(checks[0].options[0].label, "a. Assignment");
        assert!(checks[0].options[1].selected);
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

    #[test]
    fn parses_textarea_controls() {
        let controls = parse_question_controls(
            r#"<div class="qtext">Explain.</div><textarea name="q4:1_answer">old</textarea>"#,
        );
        let text = controls
            .iter()
            .find(|control| control.kind == QuizAnswerKind::Text)
            .unwrap();
        assert_eq!(text.name, "q4:1_answer");
        assert_eq!(text.value, "old");
    }

    #[test]
    fn extracts_question_text_before_answer_controls() {
        let text = question_text_from_html(
            r#"<div class="qtext"><p>What is 6 * 7?</p></div><div class="ablock"><input type="text" name="q1:1_answer"></div>"#,
        );
        assert_eq!(text, "What is 6 * 7?");
    }

    #[test]
    fn prefers_moodle_qtext_over_question_state() {
        let text = question_text_from_html(
            r#"<div class="info">Question 1 Not yet answered</div><div class="content"><div class="qtext"><p>The Moodle TUI can save simple quiz answers.</p></div><div class="ablock"><div class="answer"><input type="radio"></div></div></div>"#,
        );
        assert_eq!(text, "The Moodle TUI can save simple quiz answers.");
    }
}
