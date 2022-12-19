use yew::{html, Callback, Component, Context, Html, NodeRef, Properties};

pub struct Post;

#[derive(Properties, PartialEq, Eq)]
pub struct PostProbs {}

impl Component for Post {
    type Message = ();
    type Properties = PostProbs;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        log::debug!("Post::update()");
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="block">
                <article class="message">
                    <div class="message-header">
                        <p>
                            { "Bob" }
                        </p>
                        <p>
                            { "2022-12-03 12:15:00" }
                        </p>
                    </div>

                    <div class="message-body">
                        <h1>{ "Hello World" }</h1>
                        <p>
                            { "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla accumsan, metus ultrices eleifend gravida, nulla nunc varius lectus, nec rutrum justo nibh eu lectus. Ut vulputate semper dui. Fusce erat odio, sollicitudin vel erat vel, interdum mattis neque." }
                        </p>
                    </div>

                </article>
            </div>
        }
    }
}
