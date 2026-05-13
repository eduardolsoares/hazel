mod app;
mod block;
mod ipc;
mod models;
mod slash_menu;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
