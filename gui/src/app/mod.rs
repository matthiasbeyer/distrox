use std::sync::Arc;

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
use tokio::sync::RwLock;

use crate::timeline::Timeline;
use crate::timeline::PostLoadingRecipe;

mod message;
pub use message::Message;

use crate::gossip::GossipRecipe;

#[derive(Debug)]
enum Distrox {
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
        let (gossip_subscription_sender, gossip_subscription_recv) = tokio::sync::oneshot::channel();
        (
            Distrox::Loading {
                gossip_subscription_recv: RwLock::new(gossip_subscription_recv),
            },

            iced::Command::perform(async move {
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
        log::trace!("Received message: {}", message.description());
        match self {
            Distrox::Loading { gossip_subscription_recv } => {
                if let Message::Loaded(profile) = message {
                    *self = Distrox::Loaded {
                        profile,

                        // Don't even try to think what hoops I am jumping through here...
                        gossip_subscription_recv: std::mem::replace(gossip_subscription_recv, RwLock::new(tokio::sync::oneshot::channel().1)),
                        scroll: scrollable::State::default(),
                        input: text_input::State::default(),
                        input_value: String::default(),
                        timeline: Timeline::new(),
                        log_visible: false,
                        log: std::collections::VecDeque::with_capacity(1000),
                    };


                }
                iced::Command::none()
            },

            Distrox::Loaded { profile, ref mut input_value, timeline, log_visible, log, .. } => {
                match message {
                    Message::InputChanged(input) => {
                        *input_value = input;
                        iced::Command::none()
                    }

                    Message::CreatePost => {
                        if !input_value.is_empty() {
                            let input = input_value.clone();
                            let profile = profile.clone();
                            log::trace!("Posting...");
                            iced::Command::perform(async move {
                                log::trace!("Posting: '{}'", input);
                                profile.write().await.post_text(input).await
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

                        let profile = profile.clone();
                        iced::Command::perform(async move {
                            if let Err(e) = profile.read().await.save().await {
                                Message::ProfileStateSavingFailed(e.to_string())
                            } else {
                                Message::ProfileStateSaved
                            }
                        }, |m: Message| -> Message { m })
                    }

                    Message::ProfileStateSaved => {
                        log::info!("Profile state saved");
                        iced::Command::none()
                    },

                    Message::ProfileStateSavingFailed(e) => {
                        log::error!("Saving profile failed: {}", e);
                        iced::Command::none()
                    },

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

                    Message::GossipMessage(source, msg) => {
                        log::trace!("Received Gossip from {}: {:?}", source, msg);
                        iced::Command::perform(async {
                            Message::GossipHandled(msg)
                        }, |m: Message| -> Message { m })
                    }

                    Message::GossipHandled(msg) => {
                        use distrox_lib::gossip::GossipMessage;

                        log::trace!("Gossip handled, adding to log: {:?}", msg);
                        let msg = match msg {
                            GossipMessage::CurrentProfileState { peer_id, cid } => {
                                format!("Peer {:?} is at {:?}", peer_id, cid)
                            }
                        };
                        log.push_back(msg);
                        while log.len() > 1000 {
                            let _ = log.pop_front();
                        }
                        iced::Command::none()
                    }

                    Message::PublishGossipAboutMe => {
                        let profile = profile.clone();
                        iced::Command::perform(async move {
                            if let Err(e) = profile.read().await.gossip_own_state("distrox".to_string()).await {
                                Message::GossippingFailed(e.to_string())
                            } else {
                                Message::OwnStateGossipped
                            }
                        }, |m: Message| -> Message { m })
                    }

                    Message::OwnStateGossipped => {
                        log::trace!("Gossipped own state");
                        log.push_back("Gossipped own state".to_string());
                        iced::Command::none()
                    }

                    Message::GossippingFailed(e) => {
                        log::trace!("Gossipped failed: {}", e);
                        log.push_back(format!("Gossipped failed: {}", e));
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

            Distrox::Loaded { input, input_value, timeline, scroll, log_visible, log, .. } => {
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
                let profile = match profile.try_read() {
                    Err(_) => return iced::Subscription::none(),
                    Ok(p) => p,
                };

                match profile.head() {
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
                match gossip_subscription_recv.try_write() {
                    Err(_) => None,
                    Ok(mut sub) => sub.try_recv()
                        .ok()
                        .map(|sub| iced::Subscription::from_recipe(sub)),
                }
            },
            _ => None,
        };

        let gossip_sending_sub = {
            iced::time::every(std::time::Duration::from_secs(5))
                .map(|_| Message::PublishGossipAboutMe)
        };

        let mut subscriptions = vec![
            post_loading_subs,
            keyboard_subs,
            gossip_sending_sub,
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

