use crate::db::Db;

use eyre::Result as EResult;
use std::sync::Arc;
use warp::Filter;

async fn add_score_handler(
    db: Arc<Db>,
    name: String,
    score: i64,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    match db.insert_score(&name, score).await {
        Ok(true) => Ok("created score"),
        Ok(false) | Err(_) => Ok("failed to create score"),
    }
}

async fn delete_score_handler(
    db: Arc<Db>,
    id: i64,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    match db.delete_score(id).await {
        Ok(true) => Ok("deleted score"),
        Ok(false) => Ok("score does not exist"),
        Err(_) => Ok("failed to delete score"),
    }
}

async fn get_all_scores_handler(db: Arc<Db>) -> Result<impl warp::Reply, std::convert::Infallible> {
    Ok(match db.get_all_scores().await {
        Err(_) => "failed to get all scores".to_string(),
        Ok(scores) => scores
            .into_iter()
            .map(|score| {
                format!(
                    "[{scored_at}] {name:14} {value:10}\n",
                    scored_at = score.scored_at,
                    name = score.username,
                    value = score.score
                )
            })
            .collect::<String>(),
    })
}

pub async fn serve(db: Db) -> EResult<()> {
    let db = Arc::new(db);
    let db_hook = warp::any().map(move || Arc::clone(&db));

    let score_get_all = warp::get()
        .and(db_hook.clone())
        .and_then(get_all_scores_handler);

    let score_add = warp::post()
        .and(db_hook.clone())
        .and(warp::path::param::<String>())
        .and(warp::path::param::<i64>())
        .and_then(add_score_handler);

    let score_delete = warp::delete()
        .and(db_hook.clone())
        .and(warp::path::param::<i64>())
        .and_then(delete_score_handler);

    let score = warp::path("score").and(score_get_all.or(score_add.or(score_delete)));

    let ws = warp::path("ws").map(|| "ws upgrade");

    let routes = ws.or(score);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}
