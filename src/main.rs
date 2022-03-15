mod db;
mod models;
mod utils;
mod web;

use db::Db;

use eyre::Result as EResult;
use sqlx::postgres::*;
use tracing::*;

#[tokio::main]
async fn main() -> EResult<()> {
    utils::setup_tracing_subscriber()?;

    let connection_uri = utils::db_uri()?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection_uri)
        .await?;

    info!("running database migrations");
    sqlx::migrate!().run(&pool).await?;

    let db = Db { pool: pool };
    web::serve(db).await
}
