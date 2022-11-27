mod app;
mod login;
mod message;

fn main() {
    yew::Renderer::<crate::app::App>::new().render();
}
