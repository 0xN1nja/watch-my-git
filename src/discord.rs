use crate::db::Follower;
use reqwest::Client;
use serde_json::{Value, json};

#[derive(Debug)]
pub enum DiscordError {
    Http(reqwest::Error),
    BadWebhook,
    RateLimited,
    Unknown(u16),
}

impl std::fmt::Display for DiscordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscordError::Http(e) => write!(f, "HTTP error: {e}"),
            DiscordError::BadWebhook => write!(f, "Webhook URL is invalid or deleted"),
            DiscordError::RateLimited => write!(f, "Discord rate limit hit"),
            DiscordError::Unknown(c) => write!(f, "Unexpected Discord response: {c}"),
        }
    }
}

impl From<reqwest::Error> for DiscordError {
    fn from(e: reqwest::Error) -> Self {
        DiscordError::Http(e)
    }
}

pub struct DiscordClient {
    client: Client,
    webhook_url: String,
}

impl DiscordClient {
    pub fn new(webhook_url: String) -> Self {
        DiscordClient {
            client: Client::new(),
            webhook_url,
        }
    }

    pub async fn notify_unfollowers(
        &self,
        unfollowers: &[Follower],
        target_username: &str,
    ) -> Result<(), DiscordError> {
        for chunk in unfollowers.chunks(10) {
            let embeds: Vec<Value> = chunk.iter().map(|f| build_embed(f)).collect();

            let payload = json!({
                "username": "watch-my-git",
                "avatar_url": "https://github.githubassets.com/images/modules/logos_page/GitHub-Mark.png",
                "content": format!(
                    "**{}** lost **{}** follower{} just now.",
                    target_username,
                    unfollowers.len(),
                    if unfollowers.len() == 1 { "" } else { "s" }
                ),
                "embeds": embeds
            });

            self.send_payload(&payload).await?;
        }

        Ok(())
    }

    async fn send_payload(&self, payload: &Value) -> Result<(), DiscordError> {
        let response = self
            .client
            .post(&self.webhook_url)
            .json(payload)
            .send()
            .await?;

        match response.status().as_u16() {
            200 | 204 => Ok(()),
            400 | 404 => Err(DiscordError::BadWebhook),
            429 => Err(DiscordError::RateLimited),
            code => Err(DiscordError::Unknown(code)),
        }
    }
}

fn build_embed(follower: &Follower) -> Value {
    json!({
        "author": {
            "name": follower.login,
            "url": follower.html_url,
            "icon_url": follower.avatar_url
        },
        "description": format!(
            "[@{}]({}) has unfollowed you.",
            follower.login, follower.html_url
        ),
        "color": 0xFF4444,
        "thumbnail": {
            "url": follower.avatar_url
        },
        "footer": {
            "text": "watch-my-git"
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    })
}
