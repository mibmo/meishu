use chrono::prelude::*;
use serde::Serialize;
use sqlx::prelude::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct Score {
    pub id: i64,
    pub score: i64,
    pub username: String,
    pub scored_at: DateTime<Utc>,
}
