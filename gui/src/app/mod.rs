use std::sync::Arc;
use std::sync::RwLock as StdRwLock;

use anyhow::Result;
use iced::Application;
use iced::Column;
use iced::Container;
use iced::Length;
use iced::Row;
use iced::Scrollable;
use iced::TextInput;
use iced::scrollable;
use iced::text_input;
use distrox_lib::profile::Profile;

use crate::timeline::Timeline;
use crate::timeline::PostLoadingRecipe;

mod message;
pub use message::Message;

use crate::gossip::GossipRecipe;

#[derive(Debug)]
enum Distrox {
    Loading {
        gossip_subscription_recv: StdRwLock<tokio::sync::oneshot::Receiver<GossipRecipe>>,
    },
    Loaded {
        profile: Arc<Profile>,
        gossip_subscription_recv: StdRwLock<tokio::sync::oneshot::Receiver<GossipRecipe>>,

        scroll: scrollable::State,
        input: text_input::State,
        input_value: String,
        timeline: Timeline,

        log_visible: bool,
    },
    FailedToStart,
}

impl Application for Distrox {
    type Executor = iced::executor::Default; // tokio
    type Message = Message;
    type Flags = String;

    fn new(name: String) -> (Self, iced::Command<Self::Message>) {
        let (gossip_subscription_sender, gossip_subscription_recv) = tokio::sync::oneshot::channel();
        (
            Distrox::Loading {
                gossip_subscription_recv: StdRwLock::new(gossip_subscription_recv),
            },

            iced::Command::perform(async move {
                let profile = match Profile::load(&name).await {
                    Err(e) => return Message::FailedToLoad(e.to_string()),
                    Ok(instance) => Arc::new(instance),
                };

                if let Err(e) = profile.client()
                    .pubsub_subscribe("distrox".to_string())
                    .await
                    .map_err(anyhow::Error::from)
                    .map(|stream| {
                        log::trace!("Subscription to 'distrox' pubsub channel worked");
                        GossipRecipe::new(profile.clone(), stream)
                    })
                    .and_then(|s| gossip_subscription_sender.send(s).map_err(|_| anyhow::anyhow!("Failed to initialize gossipping module")))
                {
                    log::error!("Failed to load gossip recipe");
                    return Message::FailedToLoad(e.to_string())
                }

                Message::Loaded(profile)
            }, |m: Message| -> Message { m })
        )
    }

    fn title(&self) -> String {
        String::from("distrox")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match self {
            Distrox::Loading { gossip_subscription_recv } => {
                if let Message::Loaded(profile) = message {
                    *self = Distrox::Loaded {
                        profile,

                        // Don't even try to think what hoops I am jumping through here...
                        gossip_subscription_recv: std::mem::replace(gossip_subscription_recv, StdRwLock::new(tokio::sync::oneshot::channel().1)),
                        scroll: scrollable::State::default(),
                        input: text_input::State::default(),
                        input_value: String::default(),
                        timeline: Timeline::new(),
                        log_visible: false
                    };


                }
                iced::Command::none()
            },

            Distrox::Loaded { profile, ref mut input_value, timeline, log_visible, .. } => {
                match message {
                    Message::InputChanged(input) => {
                        *input_value = input;
                        iced::Command::none()
                    }

                    Message::CreatePost => {
                        if !input_value.is_empty() {
                            let input = input_value.clone();
                            let client = profile.client().clone();
                            log::trace!("Posting...");
                            iced::Command::perform(async move {
                                log::trace!("Posting: '{}'", input);
                                client.post_text_blob(input).await
                            },
                            |res| match res {
                                Ok(cid) => Message::PostCreated(cid),
                                Err(e) => Message::PostCreationFailed(e.to_string())
                            })
                        } else {
                            iced::Command::none()
                        }
                    }

                    Message::PostCreated(cid) => {
                        *input_value = String::new();
                        log::info!("Post created: {}", cid);
                        iced::Command::none()
                    }

                    Message::PostCreationFailed(err) => {
                        log::error!("Post creation failed: {}", err);
                        iced::Command::none()
                    }

                    Message::PostLoaded((payload, content)) => {
                        timeline.push(payload, content);
                        iced::Command::none()
                    }

                    Message::PostLoadingFailed => {
                        log::error!("Failed to load some post, TODO: Better error logging");
                        iced::Command::none()
                    }

                    Message::TimelineScrolled(f) => {
                        log::trace!("Timeline scrolled: {}", f);
                        iced::Command::none()
                    }

                    Message::ToggleLog => {
                        log::trace!("Log toggled");
                        *log_visible = !*log_visible;
                        iced::Command::none()
                    }

                    _ => iced::Command::none(),
                }
            }

            Distrox::FailedToStart => {
                unimplemented!()
            }
        }
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        match self {
            Distrox::Loading { .. } => {
                let text = iced::Text::new("Loading");

                let content = Column::new()
                    .spacing(20)
                    .push(text);

                Container::new(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .into()
            }

            Distrox::Loaded { input, input_value, timeline, scroll, log_visible, .. } => {
                let left_column = Column::new()
                    .into();

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

                let right_column = Column::new()
                    .into();

                let content = Row::with_children(vec![
                        left_column,
                        mid_column,
                        right_column
                    ])
                    .spacing(20)
                    .height(Length::Fill)
                    .width(Length::Fill);

                let content = Column::new()
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .push(content);

                if *log_visible {
                    let log = Column::new()
                        .push({
                            iced::Text::new("Here goes some log,... not yet implemented!")
                                .size(8)
                        });
                    content.push(log)
                } else {
                    content
                }.into()
            }

            Distrox::FailedToStart => {
                unimplemented!()
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let post_loading_subs = match self {
            Distrox::Loaded { profile, .. } => {
                let head = profile.head();

                match head {
                    None => iced::Subscription::none(),
                    Some(head) => {
                        iced::Subscription::from_recipe({
                            PostLoadingRecipe::new(profile.client().clone(), head.clone())
                        })
                    }
                }
            }
            _ => iced::Subscription::none(),
        };

        let keyboard_subs = {
            use iced_native::event::Event;

            iced_native::subscription::events_with(|event, _| {
                match event {
                    Event::Keyboard(iced_native::keyboard::Event::KeyPressed { key_code, .. }) => {
                        if key_code == iced_native::keyboard::KeyCode::F11 {
                            Some(Message::ToggleLog)
                        } else {
                            None
                        }
                    },
                    _ => None,
                }
            })
        };

        let gossip_sub = match self {
            Distrox::Loaded { gossip_subscription_recv, .. }  => {
                match gossip_subscription_recv.write().ok() {
                    Some(mut sub) => sub.try_recv()
                        .ok() // Either empty or closed, ignore both
                        .map(|sub| iced::Subscription::from_recipe(sub)),
                    None => None
                }
            },
            _ => None,
        };

        let mut subscriptions = vec![
            post_loading_subs,
            keyboard_subs,
        ];

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

