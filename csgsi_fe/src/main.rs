mod app;
pub mod gsi;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    yew::Renderer::<App>::new().render();
}
