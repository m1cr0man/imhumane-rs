use std::sync::Arc;

use super::{
    constants::{HEADER_ID, HEADER_TOPIC},
    error::{Error, ParseUuidSnafu},
};
use crate::imhumane::ImHumane;
use axum::{
    extract::{Json, Path},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use snafu::prelude::*;

#[derive(serde::Deserialize)]
pub struct PostPayload {
    challenge_id: String,
    answer: u32,
}

pub async fn challenge_post(
    Extension(imhumane): Extension<Arc<ImHumane>>,
    Json(payload): Json<PostPayload>,
) -> Result<impl IntoResponse, Error> {
    let challenge_id = uuid::Uuid::try_parse(&payload.challenge_id).context(ParseUuidSnafu)?;

    let answer = payload.answer;

    match imhumane.check_answer(challenge_id.to_string(), answer) {
        true => Ok(StatusCode::NO_CONTENT),
        false => Ok(StatusCode::UNAUTHORIZED),
    }
}

pub async fn challenge_get(imhumane: Extension<Arc<ImHumane>>) -> impl IntoResponse {
    let challenge = imhumane.get_challenge().await;
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE.as_str(), "image/jpeg".to_string()),
            (HEADER_ID, challenge.id),
            (HEADER_TOPIC, challenge.topic),
        ],
        challenge.image,
    )
}

pub async fn challenge_token_get(
    imhumane: Extension<Arc<ImHumane>>,
    Path(challenge_id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    match imhumane.check_token(challenge_id.to_string()) {
        true => StatusCode::NO_CONTENT,
        false => StatusCode::UNAUTHORIZED,
    }
}

pub fn get_router(service: Arc<ImHumane>) -> Router {
    Router::new()
        .layer(Extension(service))
        .route("/", get(challenge_get).post(challenge_post))
        .route("/:challenge_id", get(challenge_token_get))
}
