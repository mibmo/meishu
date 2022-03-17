use crate::models::Score;
use crate::utils::Timestamp;

use eyre::{Result as EResult, WrapErr};
use sqlx::postgres::*;
use tracing::*;

pub struct Db {
    pub pool: PgPool,
}

impl Db {
    pub async fn insert_score(&self, name: &str, score: i64) -> EResult<bool> {
        trace!(?name, ?score, "inserting Score");

        let affected = sqlx::query(
            r#"
                INSERT INTO scores ( username, score )
                VALUES ( $1, $2 )
                RETURNING id
            "#,
        )
        .bind(name)
        .bind(score)
        .execute(&self.pool)
        .await
        .wrap_err_with(|| format!("Failed to create Score with name \"{name}\" and score {score}"))?
        .rows_affected();

        Ok(affected == 1)
    }

    pub async fn delete_score(&self, id: i64) -> EResult<bool> {
        trace!(?id, "deleting Score");

        let affected = sqlx::query(
            r#"
                DELETE FROM scores
                WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .wrap_err_with(|| format!("Failed to delete Score with id {id}"))?
        .rows_affected();

        Ok(affected == 1)
    }

    pub async fn get_all_scores(&self) -> EResult<Vec<Score>> {
        trace!("fetching all Scores");

        sqlx::query_as::<_, Score>(
            r#"
                SELECT id, username, score, scored_at
                FROM scores
                ORDER BY score
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .wrap_err_with(|| format!("Failed to fetch all Scores"))
    }

    pub async fn get_scores_after_timestamp(&self, timestamp: Timestamp) -> EResult<Vec<Score>> {
        trace!(?timestamp, "fetching Scores after timestamp");

        sqlx::query_as::<_, Score>(
            r#"
                SELECT id, username, score, scored_at
                FROM scores
                WHERE scored_at <= $1
                ORDER BY score
            "#,
        )
        .bind(timestamp)
        .fetch_all(&self.pool)
        .await
        .wrap_err_with(|| format!("Failed to fetch Scores after timestamp"))
    }
}
