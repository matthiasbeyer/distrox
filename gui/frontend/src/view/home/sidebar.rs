use yew::{html, Callback, Component, Context, Html, NodeRef, Properties};

pub struct Sidebar;

#[derive(Properties, PartialEq, Eq)]
pub struct SidebarProbs {}

impl Component for Sidebar {
    type Message = ();
    type Properties = SidebarProbs;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        log::debug!("Sidebar::update()");
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <p> { "Sidebar" } </p>
        }
    }
}
