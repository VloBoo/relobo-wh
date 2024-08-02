use chrono::{DateTime, Utc};
use html2md::parse_html;
use scraper::{Html, Selector};
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
    pub async fn parse<'a>(id: i64, url: String) -> Self {
        //let url = element.select(&Selector::parse("article").unwrap()).enumerate().last();

        let client = reqwest::Client::builder()
        .user_agent("Relobo, Discord anime news bot (If you want the bot to visit your site, please email me: vlobo2004@gmail.com)")
        .build()
        .unwrap();
        let request = client.get(url.clone()).build().unwrap();
        let response = client.execute(request).await.unwrap();
        let body = response.text().await.unwrap();
        let document = Html::parse_document(&body);

        //log::info!("{:#?}", body);

        let title = document
            .select(&Selector::parse("h1").unwrap())
            .enumerate()
            .last()
            .unwrap()
            .1
            .text()
            .collect();
        let text = parse_html(
            document
                .select(&Selector::parse("div.body-inner").unwrap())
                .enumerate()
                .last()
                .unwrap()
                .1
                .inner_html()
                .as_str(),
        );
        let poster_url = match document
            .select(&Selector::parse(".b-shiki_wall > .b-image").unwrap())
            .enumerate()
            .next()
        {
            Some(value) => Some(value.1.attr("href").unwrap().to_string()),
            None => None,
        };
        let data = document
            .select(&Selector::parse(".section.created_at > time").unwrap())
            .enumerate()
            .last()
            .unwrap()
            .1
            .attr("datetime")
            .unwrap();
        Article {
            id,
            url: url.clone(),
            title,
            text,
            poster_url,
            data: data.parse().unwrap(),
        }
    }

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
            });
        return Ok(message);
    }
}
