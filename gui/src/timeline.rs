use std::collections::BTreeMap;
use std::collections::HashSet;

use anyhow::Result;
use futures::StreamExt;

use iced_native::widget::scrollable::State as ScrollableState;

use crate::app::Message;
use crate::post::Post;
use distrox_lib::client::Client;
use distrox_lib::stream::NodeStreamBuilder;
use distrox_lib::types::DateTime;
use distrox_lib::types::Payload;

#[derive(Debug)]
pub struct Timeline {
    post_ids: HashSet<cid::Cid>,
    posts: BTreeMap<DateTime, Post>,
    scrollable: ScrollableState,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            post_ids: HashSet::with_capacity(1000),
            posts: BTreeMap::new(),
            scrollable: ScrollableState::new(),
        }
    }

    pub fn push(&mut self, payload: Payload, content: String) {
        if self.post_ids.insert(payload.content()) {
            self.posts
                .insert(payload.timestamp().clone(), Post::new(payload, content));
        }
    }

    pub fn view(&mut self) -> iced::Element<Message> {
        let scrollable = iced::Scrollable::new(&mut self.scrollable)
            .padding(10)
            .spacing(20)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .on_scroll(move |offset| Message::TimelineScrolled(offset));

        self.posts
            .iter()
            .rev()
            .fold(scrollable, |scrollable, (_, post)| {
                scrollable.push(post.view())
            })
            .into()
    }
}

pub struct PostLoadingRecipe {
    client: Client,
    head: cid::Cid,
}

impl PostLoadingRecipe {
    pub fn new(client: Client, head: cid::Cid) -> Self {
        Self { client, head }
    }
}

// Make sure iced can use our download stream
impl<H, I> iced_native::subscription::Recipe<H, I> for PostLoadingRecipe
where
    H: std::hash::Hasher,
{
    type Output = Message;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;
        self.head.to_bytes().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        log::debug!("Streaming posts starting at HEAD = {:?}", self.head);
        Box::pin({
            NodeStreamBuilder::starting_from(self.head.clone())
                .into_stream(self.client.clone())
                .then(move |node| {
                    let client = self.client.clone();

                    async move {
                        let payload = client.get_payload(node?.payload()).await?;
                        let content = client.get_content_text(payload.content()).await?;

                        Ok((payload, content))
                    }
                })
                .map(|res: Result<_>| match res {
                    Err(_) => Message::PostLoadingFailed,
                    Ok(p) => Message::PostLoaded(p),
                })
        })
    }
}
