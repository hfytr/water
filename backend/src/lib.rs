use anyhow::Result;
use axum::{
    body::Bytes,
    extract::{
        multipart::{Field, Multipart},
        State,
    },
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Form, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::env::{var, VarError};
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
struct SimpleError {
    details: String,
}

impl SimpleError {
    fn new(details: String) -> SimpleError {
        Self { details }
    }
}

impl Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("error details: {}", self.details))
    }
}

impl Error for SimpleError {}

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

impl AppState {
    async fn new() -> Result<AppState> {
        Ok(Self {
            db: PgPool::connect(&format!(
                "postgres://{}:{}@localhost:{}/{}",
                var("DB_USER").expect("env var doesn't exist: DB_USER"),
                var("DB_PASSWORD").expect("env var doesn't exist: DB_PASSWORD"),
                var("DB_PORT").expect("env var doesn't exist: DB_PORT"),
                var("DB_NAME").expect("env var doesn't exist: DB_NAME"),
            ))
            .await?,
        })
    }
}

pub async fn app() -> Router {
    let state = AppState::new().await;
    Router::new()
        .route("/", get(health_check))
        .route("/post/dumb", post(dumb_post))
        .route("/post/smart", post(smart_post))
        .with_state(state.expect("failed to connect to database"))
}

async fn health_check() -> impl IntoResponse {
    dbg!("recieved");
    StatusCode::OK
}

#[derive(Deserialize)]
struct DumbMetadata {
    userid: usize,
}

async fn dumb_post(
    State(AppState { db }): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut file_data: Option<Bytes> = None;
    let mut userid: Result<usize> =
        Result::Err(SimpleError::new("userid not specified in metadata".to_string()).into());
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap();
        match name {
            "file" => {
                file_data = field.bytes().await.ok();
            }
            "json" => {
                userid = parse_json(field).await;
            }
            _ => {}
        }
    }

    if file_data.is_none() {
        return Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body("did not recieve a file in request".to_string())
            .unwrap();
    }

    if userid.is_err() {
        return Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(format!("{}", userid.unwrap_err()))
            .unwrap();
    }

    Response::builder()
        .status(StatusCode::OK)
        .body(String::new())
        .unwrap()
}

async fn parse_json(field: Field<'_>) -> Result<usize> {
    Ok(serde_json::from_str::<DumbMetadata>(&field.text().await?).map(|x| x.userid)?)
}

#[derive(Deserialize)]
pub struct SmartPostData {
    userid: usize,
    water: usize,
}

async fn smart_post(Form(form): Form<SmartPostData>) -> impl IntoResponse {
    format!("userid: {}, water: {}", form.userid, form.water)
}
