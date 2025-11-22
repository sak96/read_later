mod app;
mod components;
mod pages;
mod routes;
mod web_utils;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
