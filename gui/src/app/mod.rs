use std::sync::Arc;

use anyhow::Result;
use distrox_lib::profile::Profile;
use iced::scrollable;
use iced::text_input;
use iced::Application;
use iced::Column;
use iced::Container;
use iced::Length;
use iced::Row;
use iced::Scrollable;
use iced::TextInput;
use tokio::sync::RwLock;

use crate::timeline::PostLoadingRecipe;
use crate::timeline::Timeline;

mod message;
pub use message::Message;

mod handler;

use crate::gossip::GossipRecipe;

#[derive(Debug)]
pub(crate) enum Distrox {
    Loading {
        gossip_subscription_recv: RwLock<tokio::sync::oneshot::Receiver<GossipRecipe>>,
    },
    Loaded {
        profile: Arc<RwLock<Profile>>,
        gossip_subscription_recv: RwLock<tokio::sync::oneshot::Receiver<GossipRecipe>>,

        scroll: scrollable::State,
        input: text_input::State,
        input_value: String,
        timeline: Timeline,

        log_visible: bool,
        log: std::collections::VecDeque<String>,
    },
    FailedToStart,
}

impl Application for Distrox {
    type Executor = iced::executor::Default; // tokio
    type Message = Message;
    type Flags = String;

    fn new(name: String) -> (Self, iced::Command<Self::Message>) {
        let (gossip_subscription_sender, gossip_subscription_recv) =
            tokio::sync::oneshot::channel();
        (
            Distrox::Loading {
                gossip_subscription_recv: RwLock::new(gossip_subscription_recv),
            },
            iced::Command::perform(
                async move {
                    let profile = match Profile::load(&name).await {
                        Err(e) => return Message::FailedToLoad(e.to_string()),
                        Ok(instance) => Arc::new(RwLock::new(instance)),
                    };

                    if let Err(e) = profile
                        .read()
                        .await
                        .client()
                        .pubsub_subscribe("distrox".to_string())
                        .await
                        .map_err(anyhow::Error::from)
                        .map(|stream| {
                            log::trace!("Subscription to 'distrox' pubsub channel worked");
                            GossipRecipe::new(profile.clone(), stream)
                        })
                        .and_then(|s| {
                            gossip_subscription_sender.send(s).map_err(|_| {
                                anyhow::anyhow!("Failed to initialize gossipping module")
                            })
                        })
                    {
                        log::error!("Failed to load gossip recipe");
                        return Message::FailedToLoad(e.to_string());
                    }

                    Message::Loaded(profile)
                },
                |m: Message| -> Message { m },
            ),
        )
    }

    fn title(&self) -> String {
        String::from("distrox")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        handler::handle_message(self, message)
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        match self {
            Distrox::Loading { .. } => {
                let text = iced::Text::new("Loading");

                let content = Column::new().spacing(20).push(text);

                Container::new(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .into()
            }

            Distrox::Loaded {
                input,
                input_value,
                timeline,
                scroll,
                log_visible,
                log,
                ..
            } => {
                let left_column = Column::new().into();

                let mid_column = Column::new()
                    .push({
                        let input = TextInput::new(
                            input,
                            "What do you want to tell the world?",
                            input_value,
                            Message::InputChanged,
                        )
                        .padding(15)
                        .size(12)
                        .on_submit(Message::CreatePost);

                        let timeline = timeline.view();

                        Scrollable::new(scroll)
                            .padding(40)
                            .push(input)
                            .push(timeline)
                    })
                    .into();

                let right_column = Column::new().into();

                let content = Row::with_children(vec![left_column, mid_column, right_column])
                    .spacing(20)
                    .height(Length::Fill)
                    .width(Length::Fill);

                let content = Column::new()
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .push(content);

                if *log_visible {
                    let log = Column::with_children({
                        log.iter()
                            .map(iced::Text::new)
                            .map(|txt| txt.size(8))
                            .map(iced::Element::from)
                            .collect()
                    });
                    content.push(log)
                } else {
                    content
                }
                .into()
            }

            Distrox::FailedToStart => {
                unimplemented!()
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let post_loading_subs = match self {
            Distrox::Loaded { profile, .. } => {
                let profile = match profile.try_read() {
                    Err(_) => return iced::Subscription::none(),
                    Ok(p) => p,
                };

                match profile.head() {
                    None => iced::Subscription::none(),
                    Some(head) => iced::Subscription::from_recipe({
                        PostLoadingRecipe::new(profile.client().clone(), head.clone())
                    }),
                }
            }
            _ => iced::Subscription::none(),
        };

        let keyboard_subs = {
            use iced_native::event::Event;

            iced_native::subscription::events_with(|event, _| match event {
                Event::Keyboard(iced_native::keyboard::Event::KeyPressed { key_code, .. }) => {
                    if key_code == iced_native::keyboard::KeyCode::F11 {
                        Some(Message::ToggleLog)
                    } else {
                        None
                    }
                }
                _ => None,
            })
        };

        let gossip_sub = match self {
            Distrox::Loaded {
                gossip_subscription_recv,
                ..
            } => match gossip_subscription_recv.try_write() {
                Err(_) => None,
                Ok(mut sub) => sub
                    .try_recv()
                    .ok()
                    .map(iced::Subscription::from_recipe),
            },
            _ => None,
        };

        let gossip_sending_sub = {
            iced::time::every(std::time::Duration::from_secs(5))
                .map(|_| Message::PublishGossipAboutMe)
        };

        let mut subscriptions = vec![post_loading_subs, keyboard_subs, gossip_sending_sub];

        if let Some(gossip_sub) = gossip_sub {
            subscriptions.push(gossip_sub);
        }

        iced::Subscription::batch(subscriptions)
    }
}

pub fn run(name: String) -> Result<()> {
    let settings = iced::Settings {
        window: iced::window::Settings {
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
            ..iced::window::Settings::default()
        },
        flags: name,
        exit_on_close_request: true,
        ..iced::Settings::default()
    };

    Distrox::run(settings).map_err(anyhow::Error::from)
}
