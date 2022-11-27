use yew::{Context, Component, Html};

use crate::{message::Message, login::login_screen};

pub enum App {
    Login,
}

impl Component for App {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self::Login
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Login => {
                // TODO
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self {
            App::Login => login_screen(ctx),
        }
    }
}
