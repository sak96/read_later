use web_sys::window;
pub fn scroll_to_top(element_id: usize) {
    scroll_to_element(element_id, web_sys::ScrollLogicalPosition::Start);
}
pub fn scroll_to_center(element_id: usize) {
    scroll_to_element(element_id, web_sys::ScrollLogicalPosition::Center);
}

fn scroll_to_element(element_id: usize, position: web_sys::ScrollLogicalPosition) {
    if let Some(window) = window()
        && let Some(document) = window.document()
        && let Some(element) = document.get_element_by_id(&format!("para_{}", element_id))
    {
        let scroll_into_view_options = web_sys::ScrollIntoViewOptions::new();
        scroll_into_view_options.set_behavior(web_sys::ScrollBehavior::Smooth);
        scroll_into_view_options.set_block(position);
        element.scroll_into_view_with_scroll_into_view_options(&scroll_into_view_options);
    }
}
