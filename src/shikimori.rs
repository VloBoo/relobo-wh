use html2md::parse_html;
use regex::Regex;
use reqwest::Client;
use serde_json::Value;
use url::Url;

use crate::article::Article;
use crate::error::{Error, Result};

pub struct Shikimori {
    client: Client,
}

impl Shikimori {
    pub fn new() -> Result<Self> {
        log::info!("Init shikimori.");
        let client = reqwest::Client::builder()
            .user_agent("Relobo")
            .build()
            .map_err(|e| Error::Shikimoti(format!("{:?}", e)))?;
        return Ok(Shikimori { client });
    }

    pub async fn get_ids(&self) -> Result<Vec<i64>> {
        let answer = Self::get(
            "https://shikimori.one/api/topics?forum=news&limit=5",
            self.client.clone(),
        )
        .await?;
        let articles_json = answer.as_array().ok_or(Error::SerdeJson(
            "Не удалось получить список новостей".to_owned(),
        ))?;
        let mut articles_id = vec![];
        for article_json in articles_json {
            let id = article_json["id"].as_i64().ok_or(Error::SerdeJson(
                "Не удалось получить айди новости".to_owned(),
            ))?;
            articles_id.push(id);
            log::info!("{:?}", id);
        }
        Ok(articles_id)
    }

    pub async fn get_article(&self, id: i64) -> Result<Article> {
        let answer = Self::get(
            &format!("https://shikimori.one/api/topics/{id}"),
            self.client.clone(),
        )
        .await?;

        let url = complete_url(
            "https://shikimori.one/",
            &format!(
                "{}/{}",
                answer["forum"]["url"].as_str().ok_or(Error::SerdeJson(
                    "Не удалось получить ссылку новости".to_owned(),
                ))?,
                id
            ),
        )?;
        let title = answer["topic_title"]
            .as_str()
            .ok_or(Error::SerdeJson("Не удалось получить заголовок".to_owned()))?
            .to_owned();
        let text = replace_relative_links(
            &parse_html(
                &answer["html_body"]
                    .as_str()
                    .ok_or(Error::SerdeJson("Не удалось получить заголовок".to_owned()))?
                    .to_owned(),
            ),
            "https://shikimori.one/",
        )?;
        let poster_url = match answer["linked"]["image"]["original"]
            .as_str()
            .map(|v| v.to_owned())
        {
            Some(value) => Some(complete_url("https://shikimori.one/", &value)?),
            None => None,
        };
        let data = answer["created_at"]
            .as_str()
            .ok_or(Error::SerdeJson(
                "Не удалось получить время создания".to_owned(),
            ))?
            .parse()
            .map_err(|e| Error::Shikimoti(format!("{:?}", e)))?;
        let article = Article {
            id,
            url,
            title,
            text,
            poster_url,
            data,
        };
        log::debug!("{:#?}", article);
        Ok(article)
    }

    async fn get(url: &str, client: Client) -> Result<Value> {
        let request = client
            .get(url)
            .build()
            .map_err(|e| Error::Shikimoti(format!("{:?}", e)))?;
        let response = client
            .execute(request)
            .await
            .map_err(|e| Error::Shikimoti(format!("{:?}", e)))?;
        let body = response
            .text()
            .await
            .map_err(|e| Error::Shikimoti(format!("{:?}", e)))?;
        Ok(serde_json::from_str(&body).map_err(|e| Error::SerdeJson(format!("{:?}", e)))?)
    }
}

fn complete_url(base_url: &str, url_str: &str) -> Result<String> {
    Ok(match Url::parse(url_str) {
        Ok(_) => url_str.to_string(),
        Err(_) => {
            let base = Url::parse(base_url).map_err(|e| Error::Other(format!("{:?}", e)))?;
            base.join(url_str)
                .map_err(|e| Error::Other(format!("{:?}", e)))?
                .to_string()
        }
    })
}

fn replace_relative_links(text: &str, base_url: &str) -> Result<String> {
    let re = Regex::new(r"\[(.*?)\]\((.*?)\)").unwrap();
    Ok(re
        .replace_all(text, |caps: &regex::Captures| {
            format!(
                "[{}]({})",
                &caps[1],
                match complete_url(base_url, &caps[2]) {
                    Ok(value) => value,
                    Err(_) => (&caps[2]).to_string(),
                }
            )
        })
        .to_string())
}
