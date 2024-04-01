use std::sync::Arc;

use super::constants::{
    HEADER_GAP_SIZE, HEADER_GRID_LENGTH, HEADER_ID, HEADER_IMAGE_SIZE, HEADER_TOPIC,
};
use crate::service::ImHumane;
use axum::{
    extract::{Json, Path},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Extension, Form, Router,
};

#[derive(Debug, serde::Deserialize)]
pub struct ChallengePostPayload {
    challenge_id: uuid::Uuid,
    answer: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct TokenPostPayload {
    challenge_id: uuid::Uuid,
}

pub async fn challenge_post(
    Extension(imhumane): Extension<Arc<ImHumane>>,
    Json(payload): Json<ChallengePostPayload>,
) -> impl IntoResponse {
    let challenge_id_str = payload.challenge_id.to_string();
    let answer = payload.answer;
    let result = imhumane.check_answer(challenge_id_str.clone(), answer.clone());

    tracing::info!(
        challenge_id = challenge_id_str,
        provided_answer = answer,
        correct = result,
        "Validating challenge"
    );

    (
        (match result {
            true => StatusCode::NO_CONTENT,
            false => StatusCode::UNAUTHORIZED,
        }),
        [
            ("Access-Control-Allow-Origin", "*"),
            ("Access-Control-Allow-Headers", "*"),
            ("Access-Control-Expose-Headers", "*"),
            ("Access-Control-Allow-Method", "*"),
        ],
    )
}

pub async fn challenge_get(Extension(imhumane): Extension<Arc<ImHumane>>) -> impl IntoResponse {
    let challenge = imhumane.get_challenge().await;

    tracing::info!(
        challenge_id = challenge.id,
        answer = challenge.answer,
        "Sending challenge"
    );

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

pub async fn challenge_token_post_json(
    Extension(imhumane): Extension<Arc<ImHumane>>,
    Json(payload): Json<TokenPostPayload>,
) -> impl IntoResponse {
    let challenge_id_str = payload.challenge_id.to_string();
    let result = imhumane.check_token(challenge_id_str.clone());

    tracing::info!(
        challenge_id = challenge_id_str,
        valid = result,
        method = "POST",
        content_type = "application/json",
        "Validating token"
    );

    match result {
        true => StatusCode::NO_CONTENT,
        false => StatusCode::UNAUTHORIZED,
    }
}

pub async fn challenge_token_post_form(
    Extension(imhumane): Extension<Arc<ImHumane>>,
    Form(payload): Form<TokenPostPayload>,
) -> impl IntoResponse {
    let challenge_id_str = payload.challenge_id.to_string();
    let result = imhumane.check_token(challenge_id_str.clone());

    tracing::info!(
        challenge_id = challenge_id_str,
        valid = result,
        method = "POST",
        content_type = "application/x-www-form-urlencoded",
        "Validating token"
    );

    match result {
        true => StatusCode::NO_CONTENT,
        false => StatusCode::UNAUTHORIZED,
    }
}

pub async fn challenge_token_get(
    Extension(imhumane): Extension<Arc<ImHumane>>,
    Path(challenge_id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    let challenge_id_str = challenge_id.to_string();
    let result = imhumane.check_token(challenge_id_str.clone());

    tracing::info!(
        challenge_id = challenge_id_str,
        valid = result,
        method = "GET",
        "Validating token"
    );

    match result {
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
            "/v1/tokens/validate/json",
            get(challenge_token_post_json).options(cors),
        )
        .route(
            "/v1/tokens/validate/form",
            get(challenge_token_post_form).options(cors),
        )
        .route(
            "/v1/tokens/:challenge_id",
            get(challenge_token_get).options(cors),
        )
        .layer(Extension(service))
}
