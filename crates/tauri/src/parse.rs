use html5ever::QualName;
use html5ever::{local_name, ns};
use kuchikikiki::traits::*;
use kuchikikiki::{ElementData, NodeRef, parse_html};
use std::cell::RefCell;

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

const MAX_LENGTH: usize = 500;

pub fn process_html(html: &str) -> String {
    let document = parse_html().one(html);
    let current_id = RefCell::new(0);

    process_node(&document, &current_id);

    let mut bytes = Vec::new();
    document.serialize(&mut bytes).unwrap();
    String::from_utf8(bytes).unwrap()
}

fn process_node(node: &NodeRef, current_id: &RefCell<u32>) {
    // We only process Elements (and Document root recursively)
    if let Some(element) = node.as_element() {
        let tag_name = element.name.local.as_ref();

        // Check Skip Tags
        if is_skip_tag(tag_name) {
            return;
        }

        // 6. For code just add para and don't recurse
        if is_code_tag(tag_name) {
            tag_element(element, current_id);
            process_code_elements(node);
            return;
        }

        let text_content = node.text_contents();
        let text_len = text_content.chars().count();
        let is_block = is_block_element(tag_name);

        // Check if has mixed content (non-block elements or raw text)
        let has_mixed_child = node.children().any(|child| {
            if let Some(el) = child.as_element() {
                !is_block_element(el.name.local.as_ref())
            } else {
                child.as_text().is_some() && !child.text_contents().trim().is_empty()
            }
        });

        // 3. Tag elements which meet criteria
        let criteria_met = text_len < MAX_LENGTH && is_block && has_mixed_child;

        if criteria_met {
            tag_element(element, current_id);
            process_code_elements(node);
            // If we tagged the block, we usually stop recursion for this branch
            // as it is now a distinct "paragraph".
            return;
        }

        // 5. If tag doesn't have child elements (only text) and didn't meet criteria (too long),
        // break text into spans
        let has_element_children = node.children().any(|c| c.as_element().is_some());

        if !has_element_children && text_len > 0 {
            // It is a text-only container but was too long or failed other criteria.
            // We clear current text and replace with grouped spans.
            split_text_content(node, &text_content, current_id);
            return;
        }

        // 4. If tag doesn't meet criteria recursively try thing.
        // "note text without tags needs to be first put into a span."

        // We collect children first to avoid concurrent modification issues while iterating
        let children: Vec<NodeRef> = node.children().collect();

        for child in children {
            if let Some(text_node) = child.as_text() {
                let text = text_node.borrow();
                if !text.trim().is_empty() {
                    // Wrap text node in span
                    let new_span = NodeRef::new_element(
                        QualName::new(None, ns!(html), local_name!("span")),
                        vec![],
                    );
                    new_span.append(NodeRef::new_text(text.clone()));

                    // Replace the naked text node with the new span in the DOM
                    child.insert_after(new_span.clone());
                    child.detach();

                    // Recurse on the new span
                    process_node(&new_span, current_id);
                }
            } else if child.as_element().is_some() {
                process_node(&child, current_id);
            }
        }
    } else if node.as_document().is_some() {
        for child in node.children() {
            process_node(&child, current_id);
        }
    }
}

// Helper to add the class
fn tag_element(element: &ElementData, current_id: &RefCell<u32>) {
    let mut id_val = current_id.borrow_mut();
    element
        .attributes
        .borrow_mut()
        .insert("class", format!("tts_para_{}", *id_val));
    *id_val += 1;
}

fn process_code_elements(node: &NodeRef) {
    if let Some(element) = node.as_element() {
        let tag_name = &element.name.local;
        if tag_name == &local_name!("code") {
            let text_content = node.text_contents();
            if text_content.contains('\t') || text_content.contains('\n') {
                let mut attrs = element.attributes.borrow_mut();
                attrs.insert("class", "tts_code_block".to_string());
            }
            return;
        }
    }
    for child in node.children() {
        process_code_elements(&child);
    }
}
// Helper for Requirement 5: Split long text into grouped sentences
fn split_text_content(parent: &NodeRef, full_text: &str, current_id: &RefCell<u32>) {
    // Clear existing children (the long text node)
    for child in parent.children() {
        child.detach();
    }

    let mut current_chunk = String::new();

    // Simple sentence/word tokenizer
    // We split by whitespace to preserve words, then reconstruct.
    // A more robust solution requires unicode-segmentation, but this uses std only.
    let words = full_text.split_inclusive(|c: char| c.is_whitespace());

    for word in words {
        if current_chunk.chars().count() + word.chars().count() > MAX_LENGTH
            && !current_chunk.is_empty()
        {
            append_span(parent, &current_chunk, current_id);
            current_chunk.clear();
        }
        current_chunk.push_str(word);

        // If word ends in punctuation, it's a good place to break if we are getting "full"
        // but strictly the prompt says "upto 500 length". We greedily fill up to 500.
    }

    if !current_chunk.is_empty() {
        append_span(parent, &current_chunk, current_id);
    }
}

fn append_span(parent: &NodeRef, text: &str, current_id: &RefCell<u32>) {
    let new_span =
        NodeRef::new_element(QualName::new(None, ns!(html), local_name!("span")), vec![]);

    tag_element(new_span.as_element().unwrap(), current_id);
    new_span.append(NodeRef::new_text(text));
    parent.append(new_span);
}
