use anyhow::{anyhow, Result};
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
use serde::Deserialize;
use sqlx::{postgres::PgPool, query, Row};
use std::env::var;
use std::iter::zip;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

impl AppState {
    async fn new() -> Result<AppState> {
        Ok(Self {
            pool: PgPool::connect(
                &var("DATABASE_URL").expect("env var doesn't exist: DATABASE_URL"),
            )
            .await?,
        })
    }
}

pub async fn app() -> Router {
    let state = AppState::new().await;
    Router::new()
        .route("/", get(health_check))
        .route("/update/dumb", post(dumb_update))
        .route("/update/smart", post(smart_update))
        .route("/new-user", post(new_user))
        .with_state(state.expect("failed to connect to database"))
}

async fn health_check() -> impl IntoResponse {
    dbg!("recieved");
    StatusCode::OK
}

#[derive(Deserialize)]
struct NewUserData {
    // sha-256, swift frontend only has access to u64
    pass_hash: Vec<u8>,
    source_names: Vec<String>,
    source_rate: Vec<f32>,
}

async fn new_user(
    State(AppState { pool: db }): State<AppState>,
    Form(form): Form<NewUserData>,
) -> impl IntoResponse {
    let wrapped_id = query("select max(user_id) from user_table")
        .fetch_optional(&db)
        .await;

    if let Err(e) = wrapped_id {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("failed to query db: {}", e))
            .unwrap();
    }

    let new_user_id = wrapped_id.unwrap().map(|x| x.get("user_id")).unwrap_or(-1) + 1;

    if let Err(e) = query!(
        "insert into user_table (user_id, water_hourly, pass_hash, source_names, source_rates)
        values ($1, $2, $3, $4, $5)",
        new_user_id,
        &Vec::new(),
        form.pass_hash,
        &form.source_names,
        &form.source_rate
    )
    .execute(&db)
    .await
    {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("failed to write data to db: {}", e))
            .unwrap();
    }

    return Response::builder()
        .status(StatusCode::OK)
        .body("all good!".to_string())
        .unwrap();
}

#[derive(Deserialize)]
struct DumbMetadata {
    userid: usize,
    pass_hash: Vec<u8>,
}

async fn dumb_update(
    State(AppState { pool: db }): State<AppState>,
    mut multipart: Multipart,
) -> Result<StatusCode> {
    let mut maybe_file_data: Result<Bytes> = Result::Err(anyhow!("file data not sent in request"));
    let mut maybe_user_id: Result<usize> = Result::Err(anyhow!("user id not sent in request"));

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap();
        match name {
            "file" => {
                maybe_file_data = field.bytes().await.map_err(anyhow::Error::from);
            }
            "json" => {
                maybe_user_id = parse_json(field).await;
            }
            _ => {}
        }
    }

    let file_data = maybe_file_data?;
    let user_id = maybe_user_id?;

    Ok(StatusCode::OK)
}

async fn parse_json(field: Field<'_>) -> Result<usize> {
    Ok(serde_json::from_str::<DumbMetadata>(&field.text().await?).map(|x| x.userid)?)
}

#[derive(Deserialize)]
pub struct SmartUpdateData {
    userid: u32,
    water: f32,
    pass_hash: Vec<u8>,
}

async fn smart_update(
    State(AppState { pool: db }): State<AppState>,
    Form(form): Form<SmartUpdateData>,
) -> Result<Response<String>> {
    let user_pass_hash = query!(
        "select pass_hash from user_table where user_id = $1",
        form.userid as i64
    )
    .fetch_one(&db)
    .await?;

    if zip(
        user_pass_hash.pass_hash.unwrap().iter(),
        form.pass_hash.iter(),
    )
    .any(|(true_byte, provided_byte)| true_byte != provided_byte)
    {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("provided pass hash does not match user id".to_string())
            .unwrap());
    }

    query!(
        "update user_table set water_hourly = array_append(water_hourly, $1)",
        form.water
    )
    .execute(&db)
    .await?;

    Ok(Response::new(format!(
        "userid: {}, water: {}",
        form.userid, form.water
    )))
}
