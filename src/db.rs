use crate::models::Score;
use crate::utils::Timestamp;

use eyre::{Result as EResult, WrapErr};
use sqlx::{postgres::*, Result as SQLResult, Row};
use tracing::*;

pub struct Db {
    pub pool: PgPool,
}

#[derive(Debug, Default)]
pub struct FilterOptions {
    pub since: Option<Timestamp>,
    pub username: Option<String>,
    pub pending: Option<bool>,
}

impl Db {
    pub async fn insert_score(&self, name: &str, score: i64) -> SQLResult<i64> {
        trace!(?name, ?score, "inserting Score");

        sqlx::query(
            r#"
                INSERT INTO scores ( username, score )
                VALUES ( $1, $2 )
                RETURNING id
            "#,
        )
        .bind(name)
        .bind(score)
        .fetch_one(&self.pool)
        .await
        .map(|row| row.get(0))
    }

    pub async fn insert_pending_score(&self, score: i64) -> SQLResult<i64> {
        trace!(?score, "inserting pending Score");

        sqlx::query(
            r#"
                INSERT INTO scores ( score, pending )
                VALUES ( $1, true )
                RETURNING id
            "#,
        )
        .bind(score)
        .fetch_one(&self.pool)
        .await
        .map(|row| row.get(0))
    }

    pub async fn finalize_score(&self, id: i64, name: &str) -> SQLResult<bool> {
        trace!(?id, ?name, "finalizing score");

        let affected = sqlx::query(
            r#"
                UPDATE scores
                SET pending = false, username = $2
                WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(name)
        .execute(&self.pool)
        .await?
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

    pub async fn get_scores(&self, options: FilterOptions) -> SQLResult<Vec<Score>> {
        debug!(?options, "fetching Scores");

        let mut query = String::from(
            r#"
                SELECT id, username, score, scored_at, pending
                FROM scores
                WHERE 1=1
        "#,
        );
        if options.since.is_some() {
            query.push_str("AND scored_at >= $1");
        }
        if options.username.is_some() {
            query.push_str("AND username = $2");
        }
        if options.pending.is_some() {
            query.push_str("AND pending = $3");
        }
        query.push_str("ORDER BY id");

        sqlx::query_as::<_, Score>(&query)
            .bind(options.since)
            .bind(options.username)
            .bind(options.pending)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn get_score_by_id(&self, id: i64) -> SQLResult<Score> {
        trace!(?id, "getting score");

        sqlx::query_as::<_, Score>(
            r#"
                SELECT *
                FROM scores
                WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }
}
