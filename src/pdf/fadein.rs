use crate::pdf::rich_string::RichString;
use crate::pdf::screenplay::{Dialogue, DialogueElement, Element, Screenplay};

const OSF_TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<osf version="2.0">
<settings>
<page_size width="8.5in" height="11.0in"/>
<margins left="1.5in" right="1.0in" top="1.0in" bottom="1.0in"/>
</settings>
%%TITLE_PAGE%%
<body font="Courier Prime" size="12">
%%CONTENT%%
</body>
</osf>
"#;

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn rich_to_fadein_text(rs: &RichString) -> String {
    let mut out = String::new();
    for el in &rs.elements {
        let text = escape_xml(&el.text);
        let mut attrs = Vec::new();
        if el.is_bold() {
            attrs.push("bold=\"1\"");
        }
        if el.is_italic() {
            attrs.push("italic=\"1\"");
        }
        if el.is_underline() {
            attrs.push("underline=\"1\"");
        }
        let attr_str = if attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", attrs.join(" "))
        };
        out.push_str(&format!("<text{}>{}</text>", attr_str, text));
    }
    out
}

fn dialogue_to_fadein(dialogue: &Dialogue) -> String {
    let mut out = String::new();
    let char_name = escape_xml(&dialogue.character.to_plain_string());
    let extension = dialogue.extension.as_ref().map(|e| escape_xml(&e.to_plain_string()));
    let char_line = match extension {
        Some(ext) => format!("{} ({})", char_name, ext),
        None => char_name,
    };

    out.push_str(&format!(
        "  <para bookmark=\"Character\">\n    <text>{}</text>\n  </para>\n",
        char_line
    ));

    for elem in &dialogue.elements {
        match elem {
            DialogueElement::Parenthetical(p) => {
                out.push_str(&format!(
                    "  <para bookmark=\"Parenthetical\">\n    {}\n  </para>\n",
                    rich_to_fadein_text(p)
                ));
            }
            DialogueElement::Line(l) => {
                out.push_str(&format!(
                    "  <para bookmark=\"Dialogue\">\n    {}\n  </para>\n",
                    rich_to_fadein_text(l)
                ));
            }
        }
    }
    out
}

fn element_to_fadein(el: &Element) -> String {
    match el {
        Element::Heading { slug, number } => {
            let mut num_attr = String::new();
            if let Some(n) = number {
                num_attr = format!(" number=\"{}\"", escape_xml(n));
            }
            format!(
                "  <para bookmark=\"Scene Heading\"{}>\n    {}\n  </para>\n",
                num_attr,
                rich_to_fadein_text(slug)
            )
        }
        Element::Action(rs) => {
            format!(
                "  <para bookmark=\"Action\">\n    {}\n  </para>\n",
                rich_to_fadein_text(rs)
            )
        }
        Element::Dialogue(d) => dialogue_to_fadein(d),
        Element::DualDialogue(left, right) => {
            let mut out = "  <para bookmark=\"Dual Dialogue\">\n".to_string();
            out.push_str(&dialogue_to_fadein(left));
            out.push_str(&dialogue_to_fadein(right));
            out.push_str("  </para>\n");
            out
        }
        Element::Lyrics(rs) => {
            format!(
                "  <para bookmark=\"Lyrics\">\n    {}\n  </para>\n",
                rich_to_fadein_text(rs)
            )
        }
        Element::Transition(rs) => {
            format!(
                "  <para bookmark=\"Transition\">\n    {}\n  </para>\n",
                rich_to_fadein_text(rs)
            )
        }
        Element::CenteredText(rs) => {
            format!(
                "  <para bookmark=\"Action\" align=\"center\">\n    {}\n  </para>\n",
                rich_to_fadein_text(rs)
            )
        }
        Element::Section(rs) => {
            let plain = rs.to_plain_string();
            let hashes = plain.chars().take_while(|c| *c == '#').count();
            let level = if hashes == 0 { 1 } else { hashes };
            format!(
                "  <para bookmark=\"Outline {}\">\n    {}\n  </para>\n",
                level,
                rich_to_fadein_text(rs)
            )
        }
        Element::Shot(rs) => {
            format!(
                "  <para bookmark=\"Shot\">\n    {}\n  </para>\n",
                rich_to_fadein_text(rs)
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

    let mut title_lines = Vec::new();
    for t in &titlepage.title {
        title_lines.push(format!("  <para align=\"center\">{}</para>\n", rich_to_fadein_text(t)));
    }
    for c in &titlepage.credit {
        title_lines.push(format!("  <para align=\"center\">{}</para>\n", rich_to_fadein_text(c)));
    }
    for a in &titlepage.authors {
        title_lines.push(format!("  <para align=\"center\">{}</para>\n", rich_to_fadein_text(a)));
    }
    for s in &titlepage.source {
        title_lines.push(format!("  <para align=\"center\">{}</para>\n", rich_to_fadein_text(s)));
    }
    for d in &titlepage.draft_date {
        title_lines.push(format!("  <para>{}</para>\n", rich_to_fadein_text(d)));
    }
    for c in &titlepage.contact {
        title_lines.push(format!("  <para>{}</para>\n", rich_to_fadein_text(c)));
    }

    format!("<titlepage>\n{}</titlepage>\n", title_lines.join(""))
}

pub fn export(screenplay: &Screenplay) -> String {
    let mut content = String::new();
    for span in &screenplay.elements {
        content.push_str(&element_to_fadein(&span.inner));
    }

    let title_page = match &screenplay.titlepage {
        Some(tp) => build_title_page(tp),
        None => String::new(),
    };

    OSF_TEMPLATE
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
        assert!(result.contains("<osf version=\"2.0\">"));
        assert!(result.contains("</osf>"));
    }

    #[test]
    fn exports_scene_heading() {
        let result = export(&parse("INT. HOUSE - DAY\n\nHello."));
        assert!(result.contains("bookmark=\"Scene Heading\""));
        assert!(result.contains("INT. HOUSE - DAY"));
        assert!(result.contains("bookmark=\"Action\""));
        assert!(result.contains("Hello."));
    }
}
