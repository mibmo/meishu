use chrono::prelude::*;
use sqlx::prelude::FromRow;

#[derive(Debug, FromRow)]
pub struct Score {
    pub score: i64,
    pub username: String,
    pub scored_at: DateTime<Utc>,
}
