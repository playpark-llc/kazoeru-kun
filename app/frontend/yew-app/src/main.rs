mod app;
mod components;
mod router;
mod env;
use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
