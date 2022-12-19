use yew::{html, Component, Context, Html, Properties};

mod menu;
mod post;
mod posts;
mod sidebar;

use self::menu::Menu;
use self::posts::Posts;
use self::sidebar::Sidebar;

pub struct Home;

#[derive(Properties, PartialEq, Eq)]
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

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <div class="container">
                    <Menu />

                    <div class="columns">
                        <div class="column is-3">
                            <Sidebar />
                        </div>
                        <div class="column is-9">
                            <Posts />
                        </div>
                    </div>
                </div>
            </>
        }
    }
}
