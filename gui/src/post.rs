use crate::app::Message;
use distrox_lib::types::Payload;

#[derive(Clone, Debug)]
pub struct Post {
    payload: Payload,
    content: String,
}

impl Post {
    pub fn new(payload: Payload, content: String) -> Self {
        Self { payload, content }
    }

    pub fn view(&self) -> iced::Element<Message> {
        iced::Column::new()
            .push({
                iced::Row::new()
                    .height(iced::Length::Shrink)
                    .width(iced::Length::Fill)
                    .push({
                        iced::Column::new()
                            .width(iced::Length::Fill)
                            .align_items(iced::Alignment::Start)
                            .push({
                                iced::Text::new(self.payload.timestamp().inner().to_string())
                                    .size(10)
                            })
                    })
                    .push({
                        iced::Column::new()
                            .width(iced::Length::Fill)
                            .align_items(iced::Alignment::End)
                            .push({ iced::Text::new(self.payload.content().to_string()).size(10) })
                    })
            })
            .push(iced::rule::Rule::horizontal(10))
            .push({ iced::Text::new(self.content.clone()).size(12) })
            .push(iced::rule::Rule::horizontal(10))
            .into()
    }
}
