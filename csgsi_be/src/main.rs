mod error;

use std::{net::SocketAddr, sync::Arc};

use anyhow::anyhow;
use axum::{
    extract::{ws::WebSocket, BodyStream, ConnectInfo, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use futures::StreamExt;
use tokio::sync::broadcast;
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use error::AppError;

type Broadcaster = broadcast::Sender<Vec<u8>>;

async fn gsi_handler(
    State(broadcaster): State<Arc<Broadcaster>>,
    mut b: BodyStream,
) -> Result<(), AppError> {
    let mut body: Vec<u8> = vec![];
    while let Some(maybe_bytes) = b.next().await {
        body.extend(maybe_bytes.map_err(|e| anyhow!(e))?);
    }
    if !body.is_empty() {
        if let Err(e) = broadcaster.send(body) {
            error!("failed to broadcast: {:?}", e);
        }
    }

    Ok(())
}

async fn ws_handler(
    State(broadcaster): State<Arc<Broadcaster>>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    info!("{addr} connected.");
    let rx = broadcaster.subscribe();
    ws.on_upgrade(move |socket| handle_socket(rx, socket, addr))
}

async fn handle_socket(
    mut rx: broadcast::Receiver<Vec<u8>>,
    mut socket: WebSocket,
    who: SocketAddr,
) {
    loop {
        match rx.recv().await {
            Ok(b) => {
                let msg = String::from_utf8(b)
                    .expect("cs2 did not send valid UTF-8")
                    .into();
                if let Err(e) = socket.send(msg).await {
                    error!("error sending to socket {}: {:?}", who, e);
                    break;
                }
            }
            Err(e) => {
                error!("error receiving broadcast: {:?}", e);
                break;
            }
        }
    }
    info!("{who} disconnected");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "".into()), //.unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (broadcaster, _rx) = broadcast::channel::<Vec<u8>>(2);

    let assets_dir = "./assets";
    let app = Router::new()
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/gsi", post(gsi_handler))
        .route("/ws", get(ws_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(Arc::new(broadcaster));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
