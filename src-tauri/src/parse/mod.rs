use html5ever::QualName;
use html5ever::{local_name, ns};
use kuchikikiki::traits::TendrilSink;
use kuchikikiki::{Attribute, Attributes, ElementData, ExpandedName, NodeRef, parse_fragment};
use std::cell::RefCell;

// HTML TTS Processing Rules:
//
// Input is always wrapped in a block-level container by process_html,
// so inline-only fragments at the top level do not occur.
//
// 1. All visible text must have a `tts_para` class for TTS segmentation.
// 2. No two `tts_para` spans may overlap — each unit of text belongs to
//    exactly one `tts_para_N`.
// 3. HTML structure is retained: block elements keep their tags, inline
//    elements (<strong>, <em>, etc.) get `tts_para_N` added as a class
//    for single-sentence content; multi-sentence content keeps the
//    inline wrapper and inserts `<span class="tts_para_N">` children.
//    For code/pre elements, each line (delimited by newline) is one
//    tts_para unit; semicolons and braces also split code lines.
// 4. Class ordering: `tts_para_N` is always prepended as the first class.
//    Other classes (`tts_code_block`, `tts_anchor`) are appended after.

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
            | "tbody"
            | "td"
            | "tfoot"
            | "th"
            | "thead"
            | "tr"
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

fn is_inline_formatting_tag(name: &str) -> bool {
    matches!(
        name,
        "strong"
            | "em"
            | "b"
            | "i"
            | "u"
            | "s"
            | "del"
            | "ins"
            | "sup"
            | "sub"
            | "small"
            | "mark"
            | "span"
            | "a"
            | "abbr"
            | "cite"
            | "q"
            | "kbd"
            | "var"
            | "ruby"
            | "rt"
            | "rp"
            | "time"
            | "data"
            | "label"
            | "br"
            | "wbr"
    )
}

fn has_only_inline_children(node: &NodeRef) -> bool {
    for child in node.children() {
        if let Some(element) = child.as_element() {
            let tag_name = element.name.local.as_ref();
            if !is_inline_formatting_tag(tag_name) {
                return false;
            }
            if !has_only_inline_children(&child) {
                return false;
            }
        }
    }
    true
}

fn is_real_code_block(node: &NodeRef) -> bool {
    let text_content = node.text_contents();
    if text_content.contains('\n') {
        return true;
    }
    if has_element_children(node) {
        return !has_only_inline_children(node);
    }
    false
}

fn has_element_children(node: &NodeRef) -> bool {
    node.children().any(|c| c.as_element().is_some())
}

fn is_leaf_block_element(name: &str) -> bool {
    matches!(name, "td" | "th" | "dd" | "dt")
}

fn is_single_path_to_text(item: &ContentItem) -> bool {
    match item {
        ContentItem::Text { .. } => false,
        ContentItem::Element { children, .. } => {
            children.len() == 1 && matches!(&children[0], ContentItem::Text { .. })
        }
    }
}

fn tag_innermost_text_element(node: &NodeRef, current_id: &RefCell<u32>) {
    if let Some(element) = node.as_element() {
        let has_elem_children = node.children().any(|c| c.as_element().is_some());
        if !has_elem_children {
            tag_element(element, current_id);
        } else if let Some(first_elem) = node.children().find(|c| c.as_element().is_some()) {
            tag_innermost_text_element(&first_elem, current_id);
        }
    }
}

fn flatten_block(node: &NodeRef, pos: &mut usize) -> Vec<ContentItem> {
    let mut items = Vec::new();

    for child in node.children() {
        if let Some(element) = child.as_element() {
            let tag_name = element.name.local.as_ref();

            if is_skip_tag(tag_name) {
                continue;
            }
            if is_code_tag(tag_name) && is_real_code_block(&child) {
                continue;
            }

            if is_leaf_block_element(tag_name) {
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

fn append_class(attrs: &mut Attributes, class: &str) {
    if let Some(existing) = attrs.get_mut("class") {
        existing.push(' ');
        existing.push_str(class);
    } else {
        attrs.insert("class", class.to_string());
    }
}

fn tag_element(element: &ElementData, current_id: &RefCell<u32>) {
    let id_val = {
        let mut id = current_id.borrow_mut();
        let val = *id;
        *id += 1;
        val
    };
    let class = format!("tts_para_{id_val}");
    let mut attrs = element.attributes.borrow_mut();
    if let Some(existing) = attrs.get_mut("class") {
        let old = existing.clone();
        *existing = format!("{class} {old}");
    } else {
        attrs.insert("class", class);
    }
}

fn make_tts_para_span(text: String, current_id: &RefCell<u32>) -> NodeRef {
    let span = NodeRef::new_element(QualName::new(None, ns!(html), local_name!("span")), vec![]);
    span.append(NodeRef::new_text(text));
    if let Some(element) = span.as_element() {
        tag_element(element, current_id);
    }
    span
}

fn process_node(node: &NodeRef, current_id: &RefCell<u32>) {
    if let Some(element) = node.as_element() {
        let tag_name = element.name.local.as_ref();

        if is_skip_tag(tag_name) {
            node.detach();
            return;
        }

        if is_code_tag(tag_name) && is_real_code_block(node) {
            process_code_element(node, current_id);
            return;
        }

        process_element_tts(node, current_id);
    } else if node.as_document().is_some() {
        for child in node.children() {
            process_node(&child, current_id);
        }
    }
}

fn process_element_tts(node: &NodeRef, current_id: &RefCell<u32>) {
    if let Some(element) = node.as_element() {
        let tag_name = element.name.local.as_ref();
        if is_code_tag(tag_name) && is_real_code_block(node) {
            append_class(&mut element.attributes.borrow_mut(), "tts_code_block");
        }
    }

    let children: Vec<NodeRef> = node.children().collect();
    let has_block_or_code_children = children.iter().any(|child| {
        if let Some(element) = child.as_element() {
            let tag_name = element.name.local.as_ref();
            is_block_element(tag_name) || (is_code_tag(tag_name) && is_real_code_block(child))
        } else {
            false
        }
    });

    if has_block_or_code_children {
        for child in &children {
            if child.as_element().is_some() {
                process_node(child, current_id);
            } else if let Some(text) = child.as_text() {
                let text_content = text.borrow().clone();
                if !text_content.trim().is_empty() {
                    let sentences = segment_sentences(&text_content, MAX_LENGTH);
                    if sentences.len() <= 1 {
                        let span = make_tts_para_span(text_content, current_id);
                        child.insert_before(span);
                        child.detach();
                    } else {
                        let text_chars: Vec<char> = text_content.chars().collect();
                        for (start, end) in sentences {
                            let sentence_text: String = text_chars[start..end].iter().collect();
                            let span = make_tts_para_span(sentence_text, current_id);
                            child.insert_before(span);
                        }
                        child.detach();
                    }
                }
            }
        }
        return;
    }

    let mut pos = 0;
    let items = flatten_block(node, &mut pos);
    let flat_text = build_flat_string(&items);

    if flat_text.trim().is_empty() {
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

        if clipped.len() == 1
            && is_single_path_to_text(&clipped[0])
            && let ContentItem::Element { .. } = &clipped[0]
        {
            let nodes = build_dom_from_items(&clipped);
            if let Some(elem) = nodes.first() {
                tag_innermost_text_element(elem, current_id);
                node.append(elem.clone());
                continue;
            }
        }

        let span =
            NodeRef::new_element(QualName::new(None, ns!(html), local_name!("span")), vec![]);
        if let Some(element) = span.as_element() {
            tag_element(element, current_id);
        }
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

        append_class(&mut element.attributes.borrow_mut(), "tts_code_block");

        if has_newlines {
            let units = split_code_block(&text_content);

            if units.len() > 1 {
                for child in node.children() {
                    child.detach();
                }

                for unit in units {
                    let span = make_tts_para_span(unit, current_id);
                    node.append(span);
                }
                return;
            }
        }

        tag_element(element, current_id);
    }
}

fn split_code_block(text: &str) -> Vec<String> {
    let mut units = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let line_count = lines.len();

    for (i, line) in lines.iter().enumerate() {
        if line.is_empty() {
            if i < line_count - 1 {
                units.push("\n".to_string());
            }
            continue;
        }

        let mut line_units = split_at_code_boundaries(line);
        if i < line_count - 1
            && let Some(last) = line_units.last_mut()
        {
            last.push('\n');
        }
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
                if !current.is_empty() {
                    units.push(current.clone());
                }
                current.clear();
            }
            '/' if i + 1 < len && chars[i + 1] == '/' => {
                let rest: String = chars[i..].iter().collect();
                current.push_str(&rest[1..]);
                if !current.is_empty() {
                    units.push(current.clone());
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
                if !current.is_empty() {
                    units.push(current.clone());
                }
                current.clear();
                continue;
            }
            _ => {}
        }

        i += 1;
    }

    if !current.is_empty() {
        units.push(current);
    }

    units
}

fn process_node_url(node: &NodeRef, url: &str) {
    let base_url = match url::Url::parse(url) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Invalid base URL '{url}': {e}");
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
                let is_fragment_only =
                    url_without_fragment == base_url && absolute_url.fragment().is_some();
                if is_fragment_only {
                    *href = format!("#{}", absolute_url.fragment().unwrap());
                } else {
                    *href = absolute_url.to_string();
                    append_class(&mut element, "tts_anchor");
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

#[must_use]
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
        }
        if bytes.ends_with(HTML_CLOSE) {
            let start = bytes.len() - HTML_CLOSE.len();
            bytes[start..].copy_from_slice(DIV_CLOSE);
        }
    }
    String::from_utf8(bytes).unwrap_or_else(|_| "<p>not valid utf8</p>".to_string())
}

#[must_use]
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

            if let Some(byte_pos) = body_lower.find(&q_lower) {
                let char_pos = body_lower[..byte_pos].chars().count();
                let snippet: String = body
                    .chars()
                    .skip(char_pos.saturating_sub(HALF_SNIPPET_LENGTH))
                    .take(SNIPPET_LENGTH)
                    .collect();

                let snippet_lower = snippet.to_lowercase();
                if let Some(byte_match_pos) = snippet_lower.find(&q_lower) {
                    let char_match_pos = snippet_lower[..byte_match_pos].chars().count();
                    let query_char_len = q_lower.chars().count();

                    let prefix: String = snippet.chars().take(char_match_pos).collect();
                    let match_text: String = snippet
                        .chars()
                        .skip(char_match_pos)
                        .take(query_char_len)
                        .collect();
                    let suffix: String = snippet
                        .chars()
                        .skip(char_match_pos + query_char_len)
                        .collect();

                    return Snippet {
                        prefix,
                        match_text: Some(match_text),
                        suffix: if suffix.is_empty() {
                            None
                        } else {
                            Some(suffix)
                        },
                    };
                }

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
mod test_parse;
