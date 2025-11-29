use kuchiki::NodeRef;
use kuchiki::traits::*;

pub fn process_html(html: &str, max_length: u32, current_id: &mut u32) -> String {
    let document = kuchiki::parse_html().one(html);
    process_node(
        &document,
        max_length,
        current_id,
        &[
            "script", "style", "noscript", "iframe", "svg", "head", "link", "meta",
        ],
    );
    let mut bytes = Vec::new();
    document.serialize(&mut bytes).unwrap();
    String::from_utf8(bytes).unwrap()
}

fn process_node(node: &NodeRef, max_length: u32, current_id: &mut u32, skip_tags: &[&str]) {
    if let Some(element) = node.as_element()
        && !skip_tags.contains(&element.name.local.as_ref())
    {
        let text_len = node.text_contents().chars().count() as u32;
        if text_len > max_length {
            for child in node.children() {
                process_node(&child, max_length, current_id, skip_tags);
            }
        } else if text_len > 0 {
            element
                .attributes
                .borrow_mut()
                .insert("id", format!("para_{}", current_id));
            *current_id += 1;
        }
    } else if node.as_document().is_some() {
        for child in node.children() {
            process_node(&child, max_length, current_id, skip_tags);
        }
    }
}
