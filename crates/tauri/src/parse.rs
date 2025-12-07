use kuchiki::NodeRef;
use kuchiki::traits::*;

pub fn process_html(html: &str) -> String {
    let document = kuchiki::parse_html().one(html);
    process_node(&document, &mut 0);
    let mut bytes = Vec::new();
    document.serialize(&mut bytes).unwrap();
    String::from_utf8(bytes).unwrap()
}

fn process_node(node: &NodeRef, current_id: &mut u32) {
    if let Some(element) = node.as_element() {
        let tag_name = element.name.local.as_ref();
        if is_unwanted_element(tag_name) {
            return;
        }
        let is_block = is_block_element(tag_name);
        let has_text_content = !node.text_contents().trim().is_empty();
        if is_block && has_text_content {
            let mut meets_structural_condition = false;
            // - A non-block element (inline element)
            // - OR text that is not in any child tag (direct text node)
            for child in node.children() {
                if let Some(child_el) = child.as_element() {
                    if !is_block_element(child_el.name.local.as_ref()) {
                        meets_structural_condition = true;
                        break;
                    }
                } else if let Some(text) = child.as_text()
                    && !text.borrow().trim().is_empty()
                {
                    meets_structural_condition = true;
                    break;
                }
            }

            if meets_structural_condition {
                // Apply class tts_para_{}
                let mut attributes = element.attributes.borrow_mut();
                if let Some(class_attr) = attributes.get_mut("class") {
                    class_attr.push_str(&format!(" tts_para_{}", current_id));
                } else {
                    attributes.insert("class", format!("tts_para_{}", current_id));
                }
                *current_id += 1;
                return;
            }
        }
    }
    // Recurse into children
    for child in node.children() {
        process_node(&child, current_id);
    }
}

fn is_unwanted_element(tag: &str) -> bool {
    matches!(
        tag,
        "script" | "style" | "noscript" | "iframe" | "svg" | "head" | "link" | "meta"
    )
}

fn is_block_element(tag: &str) -> bool {
    matches!(
        tag,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "details"
            | "dialog"
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
            | "hgroup"
            | "hr"
            | "li"
            | "main"
            | "nav"
            | "ol"
            | "p"
            | "pre"
            | "section"
            | "table"
            | "ul"
            | "body"
    )
}
