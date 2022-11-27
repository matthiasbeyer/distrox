mod app;
mod login;
mod message;
mod tauri;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Starting app");
    println!("Starting app");
    yew::Renderer::<crate::app::App>::new().render();
}
