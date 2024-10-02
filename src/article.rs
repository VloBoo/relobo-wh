use chrono::{DateTime, Utc};
use webhook::models::Message;

use crate::error::Result;

#[derive(Clone, Debug)]
pub struct Article {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub text: String,
    pub poster_url: Option<String>,
    pub data: DateTime<Utc>,
}

impl Article {
    pub fn message(&self) -> Result<Message> {
        let image_url = "https://vlobo.site/1-64.png";

        let mut message = Message::new();
        message
            .username("Relobo")
            .avatar_url(&image_url)
            .embed(|embed| {
                if let Some(poster_url) = &self.poster_url {
                    embed.image(&poster_url);
                }
                embed
                    .title(&self.title)
                    .description(&self.text)
                    .footer(&format!("Оригинал: {}", self.url), None)
                    .color("6316287")
                //.image(IMAGE_URL)
                //.thumbnail(IMAGE_URL)
                //.author("Lmao#0001", Some(String::from(IMAGE_URL)), Some(String::from(IMAGE_URL)))
                //.field("name", "value", false)
            })
            //.action_row(|action_row| {
            //    action_row.link_button(|button| button.url(&self.url).label("Читать оригинал"))
            //})
            ;
        return Ok(message);
    }
}
