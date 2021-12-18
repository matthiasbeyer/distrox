use std::sync::Arc;

use anyhow::Result;
use iced::Application;
use iced::Column;
use iced::Container;
use iced::Length;
use iced::Scrollable;
use iced::TextInput;
use iced::scrollable;
use iced::text_input;

use distrox_lib::profile::Profile;
use crate::timeline::Timeline;
use crate::timeline::PostLoadingRecipe;

#[derive(Debug)]
enum Distrox {
    Loading,
    Loaded(State),
    FailedToStart,
}

#[derive(Debug)]
struct State {
    profile: Arc<Profile>,

    scroll: scrollable::State,
    input: text_input::State,
    input_value: String,
    timeline: Timeline,
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Arc<Profile>),
    FailedToLoad,

    InputChanged(String),
    CreatePost,

    PostCreated(cid::Cid),
    PostCreationFailed(String),

    PostLoaded((distrox_lib::types::Payload, String)),
    PostLoadingFailed,

    TimelineScrolled(f32),
}

impl Application for Distrox {
    type Executor = iced::executor::Default; // tokio
    type Message = Message;
    type Flags = String;

    fn new(name: String) -> (Self, iced::Command<Self::Message>) {
        (
            Distrox::Loading,
            iced::Command::perform(async move {
                match Profile::load(&name).await {
                    Err(_) => Message::FailedToLoad,
                    Ok(instance) => {
                        Message::Loaded(Arc::new(instance))
                    }
                }
            }, |m: Message| -> Message { m })
        )
    }

    fn title(&self) -> String {
        String::from("distrox")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match self {
            Distrox::Loading => {
                match message {
                    Message::Loaded(profile) => {
                        let state = State {
                            profile,
                            scroll: scrollable::State::default(),
                            input: text_input::State::default(),
                            input_value: String::default(),
                            timeline: Timeline::new(),
                        };
                        *self = Distrox::Loaded(state);
                    }

                    Message::FailedToLoad => {
                        log::error!("Failed to load");
                        *self = Distrox::FailedToStart;
                    }

                    _ => {}

                }
            }

            Distrox::Loaded(state) => {
                match message {
                    Message::InputChanged(input) => {
                        state.input_value = input;
                    }

                    Message::CreatePost => {
                        if !state.input_value.is_empty() {
                            let input = state.input_value.clone();
                            let client = state.profile.client().clone();
                            log::trace!("Posting...");
                            iced::Command::perform(async move {
                                log::trace!("Posting: '{}'", input);
                                client.post_text_blob(input).await
                            },
                            |res| match res {
                                Ok(cid) => Message::PostCreated(cid),
                                Err(e) => Message::PostCreationFailed(e.to_string())
                            });
                        }
                    }

                    Message::PostCreated(cid) => {
                        state.input_value = String::new();
                        log::info!("Post created: {}", cid);
                    }

                    Message::PostCreationFailed(err) => {
                        log::error!("Post creation failed: {}", err);
                    }

                    Message::PostLoaded((payload, content)) => {
                        state.timeline.push(payload, content);
                    }

                    Message::PostLoadingFailed => {
                        log::error!("Failed to load some post, TODO: Better error logging");
                    }

                    Message::TimelineScrolled(f) => {
                        log::trace!("Timeline scrolled: {}", f);
                    }

                    _ => {}
                }
            }

            Distrox::FailedToStart => {
                unimplemented!()
            }
        }
        iced::Command::none()
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        match self {
            Distrox::Loading => {
                let text = iced::Text::new("Loading");

                let content = Column::new()
                    .max_width(800)
                    .spacing(20)
                    .push(text);

                Container::new(content)
                    .width(Length::Fill)
                    .center_x()
                    .into()
            }

            Distrox::Loaded(state) => {
                let input = TextInput::new(
                    &mut state.input,
                    "What do you want to tell the world?",
                    &mut state.input_value,
                    Message::InputChanged,
                )
                .padding(15)
                .size(12)
                .on_submit(Message::CreatePost);

                let timeline = state.timeline.view();

                Scrollable::new(&mut state.scroll)
                    .padding(40)
                    .push(input)
                    .push(timeline)
                    .into()
            }

            Distrox::FailedToStart => {
                unimplemented!()
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        match self {
            Distrox::Loaded(state) => {
                let head = state.profile.head();

                match head {
                    None => iced::Subscription::none(),
                    Some(head) => {
                        iced::Subscription::from_recipe({
                            PostLoadingRecipe::new(state.profile.client().clone(), head.clone())
                        })
                    }
                }
            }
            _ => iced::Subscription::none(),
        }
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
