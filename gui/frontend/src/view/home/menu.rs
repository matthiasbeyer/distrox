use yew::{html, Callback, Component, Context, Html, NodeRef, Properties};

pub struct Menu;

#[derive(Properties, PartialEq)]
pub struct MenuProbs {}

impl Component for Menu {
    type Message = ();
    type Properties = MenuProbs;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        log::debug!("Menu::update()");
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <p> { "Menu" } </p>
        }
    }
}
