use crate::components::{HomeButton, LinkPopup, SpeakBar};
use crate::routes::Route;
use crate::web_utils::{invoke_no_parse_log_error, open_url, set_callback_to_link};
use shared::models::Article;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct ReadViewerProps {
    pub article: Article,
}

#[component(ReadViewer)]
pub fn read_viewer(props: &ReadViewerProps) -> Html {
    // States
    let div_ref = use_node_ref();
    let delete_modal = use_state(|| false);
    let url = &props.article.url;

    // link handling
    let external_url = use_state(|| None::<String>);
    let on_link = {
        let external_url = external_url.clone();
        Callback::from(move |href: String| {
            external_url.set(Some(href));
        })
    };
    let on_link_close = {
        let external_url = external_url.clone();
        Callback::from(move |_| {
            external_url.set(None);
        })
    };
    {
        let div_ref = div_ref.clone();
        let on_click = on_link.clone();
        let id = props.article.id;
        use_effect_with(div_ref, move |div_ref| {
            set_callback_to_link(div_ref, on_click, id);
        })
    }

    // open current article
    let open_web_url = {
        let url = url.clone();
        Callback::from(move |_| {
            let url = url.clone();
            spawn_local(async move {
                open_url((*url).to_owned()).await;
            });
        })
    };

    // delete button
    let navigator = use_navigator().unwrap();
    let delete_dialog_toggle = {
        let delete_modal = delete_modal.clone();
        Callback::from(move |_| {
            delete_modal.set(!*delete_modal);
        })
    };
    let delete_article = {
        let article_id = props.article.id;
        Callback::from(move |_| {
            let navigator = navigator.clone();
            spawn_local(async move {
                invoke_no_parse_log_error(
                    "delete_article",
                    &Some(serde_json::json!({"id": article_id})),
                )
                .await;
                navigator.push(&Route::Home);
            });
        })
    };

    html! {
        <div class="container">
            <article ref={div_ref.clone()} class="page reader_view">
                <h1>{&props.article.title}</h1>
                {Html::from_html_unchecked(props.article.body.clone().into())}
            </article>
            // External Link handling
            <LinkPopup url={(*external_url).clone()} on_close={on_link_close} />
            // Delete modal
            <dialog open={*delete_modal}>
              <article>
                <h2><strong class="ti ti-trash-x">{format!("\u{f784}: {}", &props.article.title)}</strong></h2>
                <footer>
                  <button class="secondary" onclick={delete_dialog_toggle.clone()}><i class="ti ti-x">{"\u{eb55}"}</i></button>
                  <button onclick={delete_article}><i class="ti ti-check">{"\u{ea5e}"}</i></button>
                </footer>
              </article>
            </dialog>
            // Action area
            <aside style="position: sticky; bottom: var(--safe-area-inset-bottom, 0);">
                <nav>
                    <SpeakBar {div_ref} />
                    <div role="group">
                        <HomeButton />
                        <button onclick={open_web_url}><i class="ti ti-world-www">{"\u{f38f}"}</i></button>
                        <button class="secondary" onclick={delete_dialog_toggle}><i class="ti ti-trash-x">{"\u{f784}"}</i></button>
                    </div>
                </nav>
            </aside>
        </div>
    }
}
