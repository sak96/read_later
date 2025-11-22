use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct ThemeContext {
    pub mode: String,
    pub set_mode: Callback<String>,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub children: Html,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[function_component(ThemeProvider)]
pub fn theme_provider(props: &Props) -> Html {
    let mode = use_state(|| "system".to_string());
    {
        let mode = mode.clone();
        use_effect(move || {
            spawn_local(async move {
                let result = invoke(
                    "get_setting",
                    serde_wasm_bindgen::to_value(&serde_json::json!({"name": "theme"})).unwrap(),
                )
                .await;
                if let Ok(value) = serde_wasm_bindgen::from_value::<String>(result) {
                    mode.set(value);
                }
            });
        });
    }
    {
        let mode = mode.clone();
        use_effect_with(mode, move |mode| {
            let mode = mode.clone();
            let window = web_sys::window().expect("no window");
            let document = window.document().expect("no document");
            let html = document.document_element().expect("no <html>");

            match mode.as_str() {
                "dark" | "light" => {
                    html.set_attribute("data-theme", &mode)
                        .expect("set data-theme");
                }
                _ => {
                    html.remove_attribute("data-theme").ok();
                }
            };
            || ()
        });
    }

    let set_mode = {
        let mode = mode.clone();
        Callback::from(move |new_mode: String| {
            mode.set(new_mode);
        })
    };

    let ctx = ThemeContext {
        mode: (*mode).to_owned(),
        set_mode,
    };

    html! {
        <ContextProvider<ThemeContext> context={ctx}>
            { props.children.clone() }
        </ContextProvider<ThemeContext>>
    }
}
