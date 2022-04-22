use crate::db::{Db, FilterOptions};
use crate::models::Score;
use crate::utils::env_var;

use askama::Template;
use chrono::{DateTime, NaiveDateTime, Utc};
use eyre::{Result as EResult, WrapErr};
use serde::Deserialize;
use sqlx::Error as SQLError;
use std::sync::Arc;
use tracing::*;
use warp::{
    http::StatusCode,
    reply::{html, json, Reply, Response},
    Filter,
};

#[derive(Template)]
#[template(path = "leaderboard.html")]
struct LeaderboardTemplate {
    scores: Vec<Score>,
}

#[derive(Template)]
#[template(path = "score.html")]
struct SpecificScoreTemplate {
    score: Score,
}

#[derive(Deserialize)]
struct CreateScoreRequest {
    username: Option<String>,
    score: i64,
}

#[derive(Deserialize)]
struct GetScoresRequest {
    since: Option<i64>,
    username: Option<String>,
    pending: Option<bool>,
}

async fn get_score_handler(db: Arc<Db>, id: i64) -> Response {
    match db.get_score_by_id(id).await {
        Ok(score) => reply_status(json(&score), StatusCode::OK),
        Err(SQLError::RowNotFound) => reply_status("Score doesn't exist", StatusCode::NOT_FOUND),
        Err(_) => reply_status(
            "Could not process request",
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

async fn create_score_handler(db: Arc<Db>, score: CreateScoreRequest) -> Response {
    if let Some(username) = score.username {
        match db.insert_score(&username, score.score).await {
            Ok(id) => reply_status(id.to_string(), StatusCode::CREATED),
            Err(_) => reply_status("Could not create score", StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        match db.insert_pending_score(score.score).await {
            Ok(id) => reply_status(id.to_string(), StatusCode::CREATED),
            Err(_) => reply_status(
                "Could not create pending score",
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        }
    }
}

async fn delete_score_handler(db: Arc<Db>, id: i64) -> Response {
    match db.delete_score(id).await {
        Ok(true) => reply_status("Deleted score", StatusCode::OK),
        Ok(false) => reply_status("Score not found", StatusCode::NOT_FOUND),
        Err(_) => reply_status("Did not delete score", StatusCode::NOT_MODIFIED),
    }
}

async fn get_scores_handler(db: Arc<Db>, options: GetScoresRequest) -> Response {
    let options = FilterOptions {
        username: options.username,
        since: options.since.map(|timestamp| {
            let naive = NaiveDateTime::from_timestamp(timestamp, 0);
            DateTime::<Utc>::from_utc(naive, Utc)
        }),
        pending: options.pending,
    };

    match db.get_scores(options).await {
        Ok(scores) => reply_status(json(&scores), StatusCode::OK),
        Err(_) => reply_status("Failed to get scores", StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn render_template<T: Template>(template: T) -> Response {
    match template.render() {
        Ok(render) => reply_status(html(render), StatusCode::OK),
        Err(_) => {
            let template_name = std::any::type_name::<T>();
            error!(?template_name, "Failed to render template");
            reply_status(
                "Could not render template",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    }
}

fn reply_status(reply: impl warp::Reply, status: StatusCode) -> Response {
    Box::new(warp::reply::with_status(reply, status)).into_response()
}

pub async fn serve(db: Db) -> EResult<()> {
    let db = Arc::new(db);
    let db_hook = warp::any().map(move || Arc::clone(&db));

    let get_score = warp::get()
        .and(db_hook.clone())
        .and(warp::path::param::<i64>())
        .then(get_score_handler);

    let create_score = warp::post()
        .and(db_hook.clone())
        .and(warp::body::json::<CreateScoreRequest>())
        .then(create_score_handler);

    let delete_score = warp::delete()
        .and(db_hook.clone())
        .and(warp::path::param::<i64>())
        .then(delete_score_handler);

    let score = warp::path("score").and(get_score.or(create_score).or(delete_score));

    let get_scores = warp::get()
        .and(db_hook.clone())
        .and(warp::query::<GetScoresRequest>())
        .then(get_scores_handler);

    let scores = warp::path("scores").and(get_scores);

    let leaderboard = warp::get()
        .and(warp::path::end())
        .and(db_hook.clone())
        .then(|db: Arc<Db>| async move {
            match db.get_scores(FilterOptions::default()).await {
                Ok(scores) => scores,
                Err(_) => Vec::new(),
            }
        })
        .map(|scores: Vec<Score>| LeaderboardTemplate { scores })
        .then(render_template);

    let specific_score = warp::path("score")
        .and(warp::get())
        .and(db_hook.clone())
        .and(warp::path::param::<i64>())
        .and_then(|db: Arc<Db>, id: i64| async move {
            db.get_score_by_id(id).await.map_err(|_| warp::reject())
        })
        .map(|score| SpecificScoreTemplate { score })
        .then(render_template);

    let resources = warp::path("assets").and(warp::fs::dir("resources"));

    let api = warp::path("api").and(scores.or(score));
    let web = leaderboard.or(specific_score);
    let routes = resources.or(web.or(api));
    let port: u16 = env_var("MEISHU_PORT")
        .unwrap_or("3030".to_string())
        .parse()
        .wrap_err("failed to parse port environment variable as number")?;

    info!(?port, "starting webserver");
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;

    Ok(())
}
