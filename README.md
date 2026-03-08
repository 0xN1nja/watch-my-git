# watch-my-git

Get notified when someone unfollows you on GitHub.

## Setup

Before running, you'll need:

- A [GitHub Fine-grained Personal Access Token](https://github.com/settings/personal-access-tokens/new) with **Followers** permission
- A [Discord Webhook URL](https://support.discord.com/hc/en-us/articles/228383668-Intro-to-Webhooks)

Use the `.env.example` file in the repository to setup a `.env` file.

## Configuration

| Variable              | Required | Default | Description               |
| --------------------- | -------- | ------- | ------------------------- |
| `GITHUB_USERNAME`     | ✅       | —       | GitHub username to track  |
| `GITHUB_TOKEN`        | ✅       | —       | Personal access token     |
| `DISCORD_WEBHOOK_URL` | ✅       | —       | Discord webhook URL       |
| `CHECK_INTERVAL_SECS` | ❌       | `3600`  | Check interval in seconds |

## Installation

### Docker (recommended)

Create a `docker-compose.yaml`:

```yaml
services:
  watch-my-git:
    image: ghcr.io/0xn1nja/watch-my-git:latest
    container_name: watch-my-git
    restart: unless-stopped
    volumes:
      - ./data:/app/data
    env_file:
      - .env
    environment:
      - FOLLOWERS_DB_PATH=/app/data/followers.db
```

Then run:

```bash
docker compose up -d
```

The container runs forever, restarts on crash, and persists the DB in `./data/`.

### Local build

Requires [Rust](https://rustup.rs) (stable).

```bash
git clone https://github.com/0xN1nja/watch-my-git.git
cd watch-my-git
cargo run --release
```
