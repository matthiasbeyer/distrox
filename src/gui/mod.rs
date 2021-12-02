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
use ipfs_api_backend_hyper::TryFromUri;

use crate::client::Client;
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
    client: Arc<Client>,

    scroll: scrollable::State,
    input: text_input::State,
    input_value: String,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Arc<Client>),
    FailedToLoad,

    InputChanged(String),
    CreatePost,

    PostCreated(crate::cid::Cid),
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
                match IpfsClient::from_str("http://localhost:5001") {
                    Err(_) => Message::FailedToLoad,
                    Ok(ipfs) => {
                        let config = Config::default();
                        let client = Client::new(ipfs, config);
                        Message::Loaded(Arc::new(client))
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
                    Message::Loaded(client) => {
                        let state = State {
                            client: client,
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
                            let client = state.client.clone();
                            iced::Command::perform(async move {
                                client.post_text_blob(state.input_value.clone()).await
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
                unimplemented!()
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
