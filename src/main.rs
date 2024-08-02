use std::{os::windows::raw::SOCKET, sync::Mutex};

use article::Article;
use casopis::Casopis;
use chrono::{DateTime, Utc};
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

    let mut last_time: Option<DateTime<Utc>> = None;
    let articles: Mutex<Vec<Article>> = Mutex::new(Vec::new());

    update_articles(&articles).await;

    for article in articles.lock().unwrap().iter() {
        if let Some(time) = last_time {
            if (article.data > time) {
                last_time = Some(article.data);
            }
        } else {
            last_time = Some(article.data);
        }
    }

    loop {
        log::warn!("last {:?}", last_time);
        update_articles(&articles).await;
        let mut newest_time: Option<DateTime<Utc>> = None;
        for article in articles.lock().unwrap().iter() {
            if let Some(time) = last_time {
                if article.data <= time {
                    continue;
                } else {
                    newest_time = Some(article.data);
                }
            }

            if let Some(time) = newest_time {
                if article.data > time {
                    newest_time = Some(article.data);
                }
            }

            let message = article.message().unwrap();
            log::info!("Sending article {}", article.id);
            log::warn!("{:?}", article.data);
            send_news(&webhook.lock().unwrap(), &message).await.unwrap();
            std::thread::sleep(std::time::Duration::from_secs(1)); // 1 sec
        }

        last_time = match newest_time {
            Some(time) => Some(time),
            None => last_time,
        };

        std::thread::sleep(std::time::Duration::from_secs(300)); // 5 minutes
    }
}

async fn update_articles(articles: &Mutex<Vec<Article>>) {
    let mut articles = articles.lock().unwrap();

    let client = reqwest::Client::builder()
    .user_agent("Relobo, Discord anime news bot (If you want the bot to visit your site, please email me: vlobo2004@gmail.com)")
    .build()
    .unwrap();
    let request = client
        .get("https://shikimori.one/forum/news")
        .build()
        .unwrap();
    let response = client.execute(request).await.unwrap();
    let body = response.text().await.unwrap();
    let document = Html::parse_document(&body);
    let selector = Selector::parse("article").unwrap();

    'elem: for element in document.select(&selector) {
        std::thread::sleep(std::time::Duration::from_secs(1)); // 1 sec
        let id: i64 = element.attr("id").unwrap().parse().unwrap();
        let url: String = element.attr("data-url").unwrap().parse().unwrap();
        for article in articles.iter() {
            if article.id == id {
                continue 'elem;
            }
        }
        log::info!("Created {}", id);
        articles.push(Article::parse(id, url).await);
    }

    articles.sort_by(|ar1, ar2| ar1.data.cmp(&ar2.data));
}

async fn send_news(webhook: &WebhookClient, message: &Message) -> Result<()> {
    match webhook.send_message(&message).await {
        Err(error) => Err(Error::Webhook(error.to_string())),
        Ok(_) => Ok(()),
    }
}
