use anyhow::Result;
use iced::Application;
use ipfs_api_backend_hyper::TryFromUri;

use crate::client::Client;
use crate::config::Config;
use crate::ipfs_client::IpfsClient;

#[derive(Debug)]
struct DistroxGui;

#[derive(Debug)]
enum Message {
    Loaded(Result<Client>),
}

impl Application for DistroxGui {
    type Executor = iced::executor::Default; // tokio
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, iced::Command<Self::Message>) {
        (
            DistroxGui,
            iced::Command::perform(async {
                let ipfs  = IpfsClient::from_str("http://localhost:5001")?;
                let config = Config::default();
                let client = Client::new(ipfs, config);
                Ok(client)
            }, Message::Loaded)
        )
    }

    fn title(&self) -> String {
        String::from("distrox")
    }

    fn update(&mut self, _message: Self::Message, _clipboard: &mut iced::Clipboard) -> iced::Command<Self::Message> {
        iced::Command::none()
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        iced::Text::new("Hello, world!").into()
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

    DistroxGui::run(settings).map_err(anyhow::Error::from)
}
