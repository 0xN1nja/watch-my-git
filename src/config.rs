use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub github_username: String,
    pub github_token: String,
    pub discord_webhook_url: String,
    pub check_interval_seconds: u64,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let github_username =
            env::var("GITHUB_USERNAME").map_err(|_| "Missing GITHUB_USERNAME in .env")?;

        let github_token = env::var("GITHUB_TOKEN").map_err(|_| "Missing GITHUB_TOKEN in .env")?;

        let discord_webhook_url =
            env::var("DISCORD_WEBHOOK_URL").map_err(|_| "Missing DISCORD_WEBHOOK_URL in .env")?;

        let check_interval_seconds = env::var("CHECK_INTERVAL_SECONDS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<u64>()
            .map_err(|_| "CHECK_INTERVAL_SECONDS must be a valid number")?;

        Ok(Config {
            github_username,
            github_token,
            discord_webhook_url,
            check_interval_seconds,
        })
    }
}
