#[derive(Debug)]
pub struct TimelinePost {
    mime: mime::Mime,
    content: PostContent,
}

#[derive(Debug)]
pub enum PostContent {
    Text(String)
}

impl TimelinePost {
    pub fn update(&mut self) {
        ()
    }

    pub fn view(&self) -> iced::Row<crate::app::Message> {
        iced::Row::new()
            .push({
                iced::Text::new(self.mime.as_ref().to_string())
            })
            .push({
                match self.content {
                    PostContent::Text(ref txt) => iced::Text::new(txt.clone()),
                }
            })
            .into()
    }
}
