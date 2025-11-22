mod app;
mod components;
mod pages;
mod routes;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
