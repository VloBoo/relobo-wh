use article::Article;
use casopis::Casopis;
use chrono::{DateTime, Utc};
use config::Config;
use error::{Error, Result};
use shikimori::Shikimori;
use webhook::{client::WebhookClient, models::Message};

mod article;
mod error;
mod shikimori;

#[tokio::main]
async fn main() {
    Casopis::init(log::Level::Info).unwrap();
    log::info!("Start relobo-wh!");

    let config = Config::builder()
        .add_source(config::File::with_name("/etc/relobo/config.toml"))
        //.add_source(config::File::with_name("./config.toml"))
        .build()
        .unwrap();
    let webhook_url: String = config.get("webhook").unwrap();
    let duration: u64 = config.get("duration").unwrap();

    let webhook = WebhookClient::new(&webhook_url);

    let shikimori = Shikimori::new().unwrap();

    let mut last_time: Option<DateTime<Utc>> = None;
    let mut articles: Vec<Article> = vec![];

    update_articles(&shikimori, &mut articles).await;

    for article in articles.iter() {
        if let Some(time) = last_time {
            if article.data > time {
                last_time = Some(article.data);
            }
        } else {
            last_time = Some(article.data);
        }
    }

    loop {
        log::debug!("Update with time {:?}", last_time);
        update_articles(&shikimori, &mut articles).await;
        let mut newest_time: Option<DateTime<Utc>> = None;

        for article in articles.iter() {
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
            log::info!("Sending article {}, {}", article.id, article.data);
            send_news(&webhook, &message).await.unwrap();
        }

        last_time = match newest_time {
            Some(time) => Some(time),
            None => last_time,
        };

        std::thread::sleep(std::time::Duration::from_secs(duration)); // 15 minutes
    }
}

async fn update_articles(shikimori: &Shikimori, articles: &mut Vec<Article>) {
    //let mut articles = articles.lock().unwrap();
    let articles_id = shikimori.get_ids().await.unwrap();

    'elem: for article_id in articles_id {
        for article in articles.iter() {
            if article.id == article_id {
                continue 'elem;
            }
        }
        log::info!("Created {}", article_id);
        let article = match shikimori.get_article(article_id).await {
            Ok(value) => value,
            Err(error) => {
                log::error!("Не удалось создать статью: {:?}", error);
                continue 'elem;
            }
        };
        articles.push(article);
    }

    articles.sort_by(|ar1, ar2| ar2.data.cmp(&ar1.data));
}

async fn send_news(webhook: &WebhookClient, message: &Message) -> Result<()> {
    webhook
        .send_message(&message)
        .await
        .map_err(|e| Error::Webhook(format!("{:?}", e)))?;
    return Ok(());
}
