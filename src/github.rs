use crate::db::Follower;
use reqwest::Client;
use serde::Deserialize;

const PER_PAGE: u32 = 100;

#[derive(Debug, Deserialize)]
struct GithubFollower {
    login: String,
    avatar_url: String,
    html_url: String,
}

#[derive(Debug)]
pub enum GithubError {
    Http(reqwest::Error),
    RateLimited,
    Unauthorized,
    UserNotFound(String),
    Unknown(u16, String),
}

impl std::fmt::Display for GithubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GithubError::Http(e) => write!(f, "HTTP error: {e}"),
            GithubError::RateLimited => write!(f, "GitHub API rate limit hit — try again later"),
            GithubError::Unauthorized => write!(f, "GitHub token is invalid or expired"),
            GithubError::UserNotFound(u) => write!(f, "GitHub user '{}' not found", u),
            GithubError::Unknown(code, msg) => write!(f, "Unexpected response {code}: {msg}"),
        }
    }
}

impl From<reqwest::Error> for GithubError {
    fn from(e: reqwest::Error) -> Self {
        GithubError::Http(e)
    }
}

pub struct GithubClient {
    client: Client,
    token: String,
    username: String,
}

impl GithubClient {
    pub fn new(username: String, token: String) -> Self {
        let client = Client::builder()
            .user_agent("watch-my-git/0.1")
            .build()
            .expect("Failed to build HTTP client");

        GithubClient {
            client,
            token,
            username,
        }
    }

    pub async fn fetch_all_followers(&self) -> Result<Vec<Follower>, GithubError> {
        let mut all_followers: Vec<Follower> = Vec::new();
        let mut page = 1u32;

        loop {
            let url = format!(
                "https://api.github.com/users/{}/followers?per_page={}&page={}",
                self.username, PER_PAGE, page
            );

            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .header("Accept", "application/vnd.github+json")
                .header("X-GitHub-Api-Version", "2022-11-28")
                .send()
                .await?;

            match response.status().as_u16() {
                401 => return Err(GithubError::Unauthorized),
                403 => return Err(GithubError::RateLimited),
                404 => return Err(GithubError::UserNotFound(self.username.clone())),
                200 => {}
                code => {
                    let body = response.text().await.unwrap_or_default();
                    return Err(GithubError::Unknown(code, body));
                }
            }

            let page_followers: Vec<GithubFollower> = response.json().await?;

            if page_followers.is_empty() {
                break;
            }

            let fetched = page_followers.len();

            all_followers.extend(page_followers.into_iter().map(|f| Follower {
                login: f.login,
                avatar_url: f.avatar_url,
                html_url: f.html_url,
            }));

            if fetched < PER_PAGE as usize {
                break;
            }

            page += 1;
        }

        Ok(all_followers)
    }

    pub async fn fetch_follower_count(&self) -> Result<usize, GithubError> {
        let url = format!("https://api.github.com/users/{}", self.username);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await?;

        match response.status().as_u16() {
            401 => return Err(GithubError::Unauthorized),
            403 => return Err(GithubError::RateLimited),
            404 => return Err(GithubError::UserNotFound(self.username.clone())),
            200 => {}
            code => {
                let body = response.text().await.unwrap_or_default();
                return Err(GithubError::Unknown(code, body));
            }
        }

        #[derive(Deserialize)]
        struct UserResponse {
            followers: usize,
        }

        let user: UserResponse = response.json().await?;
        Ok(user.followers)
    }
}
