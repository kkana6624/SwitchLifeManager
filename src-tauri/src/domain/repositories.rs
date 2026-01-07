use crate::domain::models::{SessionKeyStats, SessionRecord};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Saves a completed session and its key stats to the database.
    /// Returns the ID of the inserted session.
    async fn save(&self, session: &SessionRecord, stats: &[SessionKeyStats]) -> Result<i64>;

    /// Retrieves recent sessions with pagination.
    async fn get_recent(&self, limit: i64, offset: i64) -> Result<Vec<SessionRecord>>;

    /// Retrieves detailed key statistics for a specific session.
    async fn get_details(&self, session_id: i64) -> Result<Vec<SessionKeyStats>>;
}
