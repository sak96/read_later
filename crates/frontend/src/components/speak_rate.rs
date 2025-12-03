use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct SpeakRateProps {
    pub rate: f32,
    pub on_rate_change: Callback<f32>,
}

#[function_component(SpeakRate)]
pub fn speak_rate(props: &SpeakRateProps) -> Html {
    let rate = use_state(|| props.rate.clone());
    let on_change = {
        let rate = rate.clone();
        let props = props.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                && let Ok(new_rate) = input.value().parse::<f32>()
            {
                rate.set(new_rate);
                props.on_rate_change.emit(new_rate);
            }
        })
    };

    html! {
        <div role="group">
            <label role="button">
                <b>{&format!("{:0.1}x", *rate)}</b>
                <input type="range" min="0.5" step="0.5" max="2" value={rate.to_string()} onchange={on_change} />
            </label>
        </div>
    }
}
