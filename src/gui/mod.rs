use std::sync::Arc;

use anyhow::Result;
use iced::Application;
use iced::Column;
use iced::Container;
use iced::Element;
use iced::Length;
use iced::Scrollable;
use iced::TextInput;
use iced::scrollable;
use iced::text_input;

use crate::profile::Profile;
use crate::config::Config;
use crate::ipfs_client::IpfsClient;

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
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Arc<Profile>),
    FailedToLoad,

    InputChanged(String),
    CreatePost,

    PostCreated(cid::Cid),
    PostCreationFailed(String),
}

impl Application for Distrox {
    type Executor = iced::executor::Default; // tokio
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, iced::Command<Self::Message>) {
        (
            Distrox::Loading,
            iced::Command::perform(async {
                match Profile::new_inmemory(Config::default()).await {
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

    fn update(&mut self, message: Self::Message, _clipboard: &mut iced::Clipboard) -> iced::Command<Self::Message> {
        match self {
            Distrox::Loading => {
                match message {
                    Message::Loaded(profile) => {
                        let state = State {
                            profile,
                            scroll: scrollable::State::default(),
                            input: text_input::State::default(),
                            input_value: String::default(),
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
                            let profile = state.profile.clone();
                            let input = state.input_value.clone();
                            iced::Command::perform(async move {
                                profile.client().post_text_blob(input).await
                            },
                            |res| match res {
                                Ok(cid) => Message::PostCreated(cid),
                                Err(e) => Message::PostCreationFailed(e.to_string())
                            });
                        }
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
                let text = iced_native::widget::text::Text::new("Loading");

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
                .size(30)
                .on_submit(Message::CreatePost);

                let content = Column::new()
                    .max_width(800)
                    .spacing(20)
                    .push(input);

                Scrollable::new(&mut state.scroll)
                    .padding(40)
                    .push(
                        Container::new(content).width(Length::Fill).center_x(),
                    )
                    .into()
            }

            Distrox::FailedToStart => {
                unimplemented!()
            }
        }
    }

}

pub fn run() -> Result<()> {
    let settings = iced::Settings {
        window: iced::window::Settings {
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
            ..iced::window::Settings::default()
        },
        exit_on_close_request: true,
        ..iced::Settings::default()
    };

    Distrox::run(settings).map_err(anyhow::Error::from)
}
