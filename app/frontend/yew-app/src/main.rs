mod app;
mod components;
mod models;
mod router;
mod env;
use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
