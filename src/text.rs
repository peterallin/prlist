use std::mem;

#[derive(Debug, PartialEq, Eq)]
pub enum TextElement {
    Paragraph(String),
    ListEntry(String),
}

pub fn parse(raw: &str) -> Vec<TextElement> {
    enum State {
        Init,
        InParagraph { text: String, last: char },
        InListEntry { text: String, text_started: bool },
    }

    let mut state = State::Init;
    let mut result = vec![];
    for c in raw.chars() {
        match state {
            State::Init => match c {
                '\n' | ' ' => {}
                '-' | '*' => {
                    state = State::InListEntry {
                        text: String::new(),
                        text_started: false,
                    }
                }
                _ => {
                    state = State::InParagraph {
                        text: c.into(),
                        last: c,
                    }
                }
            },
            State::InParagraph {
                text: ref mut s,
                ref mut last,
            } => match c {
                '\n' if *last == '\n' => {
                    result.push(TextElement::Paragraph(mem::take(s)));
                    state = State::Init;
                }
                '\n' => {
                    *last = '\n';
                }
                ' ' if *last == '\n' => {}
                '-' | '*' if *last == '\n' => {
                    result.push(TextElement::Paragraph(mem::take(s)));
                    state = State::InListEntry {
                        text: String::new(),
                        text_started: false,
                    }
                }
                _ => {
                    if *last == '\n' {
                        s.push(' ');
                    }
                    s.push(c);
                    *last = c;
                }
            },
            State::InListEntry {
                ref mut text,
                ref mut text_started,
            } => {
                match c {
                    '\n' => {
                        result.push(TextElement::ListEntry(mem::take(text)));
                        state = State::Init;
                    }
                    _ if *text_started => {
                        text.push(c);
                    }
                    _ if c.is_whitespace() => {}
                    _ => {
                        *text_started = true;
                        text.push(c);
                    }
                };
            }
        }
    }
    match state {
        State::Init => {}
        State::InParagraph { text, .. } => result.push(TextElement::Paragraph(text)),
        State::InListEntry { text, .. } => result.push(TextElement::ListEntry(text)),
    }

    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn single_line_is_a_single_paragraph() {
        let input = "blah blah blah blah blah blah blah blah blah";
        let result = parse(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], TextElement::Paragraph(input.into()));
    }

    #[test]
    fn newlines_does_not_introduce_paragraphs() {
        let line1 = "line1 line1 line1 line1 line1 line1 line1 line1";
        let line2 = "line2 line2 line2 line2 line2 line2 line2 line2";
        let line3 = "line3 line3 line3 line3 line3 line3 line3 line3";
        let input = format!("{line1}\n{line2}\n{line3}\n");
        let result = dbg!(parse(&input));
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            TextElement::Paragraph(format!("{line1} {line2} {line3}"))
        )
    }

    #[test]
    fn double_newlines_gives_paragraphs() {
        let para1 = "para1 para1 para1\npara1 para1 para1\npara1 para1";
        let para2 = "para2 para2 para2 para2\npara2 para2 para2\npara2";
        let para3 = "para3 para3\npara3 para3 para3\npara3 para3 para3";
        let input = format!("{para1}\n\n{para2}\n\n{para3}\n\n");
        let result = dbg!(parse(&input));
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], TextElement::Paragraph(para1.replace("\n", " ")));
        assert_eq!(result[1], TextElement::Paragraph(para2.replace("\n", " ")));
        assert_eq!(result[2], TextElement::Paragraph(para3.replace("\n", " ")));
    }

    #[test]
    fn dash_at_line_start_gives_list_elements() {
        let elem1 = "- elem1 elem1 elem1";
        let elem2 = "- elem2 elem2 elem2";
        let elem3 = "- elem3 elem3 elem3";
        let input = format!("{elem1}\n{elem2}\n{elem3}\n");
        let result = dbg!(parse(&input));
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], TextElement::ListEntry(elem1[2..].into()));
        assert_eq!(result[1], TextElement::ListEntry(elem2[2..].into()));
        assert_eq!(result[2], TextElement::ListEntry(elem3[2..].into()));
    }

    #[test]
    fn strips_whitespace_around_list_entry_start() {
        let elem1 = " -  elem1 elem1 elem1";
        let elem2 = " -  elem2 elem2 elem2";
        let elem3 = " -  elem3 elem3 elem3";
        let input = format!("{elem1}\n{elem2}\n{elem3}\n");
        let result = dbg!(parse(&input));
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], TextElement::ListEntry(elem1[4..].into()));
        assert_eq!(result[1], TextElement::ListEntry(elem2[4..].into()));
        assert_eq!(result[2], TextElement::ListEntry(elem3[4..].into()));
    }

    #[test]
    fn asterisk_can_be_used_in_place_of_dash() {
        let elem1 = " *  elem1 elem1 elem1";
        let elem2 = " *  elem2 elem2 elem2";
        let elem3 = " *  elem3 elem3 elem3";
        let input = format!("{elem1}\n{elem2}\n{elem3}\n");
        let result = dbg!(parse(&input));
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], TextElement::ListEntry(elem1[4..].into()));
        assert_eq!(result[1], TextElement::ListEntry(elem2[4..].into()));
        assert_eq!(result[2], TextElement::ListEntry(elem3[4..].into()));
    }

    #[test]
    fn can_combine_paragraphs_and_list_items() {
        let input = r#"
This is a paragraph before a list item. This is a paragraph
before a list item. This is a paragraph before a list item.
* item1
* item2
* item3
This is a paragraph after a list item."#;
        let result = dbg!(parse(&input));
        assert_eq!(result.len(), 5);
        assert_eq!(result[0], TextElement::Paragraph("This is a paragraph before a list item. This is a paragraph before a list item. This is a paragraph before a list item.".into()));
        assert_eq!(result[1], TextElement::ListEntry("item1".into()));
        assert_eq!(result[2], TextElement::ListEntry("item2".into()));
        assert_eq!(result[3], TextElement::ListEntry("item3".into()));
        assert_eq!(
            result[4],
            TextElement::Paragraph("This is a paragraph after a list item.".into())
        );
    }

    #[test]
    fn can_combine_paragraphs_and_list_items_extra_lines() {
        let input = r#"
This is a paragraph before a list item. This is a paragraph
before a list item. This is a paragraph before a list item.


* item1
* item2
* item3


This is a paragraph after a list item."#;
        let result = dbg!(parse(&input));
        assert_eq!(result.len(), 5);
        assert_eq!(result[0], TextElement::Paragraph("This is a paragraph before a list item. This is a paragraph before a list item. This is a paragraph before a list item.".into()));
        assert_eq!(result[1], TextElement::ListEntry("item1".into()));
        assert_eq!(result[2], TextElement::ListEntry("item2".into()));
        assert_eq!(result[3], TextElement::ListEntry("item3".into()));
        assert_eq!(
            result[4],
            TextElement::Paragraph("This is a paragraph after a list item.".into())
        );
    }
}
