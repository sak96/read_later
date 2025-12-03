use web_sys::Element;
use yew::NodeRef;
pub fn scroll_to_top(div: &NodeRef, element_id: usize) {
    scroll_to_element(div, element_id, web_sys::ScrollLogicalPosition::Start);
}
pub fn scroll_to_center(div: &NodeRef, element_id: usize) {
    scroll_to_element(div, element_id, web_sys::ScrollLogicalPosition::Center);
}

fn scroll_to_element(div: &NodeRef, element_id: usize, position: web_sys::ScrollLogicalPosition) {
    if let Some(div) = div.cast::<Element>()
        && let Ok(Some(element)) = div.query_selector(&format!("#para_{}", element_id))
    {
        let scroll_into_view_options = web_sys::ScrollIntoViewOptions::new();
        scroll_into_view_options.set_behavior(web_sys::ScrollBehavior::Smooth);
        scroll_into_view_options.set_block(position);
        element.scroll_into_view_with_scroll_into_view_options(&scroll_into_view_options);
    }
}
