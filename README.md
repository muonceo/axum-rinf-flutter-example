# axum-rinf-flutter-example

```sh
$ mkdir axum-rinf-flutter-example
$ cd axum-rinf-flutter-example
$ git init -b main
$ code .
```

Cargo.toml
```toml
[workspace]
resolver = "2"
members = []
```

.gitignore
```
.vscode/
target/
```

README.md
```md
# axum-rinf-flutter-example
```

```
$ cargo new api-server
$ cargo new --lib models
```

Cargo.toml
```toml
[workspace]
resolver = "2"
members = ["api-server", "models"]
```

require cargo-edit

```sh
$ cd models
$ cargo add serde --features=derive
```

models/Cargo.toml
```toml
[package]
name = "models"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0.197", features = ["derive"] }
```

models/src/lib.rs
```rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Counter {
    pub number: i32,
}

impl Counter {
    pub fn new() -> Self {
        Self { number: 0 }
    }

    pub fn increment(&mut self) {
        self.number += 1;
    }

    pub fn get(&self) -> i32 {
        self.number
    }

    pub fn set(&mut self, number: i32) {
        self.number = number;
    }
}

#[cfg(test)]
mod tests {
    use crate::Counter;

    #[test]
    fn test_counter() {
        let mut counter = Counter::new();
        assert_eq!(counter.get(), 0);
        counter.increment();
        assert_eq!(counter.get(), 1);
        counter.increment();
        assert_eq!(counter.get(), 2);
        counter.set(0);
        assert_eq!(counter.get(), 0);
    }
}
```


```sh
$ cd models
$ cargo test
```

```sh
$ cd api-server
$ cargo add models --path=../models
$ cargo add axum tracing tracing-subscriber thiserror
$ cargo add tokio --features=full
$ cargo add tower-http --features=cors
$ cargo add serde --features=serde
$ cargo add --dev axum-test
```

axum-rinf-flutter-example/api-server/Cargo.toml
```toml
[package]
name = "api-server"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7.5"
axum-test = "14.8.0"
models = { version = "0.1.0", path = "../models" }
serde = { version = "1.0.197", features = ["derive"] }
thiserror = "1.0.58"
tokio = { version = "1.37.0", features = ["full"] }
tower-http = { version = "0.5.2", features = ["cors"] }
tracing = "0.1.40"
tracing-subscriber = "0.3.18"

[dev-dependencies]
axum-test = "14.8.0"
```

api-server/src/main.rs
```rs
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

```

```sh
$ cargo test
```

```sh
$ cargo install xh
...

$ xh -b localhost:3000
Hello, world!

$ xh -b localhost:3000/counter
{
    "number": 0
}

$ xh -b localhost:3000/counter
{
    "number": 1
}
```

flutter + rinf

```sh
$ flutter create app
$ cd app
$ flutter pub add rinf
$ cargo install rinf
$ rinf template
```

```app/Cargo.toml
# This file is used for telling Rust-related tools
# where various Rust crates are.
# This also unifies `./target` output folder and
# various Rust configurations.

[workspace]
members = ["./native/*"]
resolver = "2"
```

```sh
$ rm app/Cargo.toml
```

```Cargo.toml
[workspace]
resolver = "2"
members = ["api-server", "models", "app/native/*"]
```

```sh
$ cd app/messages
$ rm -rf counter_number.proto fractal_art.proto sample_folder/
```

```proto
syntax = "proto3";
package counter;

// [RINF:DART-SIGNAL]
message SetCounter {
    int32 counter = 1;
}

// [RINF:RUST-SIGNAL]
message Counter {
    int32 number = 1;
}
```

```sh
$ cd app
$ rinf message
```

```sh
$ cd app/native
$ rm -rf sample_crate
$ cd hub
$ rm sample_functions.rs
```

sample-create 삭제
app/native/hub/Cargo.toml
```toml
[package]
# Do not change the name of this crate.
name = "hub"
version = "0.1.0"
edition = "2021"

[lib]
# `lib` is required for non-library targets,
# such as tests and benchmarks.
# `cdylib` is for Linux, Android, Windows, and web.
# `staticlib` is for iOS and macOS.
crate-type = ["lib", "cdylib", "staticlib"]

[dependencies]
rinf = "6.7.0"
prost = "0.12.3"
wasm-bindgen = "0.2.91"
tokio_with_wasm = "0.4.3"
```

app/native/hub/src/lib.rs
```rs
//! This `hub` crate is the
//! entry point of the Rust logic.

// This `tokio` will be used by Rinf.
// You can replace it with the original `tokio`
// if you're not targeting the web.
use tokio_with_wasm::tokio;

mod messages;

rinf::write_interface!();

// Always use non-blocking async functions
// such as `tokio::fs::File::open`.
// If you really need to use blocking code,
// use `tokio::task::spawn_blocking`.
async fn main() {
    // Repeat `tokio::spawn` anywhere in your code
    // if more concurrent tasks are needed.
}
```

```sh
$ cd app/native
$ cargo new --lib api-client
$ cd api-client
$ cargo add models --path=../../../models
$ cargo add reqwest --no-default-features --features=json
$ cargo add thiserror
```

app/native/api-client/src/lib.rs
```rs
use models::Counter;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

// When running on mobile, you must specify the IP address.
// static HOST: &str = "http://<api-server_ip_addr>:3000";
static HOST: &str = "http://localhost:3000";

pub async fn get_counter() -> Result<Counter> {
    // `reqwest` supports all platforms, including web.
    let counter = reqwest::get(format!("{HOST}/counter"))
        .await?
        .json::<Counter>()
        .await?;
    Ok(counter)
}

pub async fn set_counter(counter: &Counter) -> Result<bool> {
    let response = reqwest::Client::new()
        .put(format!("{HOST}/counter"))
        .json(&counter)
        .send()
        .await?;
    Ok(response.status().is_success())
}
```

```sh
$ cd app/native/hub
$ cargo add models --path=../../../models
$ cargo add api-client --path=../api-client
```

app/native/hub/lib/counter.rs
```rs
use crate::messages;
use crate::tokio;
use messages::counter::*;
use rinf::debug_print;

pub async fn set_counter() {
    let mut receiver = SetCounter::get_dart_signal_receiver();
    while let Some(dart_signal) = receiver.recv().await {
        let set_counter = dart_signal.message;

        let mut counter = models::Counter::new();
        counter.set(set_counter.counter);

        if let Err(e) = api_client::set_counter(&counter).await {
            debug_print!("api_client::set_counter() error: {e}");
        }
    }
}

pub async fn counter() {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        match api_client::get_counter().await {
            Ok(counter) => {
                Counter {
                    number: counter.get(),
                }
                .send_signal_to_dart(None);
            }
            Err(e) => {
                debug_print!("api_client::get_counter() error: {e}");
            }
        }
    }
}
```

app/native/hub/src/lib.rs
```rs
//! This `hub` crate is the
//! entry point of the Rust logic.

// This `tokio` will be used by Rinf.
// You can replace it with the original `tokio`
// if you're not targeting the web.
use tokio_with_wasm::tokio;

mod counter;
mod messages;

rinf::write_interface!();

// Always use non-blocking async functions
// such as `tokio::fs::File::open`.
// If you really need to use blocking code,
// use `tokio::task::spawn_blocking`.
async fn main() {
    // Repeat `tokio::spawn` anywhere in your code
    // if more concurrent tasks are needed.
    tokio::spawn(counter::set_counter());
    tokio::spawn(counter::counter());
}
```

app/android/app/src/main/AndroidManifest.xml
```xml
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <uses-permission android:name="android.permission.INTERNET" />
    ...
</manifest>
```

macOS *.entitlements
```xml
<key>com.apple.security.network.client</key>
<true/>
```

app/lib/main.dart
```dart
import 'package:app/messages/counter.pb.dart';
import 'package:flutter/material.dart';
import './messages/generated.dart';

void main() async {
  await initializeRust();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'axum-rinf-flutter-example',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const MyHomePage(title: 'axum-rinf-flutter-example'),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key, required this.title});

  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: Text(widget.title),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            StreamBuilder(
                stream: Counter.rustSignalStream,
                builder: (context, snapshot) {
                  final rustSignal = snapshot.data;
                  if (rustSignal == null) {
                    return const Text("Nothing received yet");
                  }
                  final counter = rustSignal.message;
                  final currentNumebr = counter.number;
                  return Text(
                    currentNumebr.toString(),
                    style: const TextStyle(fontSize: 40),
                  );
                }),
          ],
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () {
          SetCounter(counter: 0).sendSignalToRust(null);
        },
        tooltip: 'Reset counter number',
        child: const Icon(Icons.restart_alt),
      ),
    );
  }
}
```

api-server start
```sh
$ cd api-server
$ cargo run
```

flutter run
```sh
$ cd app
$ flutter run
```

flutter web build
```sh
$ rinf wasm --release
$ flutter build web
```
