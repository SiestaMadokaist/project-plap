use axum::{routing::post, Router};
use backend::application::{
    ports::usecase::UsecaseAPI,
    routes::public::PublicRoute,
    usecases::hq::{
        health::Healthcheck,
        users::{challenge::GetChallenge, submit_answer::SubmitAnswer},
    },
};

use crate::http::app::{reply, AppState, Ctx, Payload, Reply};
use axum::extract::State;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(&PublicRoute::GetChallenge.to_string(), post(get_challenge))
        .route(&PublicRoute::SubmitAnswer.to_string(), post(submit_answer))
        .route(&PublicRoute::Health.to_string(), post(health).get(health))
}

async fn get_challenge(State(app): State<AppState>, Ctx(ctx): Ctx, Payload(p): Payload) -> Reply {
    let clients = app.clients.start(&ctx);
    reply(
        GetChallenge::new(&clients, app.repos.as_ref(), p.try_into()?)
            .exec()
            .await,
    )
}

async fn submit_answer(State(app): State<AppState>, Ctx(ctx): Ctx, Payload(p): Payload) -> Reply {
    let clients = app.clients.start(&ctx);
    reply(
        SubmitAnswer::new(&clients, app.repos.as_ref(), p.try_into()?)
            .exec()
            .await,
    )
}

async fn health() -> Reply {
    reply(Healthcheck::default().exec().await)
}
