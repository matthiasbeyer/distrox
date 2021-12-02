use anyhow::Result;
use iced::Application;

#[derive(Debug)]
struct DistroxGui;

impl Application for DistroxGui {
    type Executor = iced::executor::Default; // tokio
    type Message = ();
    type Flags = ();

    fn new(_flags: ()) -> (Self, iced::Command<Self::Message>) {
        (DistroxGui, iced::Command::none())
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
    DistroxGui::run(iced::Settings::default()).map_err(anyhow::Error::from)
}
