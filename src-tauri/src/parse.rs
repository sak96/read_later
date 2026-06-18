use html5ever::QualName;
use html5ever::{local_name, ns};
use kuchikikiki::traits::*;
use kuchikikiki::{parse_fragment, Attribute, ElementData, ExpandedName, NodeRef};
use std::cell::RefCell;

use crate::models::Snippet;

fn make_qual_name(tag: &str) -> QualName {
    QualName::new(None, ns!(html), tag.into())
}

const MAX_LENGTH: usize = 500;
const HTML_OPEN: &[u8] = b"<html>";
const HTML_CLOSE: &[u8] = b"</html>";
const DIV_OPEN: &[u8] = b"<div> ";
const DIV_CLOSE: &[u8] = b" </div>";

#[derive(Clone, Debug)]
enum ContentItem {
    Text {
        text: String,
        start: usize,
        end: usize,
    },
    Element {
        tag: String,
        attrs: Vec<(String, String)>,
        children: Vec<ContentItem>,
        start: usize,
        end: usize,
    },
}

fn is_block_element(name: &str) -> bool {
    matches!(
        name,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "canvas"
            | "dd"
            | "div"
            | "dl"
            | "dt"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hr"
            | "li"
            | "main"
            | "nav"
            | "noscript"
            | "ol"
            | "p"
            | "pre"
            | "section"
            | "table"
            | "tfoot"
            | "ul"
            | "video"
    )
}

fn is_skip_tag(name: &str) -> bool {
    matches!(
        name,
        "script" | "style" | "noscript" | "iframe" | "svg" | "head" | "link" | "meta"
    )
}

fn is_code_tag(name: &str) -> bool {
    matches!(name, "pre" | "code")
}

fn has_element_children(node: &NodeRef) -> bool {
    node.children().any(|c| c.as_element().is_some())
}

fn flatten_block(node: &NodeRef, pos: &mut usize) -> Vec<ContentItem> {
    let mut items = Vec::new();

    for child in node.children() {
        if let Some(element) = child.as_element() {
            let tag_name = element.name.local.as_ref();

            if is_skip_tag(tag_name) {
                continue;
            }
            if is_code_tag(tag_name) && has_element_children(&child) {
                continue;
            }

            if is_block_element(tag_name) {
                continue;
            }

            let attrs: Vec<(String, String)> = element
                .attributes
                .borrow()
                .map
                .iter()
                .map(|(k, v)| (k.local.to_string(), v.value.clone()))
                .collect();

            let start = *pos;
            let children = flatten_block(&child, pos);
            let end = *pos;

            items.push(ContentItem::Element {
                tag: tag_name.to_string(),
                attrs,
                children,
                start,
                end,
            });
        } else if let Some(text) = child.as_text() {
            let text_content = text.borrow().clone();
            let len = text_content.chars().count();

            if len > 0 {
                items.push(ContentItem::Text {
                    text: text_content,
                    start: *pos,
                    end: *pos + len,
                });
                *pos += len;
            }
        }
    }

    items
}

fn build_flat_string(items: &[ContentItem]) -> String {
    let mut result = String::new();

    for item in items {
        match item {
            ContentItem::Text { text, .. } => {
                result.push_str(text);
            }
            ContentItem::Element { children, .. } => {
                result.push_str(&build_flat_string(children));
            }
        }
    }

    result
}

fn segment_sentences(text: &str, max_len: usize) -> Vec<(usize, usize)> {
    let mut sentences = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    if len == 0 {
        return sentences;
    }

    let mut start = 0;
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        if ch == '.' || ch == '!' || ch == '?' || ch == ';' {
            if ch == '.' && i + 1 < len && chars[i + 1].is_lowercase() {
                i += 1;
                continue;
            }

            if ch == '.' && i + 1 < len && chars[i + 1].is_whitespace() {
                let mut word_len = 0;
                let mut j = i;
                while j > 0 && !chars[j - 1].is_whitespace() && chars[j - 1] != '.' {
                    word_len += 1;
                    j -= 1;
                }
                if word_len <= 2 {
                    i += 1;
                    continue;
                }
            }

            if i + 1 >= len || chars[i + 1].is_whitespace() {
                let end = i + 1;

                if end > start {
                    let sentence_text: String = chars[start..end].iter().collect();
                    if !sentence_text.trim().is_empty() {
                        sentences.push((start, end));
                    }
                }

                start = end;
                i = end;
                continue;
            }
        }

        i += 1;
    }

    if start < len {
        let sentence_text: String = chars[start..].iter().collect();
        if !sentence_text.trim().is_empty() {
            sentences.push((start, len));
        }
    }

    let mut final_sentences = Vec::new();
    for (start, end) in sentences {
        let sentence_len = end - start;
        if sentence_len <= max_len {
            final_sentences.push((start, end));
        } else {
            let sentence_text: String = chars[start..end].iter().collect();
            let sub_sentences = split_long_sentence(&sentence_text, start, max_len);
            final_sentences.extend(sub_sentences);
        }
    }

    final_sentences
}

fn split_long_sentence(text: &str, base_start: usize, max_len: usize) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    let mut start = 0;
    let mut last_break = 0;

    for i in 0..len {
        let ch = chars[i];

        if ch == ',' || ch == ';' || ch == ':' || ch.is_whitespace() {
            last_break = i;
        }

        if i - start >= max_len && last_break > start {
            let break_text: String = chars[start..last_break].iter().collect();
            if !break_text.trim().is_empty() {
                result.push((base_start + start, base_start + last_break));
            }
            start = last_break + 1;
            last_break = start;
        }
    }

    if start < len {
        let remaining: String = chars[start..].iter().collect();
        if !remaining.trim().is_empty() {
            result.push((base_start + start, base_start + len));
        }
    }

    result
}

fn clip_items(items: &[ContentItem], start: usize, end: usize) -> Vec<ContentItem> {
    let mut result = Vec::new();

    for item in items {
        match item {
            ContentItem::Text {
                text,
                start: item_start,
                end: item_end,
            } => {
                if *item_end <= start || *item_start >= end {
                    continue;
                }

                let vis_start = start.max(*item_start);
                let vis_end = end.min(*item_end);

                let text_chars: Vec<char> = text.chars().collect();
                let offset_start = vis_start - *item_start;
                let offset_end = vis_end - *item_start;
                let visible_text: String = text_chars[offset_start..offset_end].iter().collect();

                if !visible_text.is_empty() {
                    result.push(ContentItem::Text {
                        text: visible_text,
                        start: vis_start,
                        end: vis_end,
                    });
                }
            }
            ContentItem::Element {
                tag,
                attrs,
                children,
                start: item_start,
                end: item_end,
            } => {
                if *item_end <= start || *item_start >= end {
                    continue;
                }

                let clipped_children = clip_items(children, start, end);

                if !clipped_children.is_empty() {
                    result.push(ContentItem::Element {
                        tag: tag.clone(),
                        attrs: attrs.clone(),
                        children: clipped_children,
                        start: (*item_start).max(start),
                        end: (*item_end).min(end),
                    });
                }
            }
        }
    }

    result
}

fn build_dom_from_items(items: &[ContentItem]) -> Vec<NodeRef> {
    let mut nodes = Vec::new();

    for item in items {
        match item {
            ContentItem::Text { text, .. } => {
                if !text.is_empty() {
                    nodes.push(NodeRef::new_text(text.clone()));
                }
            }
            ContentItem::Element {
                tag,
                attrs,
                children,
                ..
            } => {
                let qual_name = make_qual_name(tag);
                let kuchiki_attrs: Vec<(ExpandedName, Attribute)> = attrs
                    .iter()
                    .map(|(k, v)| {
                        (
                            ExpandedName::new(ns!(), k.as_str()),
                            Attribute {
                                prefix: None,
                                value: v.clone(),
                            },
                        )
                    })
                    .collect();
                let elem = NodeRef::new_element(qual_name, kuchiki_attrs);

                let child_nodes = build_dom_from_items(children);
                for child in child_nodes {
                    elem.append(child);
                }

                nodes.push(elem);
            }
        }
    }

    nodes
}

fn tag_element(element: &ElementData, current_id: &RefCell<u32>) {
    let id_val = {
        let mut id = current_id.borrow_mut();
        let val = *id;
        *id += 1;
        val
    };
    element
        .attributes
        .borrow_mut()
        .insert("class", format!("tts_para_{}", id_val));
}

fn process_node(node: &NodeRef, current_id: &RefCell<u32>) {
    if let Some(element) = node.as_element() {
        let tag_name = element.name.local.as_ref();

        if is_skip_tag(tag_name) {
            node.detach();
            return;
        }

        if is_code_tag(tag_name) && has_element_children(node) {
            process_code_element(node, current_id);
            return;
        }

        if is_block_element(tag_name) {
            process_block_element(node, current_id);
            return;
        }

        let children: Vec<NodeRef> = node.children().collect();
        for child in children {
            process_node(&child, current_id);
        }
    } else if node.as_document().is_some() {
        for child in node.children() {
            process_node(&child, current_id);
        }
    }
}

fn process_block_element(node: &NodeRef, current_id: &RefCell<u32>) {
    let children: Vec<NodeRef> = node.children().collect();
    for child in &children {
        if let Some(element) = child.as_element() {
            let tag_name = element.name.local.as_ref();
            if is_code_tag(tag_name) && has_element_children(child) {
                process_code_element(child, current_id);
            } else if is_block_element(tag_name) {
                process_node(child, current_id);
            }
        }
    }

    let has_block_or_code_children = children.iter().any(|child| {
        if let Some(element) = child.as_element() {
            let tag_name = element.name.local.as_ref();
            is_block_element(tag_name) || (is_code_tag(tag_name) && has_element_children(child))
        } else {
            false
        }
    });

    if has_block_or_code_children {
        for child in &children {
            if let Some(element) = child.as_element() {
                let tag_name = element.name.local.as_ref();
                if !is_block_element(tag_name)
                    && !(is_code_tag(tag_name) && has_element_children(child))
                {
                    process_node(child, current_id);
                }
            }
        }
        return;
    }

    let mut pos = 0;
    let items = flatten_block(node, &mut pos);
    let flat_text = build_flat_string(&items);

    if flat_text.trim().is_empty() {
        if let Some(element) = node.as_element() {
            tag_element(element, current_id);
        }
        return;
    }

    let sentences = segment_sentences(&flat_text, MAX_LENGTH);

    if sentences.len() <= 1 {
        if let Some(element) = node.as_element() {
            tag_element(element, current_id);
        }
        return;
    }

    for child in node.children() {
        child.detach();
    }

    for (start, end) in sentences {
        let clipped = clip_items(&items, start, end);
        let id_val = {
            let mut id = current_id.borrow_mut();
            let val = *id;
            *id += 1;
            val
        };

        let span = NodeRef::new_element(
            QualName::new(None, ns!(html), local_name!("span")),
            vec![(
                ExpandedName::new(ns!(), "class"),
                Attribute {
                    prefix: None,
                    value: format!("tts_para_{}", id_val),
                },
            )],
        );

        let child_nodes = build_dom_from_items(&clipped);
        for child in child_nodes {
            span.append(child);
        }

        node.append(span);
    }
}

fn process_code_element(node: &NodeRef, current_id: &RefCell<u32>) {
    if let Some(element) = node.as_element() {
        let text_content = node.text_contents();
        let has_newlines = text_content.contains('\n');

        {
            let mut attrs = element.attributes.borrow_mut();
            attrs.insert("class", "tts_code_block".to_string());
        }

        if has_newlines {
            let units = split_code_block(&text_content);

            if units.len() > 1 {
                for child in node.children() {
                    child.detach();
                }

                for unit in units {
                    let id_val = {
                        let mut id = current_id.borrow_mut();
                        let val = *id;
                        *id += 1;
                        val
                    };

                    let span = NodeRef::new_element(
                        QualName::new(None, ns!(html), local_name!("span")),
                        vec![(
                            ExpandedName::new(ns!(), "class"),
                            Attribute {
                                prefix: None,
                                value: format!("tts_para_{}", id_val),
                            },
                        )],
                    );
                    span.append(NodeRef::new_text(unit));
                    node.append(span);
                }
            }
        } else {
            let id_val = {
                let mut id = current_id.borrow_mut();
                let val = *id;
                *id += 1;
                val
            };
            let mut attrs = element.attributes.borrow_mut();
            attrs.insert("class", format!("tts_code_block tts_para_{}", id_val));
        }
    }
}

fn split_code_block(text: &str) -> Vec<String> {
    let mut units = Vec::new();

    for line in text.lines() {
        let trimmed = line.to_string();
        if trimmed.trim().is_empty() {
            continue;
        }

        let line_units = split_at_code_boundaries(&trimmed);
        units.extend(line_units);
    }

    units
}

fn split_at_code_boundaries(line: &str) -> Vec<String> {
    let mut units = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];
        current.push(ch);

        match ch {
            ';' | '{' | '}' => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    units.push(trimmed);
                }
                current.clear();
            }
            '/' if i + 1 < len && chars[i + 1] == '/' => {
                let rest: String = chars[i..].iter().collect();
                current.push_str(&rest[1..]);
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    units.push(trimmed);
                }
                current.clear();
                break;
            }
            '/' if i + 1 < len && chars[i + 1] == '*' => {
                current.push('*');
                i += 2;
                while i < len {
                    let c = chars[i];
                    current.push(c);
                    if c == '*' && i + 1 < len && chars[i + 1] == '/' {
                        current.push('/');
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    units.push(trimmed);
                }
                current.clear();
                continue;
            }
            _ => {}
        }

        i += 1;
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        units.push(trimmed);
    }

    units
}

fn process_node_url(node: &NodeRef, url: &str) {
    let base_url = match url::Url::parse(url) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Invalid base URL '{}': {}", url, e);
            return;
        }
    };

    for anchor in node.select("a").unwrap() {
        let mut element = anchor.attributes.borrow_mut();

        if let Some(href) = element.get_mut("href") {
            let original_href = href.clone();

            if let Ok(absolute_url) = base_url.join(&original_href) {
                let mut url_without_fragment = absolute_url.clone();
                url_without_fragment.set_fragment(None);
                let is_fragement_only =
                    url_without_fragment == base_url && absolute_url.fragment().is_some();
                if is_fragement_only {
                    *href = format!("#{}", absolute_url.fragment().unwrap());
                } else {
                    *href = absolute_url.to_string();
                    if let Some(class) = element.get_mut("class") {
                        if !class.contains("tts_anchor") {
                            class.push_str(" tts_anchor");
                        }
                    } else {
                        element.insert("class", "tts_anchor".to_string());
                    }
                }
            }
        }
    }

    for img in node.select("img").unwrap() {
        let mut element = img.attributes.borrow_mut();

        if let Some(src) = element.get_mut("src") {
            let original_src = src.clone();

            if let Ok(absolute_url) = base_url.join(&original_src) {
                *src = absolute_url.to_string();
            }
        }
    }
}

pub fn process_html(frag: &str, url: &str) -> String {
    let ctx_name = QualName::new(None, ns!(html), local_name!("article"));
    let document = parse_fragment(ctx_name, vec![]).one(frag);
    let current_id = RefCell::new(0);

    process_node(&document, &current_id);
    process_node_url(&document, url);

    let mut bytes = Vec::new();
    document.serialize(&mut bytes).unwrap();
    {
        let bytes = bytes.as_mut_slice();
        if bytes.starts_with(HTML_OPEN) {
            bytes[..HTML_OPEN.len()].copy_from_slice(DIV_OPEN);
        };
        if bytes.ends_with(HTML_CLOSE) {
            let start = bytes.len() - HTML_CLOSE.len();
            bytes[start..].copy_from_slice(DIV_CLOSE);
        };
    }
    String::from_utf8(bytes).unwrap_or_else(|_| "<p>not valid utf8</p>".to_string())
}

pub fn build_snippet(body: &str, query: Option<&str>) -> Snippet {
    const SNIPPET_LENGTH: usize = 100;
    const HALF_SNIPPET_LENGTH: usize = SNIPPET_LENGTH / 2;
    match query {
        None => Snippet {
            prefix: body.chars().take(SNIPPET_LENGTH).collect(),
            match_text: None,
            suffix: None,
        },
        Some(q) => {
            let body_lower = body.to_lowercase();
            let q_lower = q.to_lowercase();

            if let Some(pos) = body_lower.find(&q_lower) {
                let snippet: String = body
                    .chars()
                    .skip(pos.saturating_sub(HALF_SNIPPET_LENGTH))
                    .take(SNIPPET_LENGTH)
                    .collect();

                return Snippet {
                    prefix: snippet,
                    match_text: None,
                    suffix: None,
                };
            }

            Snippet {
                prefix: body.chars().take(SNIPPET_LENGTH).collect(),
                match_text: None,
                suffix: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process_html_test(frag: &str) -> String {
        process_html(frag, "https://example.com")
    }

    fn has_class(html: &str, class: &str) -> bool {
        html.contains(&format!("class=\"{}\"", class))
            || html.contains(&format!("class=\"{} ", class))
            || html.contains(&format!(" {}\"", class))
            || html.contains(&format!(" {} ", class))
    }

    #[test]
    fn test_simple_sentences() {
        let input = "<p>Call me Ishmael. Some years ago I thought I would sail.</p>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_para_0"),
            "should have tts_para_0: {}",
            output
        );
        assert!(
            has_class(&output, "tts_para_1"),
            "should have tts_para_1: {}",
            output
        );
        assert!(
            output.contains("Call me Ishmael."),
            "should contain first sentence: {}",
            output
        );
        assert!(
            output.contains("Some years ago I thought I would sail."),
            "should contain second sentence: {}",
            output
        );
    }

    #[test]
    fn test_inline_elements_preserved() {
        let input = "<p>This is a sentence with <em>emphasis. And it continues</em> into the next sentence!</p>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_para_0"),
            "should have tts_para_0: {}",
            output
        );
        assert!(
            has_class(&output, "tts_para_1"),
            "should have tts_para_1: {}",
            output
        );
        assert!(
            output.contains("<em>"),
            "should preserve em element: {}",
            output
        );
    }

    #[test]
    fn test_multiple_inline_elements() {
        let input =
            "<p>First sentence <strong>with bold. Second sentence</strong> without bold.</p>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_para_0"),
            "should have tts_para_0: {}",
            output
        );
        assert!(
            has_class(&output, "tts_para_1"),
            "should have tts_para_1: {}",
            output
        );
        assert!(
            output.contains("<strong>"),
            "should preserve strong element: {}",
            output
        );
    }

    #[test]
    fn test_abbreviations_not_split() {
        let input = "<p>Dr. Smith went to the U.S.A. and met Mr. Jones. Then he left.</p>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_para_0"),
            "should have tts_para_0: {}",
            output
        );
        assert!(
            has_class(&output, "tts_para_1"),
            "should have tts_para_1: {}",
            output
        );
        assert!(
            output.contains("Dr. Smith went to the U.S.A. and met Mr. Jones."),
            "should not split at abbreviations: {}",
            output
        );
        assert!(
            output.contains("Then he left."),
            "should have second sentence: {}",
            output
        );
    }

    #[test]
    fn test_skip_tags_removed() {
        let input = "<p>Visible text.</p><script>alert('hidden');</script><p>More visible.</p>";
        let output = process_html_test(input);
        assert!(
            !output.contains("alert"),
            "script should be removed: {}",
            output
        );
        assert!(
            output.contains("Visible text."),
            "should contain first sentence: {}",
            output
        );
        assert!(
            output.contains("More visible."),
            "should contain second sentence: {}",
            output
        );
    }

    #[test]
    fn test_code_block_with_newlines() {
        let input = "<pre><code>function hello() {\n  console.log(\"world\");\n  return true;\n}</code></pre>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_code_block"),
            "should have tts_code_block: {}",
            output
        );
        assert!(
            has_class(&output, "tts_para_0"),
            "should have tts_para_0: {}",
            output
        );
        assert!(
            has_class(&output, "tts_para_1"),
            "should have tts_para_1: {}",
            output
        );
    }

    #[test]
    fn test_code_inline_no_element_children_flattened() {
        let input = "<p>Here is code: <code>let x = 1;</code></p>";
        let output = process_html_test(input);
        assert!(
            !has_class(&output, "tts_code_block"),
            "should NOT have tts_code_block for inline code without element children: {}",
            output
        );
        assert!(
            output.contains("let x = 1;"),
            "should preserve code content: {}",
            output
        );
    }

    #[test]
    fn test_semicolon_boundary() {
        let input = "<p>First clause; second clause; third clause.</p>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_para_0"),
            "should have tts_para_0: {}",
            output
        );
        assert!(
            has_class(&output, "tts_para_1"),
            "should have tts_para_1: {}",
            output
        );
        assert!(
            has_class(&output, "tts_para_2"),
            "should have tts_para_2: {}",
            output
        );
    }

    #[test]
    fn test_no_punctuation_single_sentence() {
        let input = "<div>Just some text without any punctuation marks</div>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_para_0"),
            "should have tts_para_0: {}",
            output
        );
        assert!(
            output.contains("Just some text without any punctuation marks"),
            "should contain the text: {}",
            output
        );
    }

    #[test]
    fn test_long_sentence_split() {
        let text = "word ".repeat(200);
        let input = format!("<p>{}</p>", text);
        let output = process_html_test(&input);
        assert!(
            has_class(&output, "tts_para_0"),
            "should have tts_para_0: {}",
            output
        );
        assert!(
            has_class(&output, "tts_para_1"),
            "should have tts_para_1: {}",
            output
        );
    }

    #[test]
    fn test_empty_paragraph() {
        let input = "<p></p>";
        let output = process_html_test(input);
        assert!(
            output.contains("<p"),
            "should preserve empty paragraph: {}",
            output
        );
    }

    #[test]
    fn test_anchor_url_resolution() {
        let input = "<p>Visit <a href=\"/about\">our page</a>.</p>";
        let output = process_html_test(input);
        assert!(
            output.contains("https://example.com/about"),
            "should resolve relative URL: {}",
            output
        );
        assert!(
            has_class(&output, "tts_anchor"),
            "should have tts_anchor class: {}",
            output
        );
    }

    #[test]
    fn test_img_url_resolution() {
        let input = "<p><img src=\"/image.png\" alt=\"test\"></p>";
        let output = process_html_test(input);
        assert!(
            output.contains("https://example.com/image.png"),
            "should resolve img src: {}",
            output
        );
    }

    #[test]
    fn test_blockquote_sentences() {
        let input = "<blockquote>First quote. Second quote.</blockquote>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_para_0"),
            "should have tts_para_0: {}",
            output
        );
        assert!(
            has_class(&output, "tts_para_1"),
            "should have tts_para_1: {}",
            output
        );
    }

    #[test]
    fn test_list_items() {
        let input = "<ul><li>First item. Second sentence.</li><li>Another item.</li></ul>";
        let output = process_html_test(input);
        assert!(
            output.contains("tts_para_"),
            "should have tts_para classes: {}",
            output
        );
    }

    #[test]
    fn test_heading_sentences() {
        let input = "<h1>Title here. Subtitle here.</h1>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_para_0"),
            "should have tts_para_0: {}",
            output
        );
        assert!(
            has_class(&output, "tts_para_1"),
            "should have tts_para_1: {}",
            output
        );
    }

    #[test]
    fn test_nested_inline_elements() {
        let input = "<p>Text <em>with <strong>bold. More</strong> text</em> here.</p>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_para_0"),
            "should have tts_para_0: {}",
            output
        );
        assert!(
            has_class(&output, "tts_para_1"),
            "should have tts_para_1: {}",
            output
        );
        assert!(output.contains("<em>"), "should preserve em: {}", output);
        assert!(
            output.contains("<strong>"),
            "should preserve strong: {}",
            output
        );
    }

    #[test]
    fn test_code_block_no_newlines_no_element_children() {
        let input = "<pre><code>let a = 1; let b = 2; let c = 3;</code></pre>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_code_block"),
            "should have tts_code_block: {}",
            output
        );
        assert!(
            has_class(&output, "tts_para_0"),
            "pre should get tts_para_0: {}",
            output
        );
    }

    #[test]
    fn test_code_block_split_at_braces() {
        let input = "<pre><code>if (true) {\n  doSomething();\n}</code></pre>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_code_block"),
            "should have tts_code_block: {}",
            output
        );
        assert!(
            has_class(&output, "tts_para_0"),
            "should have tts_para_0: {}",
            output
        );
    }

    #[test]
    fn test_code_block_split_at_line_comment() {
        let input = "<pre><code>let x = 1; // this is a comment\nlet y = 2;</code></pre>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_code_block"),
            "should have tts_code_block: {}",
            output
        );
    }

    #[test]
    fn test_multiple_block_elements() {
        let input =
            "<h1>Title.</h1><p>First paragraph. Second sentence.</p><p>Another paragraph.</p>";
        let output = process_html_test(input);
        assert!(
            output.contains("tts_para_"),
            "should have tts_para classes: {}",
            output
        );
        assert!(
            output.contains("Title."),
            "should contain title: {}",
            output
        );
        assert!(
            output.contains("First paragraph."),
            "should contain first paragraph: {}",
            output
        );
        assert!(
            output.contains("Another paragraph."),
            "should contain another paragraph: {}",
            output
        );
    }

    #[test]
    fn test_exclamation_mark_boundary() {
        let input = "<p>Hello world! How are you? I am fine.</p>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_para_0"),
            "should have tts_para_0: {}",
            output
        );
        assert!(
            has_class(&output, "tts_para_1"),
            "should have tts_para_1: {}",
            output
        );
        assert!(
            has_class(&output, "tts_para_2"),
            "should have tts_para_2: {}",
            output
        );
    }

    #[test]
    fn test_question_mark_boundary() {
        let input = "<p>What is this? It is a test.</p>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_para_0"),
            "should have tts_para_0: {}",
            output
        );
        assert!(
            has_class(&output, "tts_para_1"),
            "should have tts_para_1: {}",
            output
        );
    }

    #[test]
    fn test_whitespace_only_paragraph() {
        let input = "<p>   </p>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_para_0"),
            "should have tts_para_0: {}",
            output
        );
    }

    #[test]
    fn test_mixed_content_with_divs() {
        let input = "<div><p>Paragraph one.</p><p>Paragraph two.</p></div>";
        let output = process_html_test(input);
        assert!(
            output.contains("Paragraph one."),
            "should contain paragraph one: {}",
            output
        );
        assert!(
            output.contains("Paragraph two."),
            "should contain paragraph two: {}",
            output
        );
    }

    #[test]
    fn test_sentence_boundary_with_comma() {
        let input = "<p>This is a long sentence, and it continues here, and ends here.</p>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_para_0"),
            "should have tts_para_0: {}",
            output
        );
    }

    #[test]
    fn test_div_without_punctuation() {
        let input = "<div>Some text without punctuation</div>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_para_0"),
            "should have tts_para_0: {}",
            output
        );
        assert!(
            output.contains("Some text without punctuation"),
            "should contain text: {}",
            output
        );
    }

    #[test]
    fn test_preserves_html_structure() {
        let input = "<article><h1>Title.</h1><p>Content here.</p></article>";
        let output = process_html_test(input);
        assert!(
            output.contains("<article"),
            "should preserve article: {}",
            output
        );
        assert!(output.contains("<h1"), "should preserve h1: {}", output);
        assert!(output.contains("<p"), "should preserve p: {}", output);
    }

    #[test]
    fn test_link_inside_sentence() {
        let input = "<p>Click <a href=\"/page\">here</a> for more info. Thanks.</p>";
        let output = process_html_test(input);
        assert!(
            has_class(&output, "tts_para_0"),
            "should have tts_para_0: {}",
            output
        );
        assert!(
            has_class(&output, "tts_para_1"),
            "should have tts_para_1: {}",
            output
        );
        assert!(output.contains("<a"), "should preserve anchor: {}", output);
    }

    #[test]
    fn test_fragment_only_url() {
        let input = "<p><a href=\"#section1\">Go to section</a></p>";
        let output = process_html_test(input);
        assert!(
            output.contains("#section1"),
            "should keep fragment-only URL: {}",
            output
        );
        assert!(
            !has_class(&output, "tts_anchor"),
            "fragment-only should not have tts_anchor: {}",
            output
        );
    }
}
