use crate::pdf::rich_string::RichString;
use crate::pdf::screenplay::{Dialogue, DialogueElement, Element, Screenplay};

const FDX_TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<FinalDraft DocumentType="Script" Template="No" Version="1">
<Content>
%%CONTENT%%
</Content>
%%TITLE_PAGE%%
<PageLayout>
<PageSize Height="11.0" Width="8.50"/>
</PageLayout>
</FinalDraft>
"#;

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn element_type_to_fdx(el: &Element) -> Option<String> {
    match el {
        Element::Heading { .. } => Some("Scene Heading".into()),
        Element::Action(_) => Some("Action".into()),
        Element::Dialogue(d) => Some(
            match d.extension.as_ref() {
                Some(ext)
                    if ext.to_plain_string().eq_ignore_ascii_case("V.O.")
                        || ext.to_plain_string().eq_ignore_ascii_case("V.O") =>
                {
                    "Character"
                }
                _ => "Character",
            }
            .into(),
        ),
        Element::DualDialogue(_, _) => None,
        Element::Lyrics(_) => Some("Lyrics".into()),
        Element::Transition(_) => Some("Transition".into()),
        Element::CenteredText(_) => Some("Action".into()),
        Element::Synopsis(_) => None,
        Element::Section(_) => Some("Outline 1".into()),
        Element::Shot(_) => Some("Shot".into()),
        Element::PageBreak => None,
    }
}

fn rich_to_fdx_text(rs: &RichString) -> String {
    let mut out = String::new();
    for el in &rs.elements {
        let text = escape_xml(&el.text);
        let mut attrs = Vec::new();
        let mut styles = Vec::new();
        if el.is_bold() {
            styles.push("Bold");
        }
        if el.is_italic() {
            styles.push("Italic");
        }
        if el.is_underline() {
            styles.push("Underline");
        }
        if !styles.is_empty() {
            attrs.push(format!("Style=\"{}\"", styles.join("+")));
        }
        let attr_str = if attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", attrs.join(" "))
        };
        out.push_str(&format!("        <Text{}>{}</Text>\n", attr_str, text));
    }
    out
}

fn dialogue_to_fdx(dialogue: &Dialogue, prefix: &str) -> String {
    let mut out = String::new();
    let char_name = escape_xml(&dialogue.character.to_plain_string());
    let extension = dialogue.extension.as_ref().map(|e| escape_xml(&e.to_plain_string()));
    let char_line = match extension {
        Some(ext) => format!("{} ({})", char_name, ext),
        None => char_name.clone(),
    };
    out.push_str(&format!(
        "{}      <Paragraph Type=\"{}\">\n{}      </Paragraph>\n",
        prefix,
        "Character",
        rich_to_fdx_text(&RichString::from(char_line.as_str())),
    ));
    for elem in &dialogue.elements {
        match elem {
            DialogueElement::Parenthetical(p) => {
                out.push_str(&format!(
                    "{}      <Paragraph Type=\"Parenthetical\">\n{}      </Paragraph>\n",
                    prefix,
                    rich_to_fdx_text(p),
                ));
            }
            DialogueElement::Line(l) => {
                out.push_str(&format!(
                    "{}      <Paragraph Type=\"Dialogue\">\n{}      </Paragraph>\n",
                    prefix,
                    rich_to_fdx_text(l),
                ));
            }
        }
    }
    out
}

fn element_to_fdx(el: &Element) -> String {
    match el {
        Element::Heading { slug, number } => {
            let mut attrs = vec![format!("Type=\"{}\"", element_type_to_fdx(el).unwrap())];
            if let Some(num) = number {
                attrs.push(format!("Number=\"{}\"", escape_xml(num)));
            }
            let mut out = format!("      <Paragraph {}>\n", attrs.join(" "));
            out.push_str(&rich_to_fdx_text(slug));
            out.push_str("      </Paragraph>\n");
            out
        }
        Element::Action(rs) => {
            format!(
                "      <Paragraph Type=\"Action\">\n{}      </Paragraph>\n",
                rich_to_fdx_text(rs)
            )
        }
        Element::Dialogue(d) => dialogue_to_fdx(d, ""),
        Element::DualDialogue(left, right) => {
            let mut out = "      <Paragraph>\n        <DualDialogue>\n".to_string();
            out.push_str(&dialogue_to_fdx(left, "          "));
            out.push_str(&dialogue_to_fdx(right, "          "));
            out.push_str("        </DualDialogue>\n      </Paragraph>\n");
            out
        }
        Element::Lyrics(rs) => {
            format!(
                "      <Paragraph Type=\"Lyrics\">\n{}      </Paragraph>\n",
                rich_to_fdx_text(rs)
            )
        }
        Element::Transition(rs) => {
            format!(
                "      <Paragraph Type=\"Transition\">\n{}      </Paragraph>\n",
                rich_to_fdx_text(rs)
            )
        }
        Element::CenteredText(rs) => {
            format!(
                "      <Paragraph Type=\"Action\" Alignment=\"Centered\">\n{}      </Paragraph>\n",
                rich_to_fdx_text(rs)
            )
        }
        Element::Section(rs) => {
            let plain = rs.to_plain_string();
            let hashes = plain.chars().take_while(|c| *c == '#').count();
            let outline_type = match hashes {
                1 => "Outline 1",
                2 => "Outline 2",
                3 => "Outline 3",
                _ => "Outline 1",
            };
            format!(
                "      <Paragraph Type=\"{}\">\n{}      </Paragraph>\n",
                outline_type,
                rich_to_fdx_text(rs)
            )
        }
        Element::Shot(rs) => {
            format!(
                "      <Paragraph Type=\"Shot\">\n{}      </Paragraph>\n",
                rich_to_fdx_text(rs)
            )
        }
        Element::Synopsis(_) => String::new(),
        Element::PageBreak => String::new(),
    }
}

fn build_title_page(titlepage: &crate::pdf::screenplay::TitlePage) -> String {
    let has_content = !titlepage.title.is_empty()
        || !titlepage.authors.is_empty()
        || !titlepage.credit.is_empty()
        || !titlepage.source.is_empty()
        || !titlepage.draft_date.is_empty()
        || !titlepage.contact.is_empty()
        || !titlepage.notes.is_empty();
    if !has_content {
        return String::new();
    }

    let title = titlepage.title.first().map(|r| r.to_plain_string());
    let credit = titlepage.credit.first().map(|r| r.to_plain_string());
    let author = titlepage.authors.first().map(|r| r.to_plain_string());
    let source = titlepage.source.first().map(|r| r.to_plain_string());
    let draft_date = titlepage.draft_date.first().map(|r| r.to_plain_string());
    let contact = titlepage.contact.first().map(|r| r.to_plain_string());

    let mut lines: Vec<String> = Vec::new();

    for _ in 0..18 {
        lines.push(empty_title_line());
    }

    if let Some(t) = &title {
        lines.push(centered_title_line(t));
    }
    if let Some(c) = &credit {
        lines.push(empty_title_line());
        lines.push(centered_title_line(c));
    }
    if let Some(a) = &author {
        lines.push(empty_title_line());
        lines.push(centered_title_line(a));
    }
    if let Some(s) = &source {
        lines.push(empty_title_line());
        lines.push(centered_title_line(s));
    }

    while lines.len() < 44 {
        lines.push(empty_title_line());
    }

    if let Some(d) = &draft_date {
        lines.push(left_title_line(d));
    }
    if let Some(c) = &contact {
        lines.push(left_title_line(c));
    }

    let content: String = lines.join("");
    format!(
        "  <TitlePage>\n    <Content>\n{}    </Content>\n  </TitlePage>\n",
        content
    )
}

fn empty_title_line() -> String {
    "      <Paragraph>\n        <Text></Text>\n      </Paragraph>\n".into()
}

fn centered_title_line(text: &str) -> String {
    let escaped = escape_xml(text);
    format!(
        "      <Paragraph Alignment=\"Center\">\n        <Text>{}</Text>\n      </Paragraph>\n",
        escaped
    )
}

fn left_title_line(text: &str) -> String {
    let escaped = escape_xml(text);
    format!(
        "      <Paragraph>\n        <Text>{}</Text>\n      </Paragraph>\n",
        escaped
    )
}

pub fn export(screenplay: &Screenplay) -> String {
    let mut content = String::new();
    for span in &screenplay.elements {
        let fdx = element_to_fdx(&span.inner);
        content.push_str(&fdx);
    }

    let title_page = match &screenplay.titlepage {
        Some(tp) => build_title_page(tp),
        None => String::new(),
    };

    FDX_TEMPLATE
        .replace("%%CONTENT%%", &content)
        .replace("%%TITLE_PAGE%%", &title_page)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::parse;

    #[test]
    fn exports_empty_screenplay() {
        let result = export(&parse(""));
        assert!(result.contains("<FinalDraft"));
        assert!(result.contains("</FinalDraft>"));
    }

    #[test]
    fn exports_scene_heading() {
        let result = export(&parse("INT. HOUSE - DAY\n\nHello."));
        assert!(result.contains("Type=\"Scene Heading\""));
        assert!(result.contains("INT. HOUSE - DAY"));
        assert!(result.contains("Type=\"Action\""));
        assert!(result.contains("Hello."));
    }

    #[test]
    fn exports_scene_number() {
        let result = export(&parse("INT. HOUSE - DAY #1#\n\nHello."));
        assert!(result.contains("Number=\"1\""));
    }

    #[test]
    fn exports_dialogue() {
        let src = "CHAR\nHello!";
        let result = export(&parse(src));
        assert!(result.contains("Type=\"Character\""));
        assert!(result.contains("Type=\"Dialogue\""));
        assert!(result.contains("Hello!"));
    }
}
