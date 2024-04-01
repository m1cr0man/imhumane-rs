use std::sync::Arc;

use super::{
    constants::{HEADER_GAP_SIZE, HEADER_GRID_LENGTH, HEADER_ID, HEADER_IMAGE_SIZE, HEADER_TOPIC},
    error::{Error, ParseUuidSnafu},
};
use crate::service::ImHumane;
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
    answer: String,
}

pub async fn challenge_post(
    Extension(imhumane): Extension<Arc<ImHumane>>,
    Json(payload): Json<PostPayload>,
) -> Result<impl IntoResponse, Error> {
    let challenge_id = uuid::Uuid::try_parse(&payload.challenge_id).context(ParseUuidSnafu)?;

    let answer = payload.answer;

    Ok((
        (match imhumane.check_answer(challenge_id.to_string(), answer) {
            true => StatusCode::NO_CONTENT,
            false => StatusCode::UNAUTHORIZED,
        }),
        [
            ("Access-Control-Allow-Origin", "*"),
            ("Access-Control-Allow-Headers", "*"),
            ("Access-Control-Expose-Headers", "*"),
            ("Access-Control-Allow-Method", "*"),
        ],
    ))
}

pub async fn challenge_get(Extension(imhumane): Extension<Arc<ImHumane>>) -> impl IntoResponse {
    let challenge = imhumane.get_challenge().await;
    println!("Sending {} with answer {}", challenge.id, challenge.answer);
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE.as_str(), "image/webp".to_string()),
            (HEADER_ID, challenge.id),
            (HEADER_TOPIC, challenge.topic),
            (HEADER_GAP_SIZE, challenge.gap_size.to_string()),
            (HEADER_IMAGE_SIZE, challenge.image_size.to_string()),
            (HEADER_GRID_LENGTH, challenge.grid_length.to_string()),
            ("Access-Control-Allow-Origin", "*".to_string()),
            ("Access-Control-Allow-Headers", "*".to_string()),
            ("Access-Control-Expose-Headers", "*".to_string()),
            ("Access-Control-Allow-Method", "*".to_string()),
        ],
        challenge.image,
    )
}

pub async fn challenge_token_get(
    Extension(imhumane): Extension<Arc<ImHumane>>,
    Path(challenge_id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    match imhumane.check_token(challenge_id.to_string()) {
        true => StatusCode::NO_CONTENT,
        false => StatusCode::UNAUTHORIZED,
    }
}

pub async fn cors() -> impl IntoResponse {
    (
        StatusCode::NO_CONTENT,
        [
            ("Access-Control-Allow-Origin", "*"),
            ("Access-Control-Allow-Headers", "*"),
            ("Access-Control-Expose-Headers", "*"),
            ("Access-Control-Allow-Method", "*"),
        ],
    )
}

pub fn get_router(service: Arc<ImHumane>) -> Router {
    Router::new()
        .route(
            "/v1/challenge",
            get(challenge_get).post(challenge_post).options(cors),
        )
        .route(
            "/v1/tokens/:challenge_id",
            get(challenge_token_get).options(cors),
        )
        .layer(Extension(service))
}
