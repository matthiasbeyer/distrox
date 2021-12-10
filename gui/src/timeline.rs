use crate::timeline_post::TimelinePost;

#[derive(Debug)]
pub struct Timeline {
    posts: Vec<TimelinePost>
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            posts: Vec::with_capacity(100),
        }
    }

    pub fn update(&mut self) {
        self.posts.iter_mut().for_each(|mut post| post.update());
    }

    pub fn view(&self) -> iced::Column<crate::app::Message> {
        self.posts
            .iter()
            .fold(iced::Column::new(), |c, post| {
                c.push(post.view())
            })
    }
}
