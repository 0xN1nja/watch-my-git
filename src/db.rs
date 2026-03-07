use chrono::Utc;
use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone)]
pub struct Follower {
    pub login: String,
    pub avatar_url: String,
    pub html_url: String,
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open() -> Result<Self> {
        let conn = Connection::open("followers.db")?;
        let db = Db { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS followers (
                login       TEXT PRIMARY KEY,
                avatar_url  TEXT NOT NULL,
                html_url    TEXT NOT NULL,
                first_seen  TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS meta (
                key         TEXT PRIMARY KEY,
                value       TEXT NOT NULL
            );
        ",
        )?;
        Ok(())
    }

    pub fn get_followers(&self) -> Result<Vec<Follower>> {
        let mut stmt = self
            .conn
            .prepare("SELECT login, avatar_url, html_url FROM followers")?;

        let followers = stmt
            .query_map([], |row| {
                Ok(Follower {
                    login: row.get(0)?,
                    avatar_url: row.get(1)?,
                    html_url: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(followers)
    }

    pub fn set_followers(&self, followers: &[Follower]) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        self.conn.execute_batch("DELETE FROM followers")?;

        for f in followers {
            self.conn.execute(
                "INSERT OR IGNORE INTO followers (login, avatar_url, html_url, first_seen)
                 VALUES (?1, ?2, ?3, ?4)",
                params![f.login, f.avatar_url, f.html_url, now],
            )?;
        }

        Ok(())
    }

    pub fn get_last_follower_count(&self) -> Result<Option<usize>> {
        let result = self.conn.query_row(
            "SELECT value FROM meta WHERE key = 'follower_count'",
            [],
            |row| row.get::<_, String>(0),
        );

        match result {
            Ok(val) => Ok(val.parse::<usize>().ok()),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn set_last_follower_count(&self, count: usize) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('follower_count', ?1)",
            params![count.to_string()],
        )?;
        Ok(())
    }

    pub fn find_unfollowers(old: &[Follower], new: &[Follower]) -> Vec<Follower> {
        let new_logins: std::collections::HashSet<&str> =
            new.iter().map(|f| f.login.as_str()).collect();

        old.iter()
            .filter(|f| !new_logins.contains(f.login.as_str()))
            .cloned()
            .collect()
    }
}
