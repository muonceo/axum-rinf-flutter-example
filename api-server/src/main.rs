use std::sync::Arc;

use axum::{
    extract::State,
    http::{self, header::InvalidHeaderName, HeaderName, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, put},
    Json, Router,
};
use models::Counter;
use serde::Serialize;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct AppState {
    counter: Arc<Mutex<Counter>>, // Wrap it with Arc and Mutex to share between threads.
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid header name: {0}")]
    InvalidHeaderName(#[from] InvalidHeaderName),
    #[error("axum error: {0}")]
    AxumError(#[from] axum::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let error = self.to_string();
        let status_code = match self {
            Error::InvalidHeaderName(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::AxumError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status_code, Json(ErrorResponseBody { error })).into_response()
    }
}

#[derive(Serialize, Debug)]
struct ErrorResponseBody {
    error: String,
}

type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger.
    tracing_subscriber::fmt::init();

    // Start the server.
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 3000))
        .await
        .unwrap();
    axum::serve(listener, create_app()?).await.unwrap();

    Ok(())
}

// Create an app by defining routes.
fn create_app() -> Result<Router> {
    let app_state = AppState {
        counter: Arc::new(Mutex::new(Counter { number: 0 })),
    };

    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/counter", get(get_counter))
        .route("/counter", put(set_counter))
        .with_state(app_state)
        .layer(cors_layer()?);

    Ok(app)
}

// This is necessary to use it on a web built with Flutter.
fn cors_layer() -> Result<CorsLayer> {
    Ok(CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(vec![
            http::header::AUTHORIZATION,
            http::header::CONTENT_TYPE,
            HeaderName::try_from("x-response-content-type")?,
        ]))
}

// Increments the counter every time it runs.
async fn get_counter(State(app_state): State<AppState>) -> Result<Json<Counter>> {
    let mut counter = app_state.counter.lock().await;
    let json = Json(counter.clone());
    counter.increment();
    Ok(json)
}

// Set counter number.
async fn set_counter(
    State(app_state): State<AppState>,
    Json(new_counter): Json<Counter>,
) -> Result<StatusCode> {
    let mut counter = app_state.counter.lock().await;
    counter.set(new_counter.get());
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use models::Counter;

    use crate::{create_app, Result};

    #[tokio::test]
    async fn test_hello_world() -> Result<()> {
        let server = TestServer::new(create_app()?).unwrap();
        let response = server.get("/").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(response.text(), "Hello, world!");
        Ok(())
    }

    #[tokio::test]
    async fn test_counter() -> Result<()> {
        let server = TestServer::new(create_app()?).unwrap();

        let response = server.get("/counter").await;
        let counter: Counter = response.json();
        assert_eq!(counter.get(), 0);

        let response = server.get("/counter").await;
        let counter: Counter = response.json();
        assert_eq!(counter.get(), 1);

        let mut counter = Counter::new();
        counter.set(100);

        let response = server.put("/counter").json(&counter).await;
        assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

        let response = server.get("/counter").await;
        let counter: Counter = response.json();
        assert_eq!(counter.get(), 100);

        Ok(())
    }
}
