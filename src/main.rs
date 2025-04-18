use askama::Template;

use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use socketioxide::{
    SocketIo,
    extract::{Data, SocketRef},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub fn get_env(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

#[derive(Debug, Clone)]
struct AppState {
    content: Option<String>,
    expiry: Option<DateTime<Utc>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            content: None,
            expiry: None,
        }
    }
}

type Db = Arc<RwLock<AppState>>;

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    id: String,
    content: String,
}

#[tokio::main]
async fn main() {
    // dotenvy::dotenv().unwrap();
    let db = Db::default();

    let (layer, io) = SocketIo::builder().with_state(db.clone()).build_layer();

    io.ns("/", on_connect);

    let app = Router::new()
        .route("/", get(index))
        .with_state(db)
        .layer(layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

fn on_connect(socket: SocketRef) {
    socket.on(
        "TEXT_INPUT",
        |io: SocketRef,
         Data::<Message>(message),
         socketioxide::extract::State(db): socketioxide::extract::State<Db>| async move {
            let mut state = db.write().await;
            let expiry_duration = get_env("PASTE_EXPIRES_AFTER_SECONDS")
                .unwrap_or("900".to_string())
                .parse::<i64>()
                .unwrap_or(900);
            io.broadcast().emit("TEXT_BROADCAST", &message).await.ok();
            state.content = if !message.content.is_empty() {
                Some(message.content)
            } else {
                None
            };
            state.expiry = Some(Utc::now() + Duration::seconds(expiry_duration));
        },
    );
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexPage {
    uuid: String,
    content: String,
}

async fn index(State(db): State<Db>) -> impl IntoResponse {
    let mut state = db.write().await;
    if let Some(expiry) = state.expiry {
        if expiry <= Utc::now() {
            state.content = None;
        }
    }
    let template = IndexPage {
        content: state.content.clone().unwrap_or("".to_string()),
        uuid: Uuid::new_v4().to_string(),
    };
    HtmlTemplate(template)
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response<Body> {
        match self.0.render() {
            Ok(html) => (StatusCode::OK, Html(html)).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}
