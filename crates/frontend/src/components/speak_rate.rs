use crate::web_utils::{get_setting, set_setting};
use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct SpeakRateProps {
    pub on_rate_change: Callback<f32>,
    #[prop_or_default]
    pub outline: bool,
}

const RATE: [f32; 4] = [0.5, 1.0, 1.5, 2.0];

#[component(SpeakRate)]
pub fn speak_rate(props: &SpeakRateProps) -> Html {
    let rate = use_state(|| 1.0);
    {
        let rate = rate.clone();
        let props = props.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(value) = get_setting("rate").await
                    && let Ok(value) = value.parse::<f32>()
                    && RATE.contains(&value)
                {
                    rate.set(value);
                    props.on_rate_change.emit(value);
                } else {
                    let value = 1.0;
                    set_setting("rate", &value.to_string()).await;
                    rate.set(value);
                    props.on_rate_change.emit(*rate);
                }
            });
        });
    }
    let on_change = {
        let rate = rate.clone();
        let props = props.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                && let Ok(new_rate) = input.value().parse::<f32>()
            {
                let rate = rate.clone();
                let props = props.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    set_setting("rate", &new_rate.to_string()).await;
                    rate.set(new_rate);
                    props.on_rate_change.emit(new_rate);
                });
            }
        })
    };

    html! {
        <div role="group">
            <label role="button" class={classes!(props.outline.then_some("outline"))}>
                <b>{&format!("{:0.1}x", *rate)}</b>
                <input type="range" min="0.5" step="0.5" max="2" value={rate.to_string()} onchange={on_change} />
            </label>
        </div>
    }
}
