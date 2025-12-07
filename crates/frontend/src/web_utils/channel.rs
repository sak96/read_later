use crate::web_utils::{error, transform_callback, unregister_channel};
use serde::{Deserialize, Serialize, Serializer};
use std::marker::PhantomData;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[derive(Clone)]
pub struct Channel<T> {
    id: u32,
    _type: PhantomData<T>,
}

impl<T> Serialize for Channel<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("__CHANNEL__:{}", self.id);
        serializer.serialize_str(&s)
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        unregister_channel(self.id)
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Message<T> {
    pub message: T,
    index: u32,
}

impl<T> From<Callback<T>> for Channel<T>
where
    T: serde::de::DeserializeOwned,
{
    fn from(callback: Callback<T>) -> Self {
        let closure = Closure::wrap(Box::new(move |response: JsValue| {
            match serde_wasm_bindgen::from_value::<Message<T>>(response.clone()) {
                Ok(event) => {
                    callback.emit(event.message);
                }
                Err(err) => error(&format!(
                    "failed to parse {response:#?} with error {err:#?}"
                )),
            }
        }) as Box<dyn FnMut(JsValue)>);
        let id = transform_callback(closure.as_ref().unchecked_ref::<_>(), false);
        closure.forget();
        Channel::<T> {
            id,
            _type: PhantomData,
        }
    }
}
