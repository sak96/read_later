use crate::{
    components::{LanguageSelection, SpeakRate},
    web_utils::{
        extract_text, find_visible_para_id, scroll_to_center, scroll_to_top, speak, stop_speak,
    },
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ViewMode {
    View,
    Reader,
}

#[derive(Properties, PartialEq, Clone)]
pub struct SpeakBarProps {
    pub div_ref: NodeRef,
}

#[function_component(SpeakBar)]
pub fn speak_bar(props: &SpeakBarProps) -> Html {
    let div_ref = props.div_ref.clone();
    let checkpoint = use_state(|| 0);
    let mode = use_state(|| ViewMode::View);
    let rate = use_state(|| 1.0);
    // Rate change
    let on_rate_change = {
        let rate = rate.clone();
        Callback::from(move |new_rate| rate.set(new_rate))
    };
    // Handle mode transitions
    let on_mode_switch = {
        let mode = mode.clone();
        let div_ref = div_ref.clone();
        let checkpoint = checkpoint.clone();
        Callback::from(move |_| {
            if *mode == ViewMode::Reader {
                spawn_local(stop_speak());
                scroll_to_top(&div_ref, *checkpoint);
                mode.set(ViewMode::View);
            } else {
                let id = find_visible_para_id();
                checkpoint.set(id);
                mode.set(ViewMode::Reader);
            }
        })
    };

    // Reader background task
    {
        let mode = mode.clone();
        let checkpoint = checkpoint.clone();
        let rate = rate.clone();
        let div_ref = div_ref.clone();
        use_effect_with((*mode, checkpoint), move |(reader_mode, checkpoint)| {
            if *reader_mode == ViewMode::Reader {
                let mode = mode.clone();
                let checkpoint = checkpoint.clone();
                let rate = rate.clone();
                let div_ref = div_ref.clone();
                spawn_local(async move {
                    if *mode == ViewMode::Reader {
                        if let Some(para_text) = extract_text(&div_ref, *checkpoint) {
                            let div_ref = div_ref.clone();
                            scroll_to_center(&div_ref, *checkpoint);
                            speak(para_text.clone(), *rate).await;
                            checkpoint.set(*checkpoint + 1);
                        } else {
                            mode.set(ViewMode::View);
                        }
                    }
                });
            }
            || ()
        });
    }

    // Scroll to checkpoint
    let scroll_to_checkpoint = {
        let div_ref = div_ref.clone();
        let checkpoint = checkpoint.clone();
        Callback::from(move |_| {
            scroll_to_top(&div_ref, *checkpoint);
        })
    };
    html! {
        <>
            <style>{{
                let current_para = *checkpoint;
                format!("#para_{current_para} {{border: var(--pico-border-width) solid var(--pico-primary-hover);border-radius: var(--pico-border-radius)}}")
            }}</style>
            if *mode == ViewMode::View {
                <fieldset role="group">
                    <button onclick={on_mode_switch.clone()}>
                        <i class="ti ti-volume"></i>
                    </button>
                    <LanguageSelection />
                    <button onclick={scroll_to_checkpoint}><i class="ti ti-player-skip-back"></i></button>
                </fieldset>
            } else {
                <fieldset role="group">
                    <button class="icon-btn pause-btn" onclick={on_mode_switch}>
                        <i class="ti ti-player-pause"></i>
                    </button>
                    <SpeakRate {on_rate_change} />
                </fieldset>
            }
        </>
    }
}
