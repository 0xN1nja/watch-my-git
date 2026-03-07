mod config;
mod db;
mod discord;
mod github;

use chrono::Local;
use std::time::Duration;
use tokio::time::sleep;

use config::Config;
use db::Db;
use discord::DiscordClient;
use github::GithubClient;

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║          watch-my-git                ║");
    println!("╚══════════════════════════════════════╝");

    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("[error] Failed to load config: {}", e);
        std::process::exit(1);
    });

    let db = Db::open(&config.db_path).unwrap_or_else(|e| {
        eprintln!("[error] Failed to open database: {}", e);
        std::process::exit(1);
    });

    let github = GithubClient::new(config.github_username.clone(), config.github_token.clone());
    let discord = DiscordClient::new(config.discord_webhook_url.clone());

    println!("[info] Tracking: @{}", config.github_username);
    println!(
        "[info] Check interval: {} seconds",
        config.check_interval_seconds
    );
    println!("[info] Starting first check...\n");

    loop {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S");
        println!("[{}] Running check...", now);

        match run_check(&config, &db, &github, &discord).await {
            Ok(outcome) => println!("[{}] {}", now, outcome),
            Err(e) => eprintln!("[{}] Check failed: {}", now, e),
        }

        let interval = Duration::from_secs(config.check_interval_seconds);
        println!(
            "[info] Next check in {} seconds.\n",
            config.check_interval_seconds
        );
        sleep(interval).await;
    }
}

async fn run_check(
    config: &Config,
    db: &Db,
    github: &GithubClient,
    discord: &DiscordClient,
) -> Result<String, String> {
    let current_count = github
        .fetch_follower_count()
        .await
        .map_err(|e| e.to_string())?;

    let last_count = db.get_last_follower_count().map_err(|e| e.to_string())?;

    match last_count {
        None => {
            println!("[info] First run — fetching and caching all followers...");

            let followers = github
                .fetch_all_followers()
                .await
                .map_err(|e| e.to_string())?;

            db.set_followers(&followers).map_err(|e| e.to_string())?;
            db.set_last_follower_count(followers.len())
                .map_err(|e| e.to_string())?;

            return Ok(format!(
                "First run complete. Cached {} follower(s). Will detect unfollowers from next check.",
                followers.len()
            ));
        }

        Some(last) if last == current_count => {
            return Ok(format!(
                "No change detected ({} followers). Skipping full fetch.",
                current_count
            ));
        }

        Some(last) => {
            println!(
                "[info] Follower count changed: {} → {}. Fetching full list...",
                last, current_count
            );
        }
    }

    let old_followers = db.get_followers().map_err(|e| e.to_string())?;

    let new_followers = github
        .fetch_all_followers()
        .await
        .map_err(|e| e.to_string())?;

    let unfollowers = Db::find_unfollowers(&old_followers, &new_followers);

    if !unfollowers.is_empty() {
        println!(
            "[info] {} unfollower(s) detected: {}",
            unfollowers.len(),
            unfollowers
                .iter()
                .map(|f| f.login.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );

        discord
            .notify_unfollowers(&unfollowers, &config.github_username)
            .await
            .map_err(|e| e.to_string())?;

        println!("[info] Discord notification sent.");
    }

    db.set_followers(&new_followers)
        .map_err(|e| e.to_string())?;
    db.set_last_follower_count(new_followers.len())
        .map_err(|e| e.to_string())?;

    let gained =
        (new_followers.len() as i64) - (old_followers.len() as i64) + unfollowers.len() as i64;

    Ok(format!(
        "Done. {} follower(s) now. {} unfollowed, {} new.",
        new_followers.len(),
        unfollowers.len(),
        gained.max(0)
    ))
}
