use std::sync::Mutex;

use article::Article;
use casopis::Casopis;
use config::Config;
use error::{Error, Result};
use scraper::{Html, Selector};
use webhook::{client::WebhookClient, models::Message};

mod article;
mod error;

#[tokio::main]
async fn main() {
    Casopis::init(log::Level::Info).unwrap();
    log::info!("Start relobo-wh!");

    let config = Config::builder()
        //.add_source(config::File::with_name("/etc/relobo/config.toml"))
        .add_source(config::File::with_name("./config.toml"))
        .build()
        .unwrap();
    let webhook_url: String = config.get("webhook").unwrap();
    let webhook = Mutex::new(WebhookClient::new(&webhook_url));

    let client = reqwest::Client::builder()
        .user_agent("Relobo, Discord anime news bot (If you want the bot to visit your site, please email me: vlobo2004@gmail.com)")
        .build()
        .unwrap();

    let last_article: Option<i64> = None;
    let articles: Mutex<Vec<Article>> = Mutex::new(Vec::new());

    loop {
        let request = client
            .get("https://shikimori.one/forum/news")
            .build()
            .unwrap();

        let response = client.execute(request).await.unwrap();
        let body = response.text().await.unwrap();
        let document = Html::parse_document(&body);
        let selector = Selector::parse("article").unwrap();

        'elem: for element in document.select(&selector) {
            let id: i64 = element.attr("id").unwrap().parse().unwrap();
            let url: String = element.attr("data-url").unwrap().parse().unwrap();
            for article in articles.lock().unwrap().iter() {
                if article.id == id {
                    continue 'elem;
                }
            }
            log::info!("New news {}", id);
            articles.lock().unwrap().push(Article::parse(id, url).await);
        }

        for article in articles.lock().unwrap().iter() {
            let message = article.message().unwrap();
            send_news(&webhook.lock().unwrap(), &message).await.unwrap();
        }

        std::thread::sleep(std::time::Duration::from_secs(300)); // 5 minutes
    }
}

async fn send_news(webhook: &WebhookClient, message: &Message) -> Result<()> {
    match webhook.send_message(&message).await {
        Err(error) => Err(Error::Webhook(error.to_string())),
        Ok(_) => Ok(()),
    }
}
