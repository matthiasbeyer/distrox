use yew::{html, Callback, Component, Context, Html, NodeRef, Properties};

mod menu;
mod posts;
mod sidebar;

use self::menu::Menu;
use self::posts::Posts;
use self::sidebar::Sidebar;

pub struct Home;

#[derive(Properties, PartialEq)]
pub struct HomeProbs {}

impl Component for Home {
    type Message = ();
    type Properties = HomeProbs;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        log::debug!("Home::update()");
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <Menu />
                <Sidebar />
                <Posts />
            </>
        }
    }
}
