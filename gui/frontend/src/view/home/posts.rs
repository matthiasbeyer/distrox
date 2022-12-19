use yew::{html, Component, Context, Html, Properties};

use crate::view::home::post::Post;

pub struct Posts;

#[derive(Properties, PartialEq, Eq)]
pub struct PostsProbs {}

impl Component for Posts {
    type Message = ();
    type Properties = PostsProbs;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        log::debug!("Posts::update()");
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <Post />
            </>
        }
    }
}
