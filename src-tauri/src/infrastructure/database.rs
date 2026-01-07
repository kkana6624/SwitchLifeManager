use crate::domain::models::{SessionKeyStats, SessionRecord};
use crate::domain::repositories::SessionRepository;
use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_url = format!("sqlite://{}", db_path.as_ref().to_string_lossy());

        // Ensure file exists if not memory
        if db_url != "sqlite::memory:" {
            if let Some(parent) = db_path.as_ref().parent() {
                std::fs::create_dir_all(parent).context("Failed to create database directory")?;
            }

            // Create empty file if not exists to allow sqlx to connect
            if !db_path.as_ref().exists() {
                std::fs::File::create(db_path.as_ref())
                    .context("Failed to create database file")?;
            }
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .context("Failed to connect to SQLite")?;

        let repo = Self { pool };
        repo.migrate().await?;
        Ok(repo)
    }

    pub async fn new_in_memory() -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .context("Failed to connect to in-memory SQLite")?;

        let repo = Self { pool };
        repo.migrate().await?;
        Ok(repo)
    }

    async fn migrate(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                start_time TEXT NOT NULL,
                end_time TEXT NOT NULL,
                duration_secs INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS session_keys (
                session_id INTEGER NOT NULL,
                key_name TEXT NOT NULL,
                presses INTEGER NOT NULL,
                chatters INTEGER NOT NULL,
                chatter_releases INTEGER NOT NULL,
                FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE,
                PRIMARY KEY(session_id, key_name)
            );",
        )
        .execute(&self.pool)
        .await
        .context("Failed to run migrations")?;
        Ok(())
    }
}

#[async_trait]
impl SessionRepository for SqliteRepository {
    async fn save(&self, session: &SessionRecord, stats: &[SessionKeyStats]) -> Result<i64> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to begin transaction")?;

        let id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO sessions (start_time, end_time, duration_secs) VALUES (?, ?, ?) RETURNING id",
        )
        .bind(session.start_time.to_rfc3339())
        .bind(session.end_time.to_rfc3339())
        .bind(session.duration_secs as i64)
        .fetch_one(&mut *tx)
        .await
        .context("Failed to insert session")?;

        for stat in stats {
            sqlx::query(
                "INSERT INTO session_keys (session_id, key_name, presses, chatters, chatter_releases) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(id)
            .bind(&stat.key_name)
            .bind(stat.presses as i64)
            .bind(stat.chatters as i64)
            .bind(stat.chatter_releases as i64)
            .execute(&mut *tx)
            .await
            .context("Failed to insert session key stat")?;
        }

        tx.commit().await.context("Failed to commit transaction")?;
        Ok(id)
    }

    async fn get_recent(&self, limit: i64, offset: i64) -> Result<Vec<SessionRecord>> {
        let rows = sqlx::query(
            "SELECT id, start_time, end_time, duration_secs FROM sessions ORDER BY start_time DESC LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch recent sessions")?;

        let mut sessions = Vec::new();
        for row in rows {
            use sqlx::Row;
            let id: i64 = row.get("id");
            let start_str: String = row.get("start_time");
            let end_str: String = row.get("end_time");
            let duration_secs: i64 = row.get("duration_secs");

            sessions.push(SessionRecord {
                id: Some(id),
                start_time: chrono::DateTime::parse_from_rfc3339(&start_str)?
                    .with_timezone(&chrono::Utc),
                end_time: chrono::DateTime::parse_from_rfc3339(&end_str)?
                    .with_timezone(&chrono::Utc),
                duration_secs: duration_secs as u64,
            });
        }

        Ok(sessions)
    }

    async fn get_details(&self, session_id: i64) -> Result<Vec<SessionKeyStats>> {
        let rows = sqlx::query(
            "SELECT session_id, key_name, presses, chatters, chatter_releases FROM session_keys WHERE session_id = ?",
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch session details")?;

        let mut stats = Vec::new();
        for row in rows {
            use sqlx::Row;
            stats.push(SessionKeyStats {
                session_id: row.get("session_id"),
                key_name: row.get("key_name"),
                presses: row.get::<i64, _>("presses") as u64,
                chatters: row.get::<i64, _>("chatters") as u64,
                chatter_releases: row.get::<i64, _>("chatter_releases") as u64,
            });
        }

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_save_and_retrieve_session() {
        let repo = SqliteRepository::new_in_memory().await.unwrap();

        let session = SessionRecord {
            id: None,
            start_time: Utc::now(),
            end_time: Utc::now(),
            duration_secs: 60,
        };

        let stats = vec![
            SessionKeyStats {
                session_id: 0, // Ignored on insert
                key_name: "Key1".to_string(),
                presses: 100,
                chatters: 5,
                chatter_releases: 2,
            },
            SessionKeyStats {
                session_id: 0,
                key_name: "Key2".to_string(),
                presses: 200,
                chatters: 0,
                chatter_releases: 0,
            },
        ];

        // Test Save
        let id = repo.save(&session, &stats).await.unwrap();
        assert!(id > 0);

        // Test Retrieve Recent
        let recent = repo.get_recent(10, 0).await.unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].duration_secs, 60);

        // Test Retrieve Details
        let details = repo.get_details(id).await.unwrap();
        assert_eq!(details.len(), 2);

        let k1 = details.iter().find(|s| s.key_name == "Key1").unwrap();
        assert_eq!(k1.presses, 100);
        assert_eq!(k1.chatters, 5);

        let k2 = details.iter().find(|s| s.key_name == "Key2").unwrap();
        assert_eq!(k2.presses, 200);
    }
}
