use yew::{html, Callback, Component, Context, Html, NodeRef, Properties};

pub struct Menu;

#[derive(Properties, PartialEq, Eq)]
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

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <nav class="navbar" role="navigation" aria-label="main navigation">
                <div class="navbar-brand">
                    <p class="navbar-item">{ "distrox" }</p>

                    <a role="button" class="navbar-burger" aria-label="menu" aria-expanded="false" data-target="navbarBasic">
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                    </a>
                </div>

                <div id="navbarBasic" class="navbar-menu">
                    <div class="navbar-start">
                        <a class="navbar-item">{ "Home" }</a>
                        <a class="navbar-item">{ "Help" }</a>
                    </div>

                    <div class="navbar-end">
                        <div class="navbar-item">
                            <div class="navbar-item has-dropdown is-hoverable">
                                <a class="navbar-link">{ "Account" }</a>

                                <div class="navbar-dropdown">
                                    <a class="navbar-item">{ "My Account" }</a>
                                    <a class="navbar-item">{ "Configuration" }</a>
                                    <a class="navbar-item">{ "Logout" }</a>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </nav>
        }
    }
}
