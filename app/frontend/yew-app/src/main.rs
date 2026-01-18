mod app;
mod components;
mod models;
mod router;
mod env;
mod api_client;
use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
