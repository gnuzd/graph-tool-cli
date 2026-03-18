use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

#[derive(Clone, Serialize, Debug)]
pub struct Node {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Serialize, Debug)]
pub struct Edge {
    pub source: String,
    pub target: String,
}

#[derive(Clone, Serialize, Debug, Default)]
pub struct GraphData {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Clone, Serialize, Debug)]
pub struct GraphUpdate {
    pub kind: String,
    pub payload: GraphData,
}

pub struct AppState {
    pub tx: broadcast::Sender<GraphUpdate>,
    pub current_graph: Arc<Mutex<GraphData>>,
}

pub async fn start_server(state: Arc<AppState>, ui_path: PathBuf) {
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .nest_service("/", ServeDir::new(ui_path))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    println!("Graph visualizer ready at http://localhost:8000");
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    // 1. Send current state immediately on connection
    let initial_data = {
        let graph = state.current_graph.lock().unwrap();
        GraphUpdate {
            kind: "graph".to_string(),
            payload: graph.clone(),
        }
    };

    let msg = serde_json::to_string(&initial_data).unwrap();
    if socket.send(Message::Text(msg.into())).await.is_ok() {
        // 2. Then listen for future updates
        let mut rx = state.tx.subscribe();
        while let Ok(update) = rx.recv().await {
            let msg = serde_json::to_string(&update).unwrap();
            if socket.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    }
}
