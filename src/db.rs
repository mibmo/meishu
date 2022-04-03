use crate::models::Score;
use crate::utils::Timestamp;

use eyre::{Result as EResult, WrapErr};
use sqlx::{postgres::*, Result as SQLResult, Row};
use tracing::*;

pub struct Db {
    pub pool: PgPool,
}

#[derive(Debug)]
pub struct FilterOptions {
    pub since: Option<Timestamp>,
    pub username: Option<String>,
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
        trace!(?options, "fetching Scores");

        let mut query = String::from(
            r#"
                SELECT id, username, score, scored_at
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
        query.push_str("ORDER BY id");

        sqlx::query_as::<_, Score>(&query)
            .bind(options.since)
            .bind(options.username)
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
