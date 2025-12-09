use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::{Html, Response},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;

pub struct DevServer {
    port: u16,
}

impl DevServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }
    
    pub async fn start(&self) -> anyhow::Result<()> {
        let app = Router::new()
            .route("/", get(index_handler))
            .route("/ws", get(ws_handler))
            .nest_service("/dist", ServeDir::new("dist"))
            .layer(CorsLayer::permissive());
        
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        println!("🚀 RUX dev server running on http://{}", addr);
        
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        
        Ok(())
    }
}

async fn index_handler() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>RUX Dev Server</title>
    <meta charset="utf-8">
</head>
<body>
    <div id="root"></div>
    <script type="module">
        // WASM loading would go here
        console.log('RUX dev server loaded');
    </script>
</body>
</html>
    "#)
}

async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_websocket)
}

async fn handle_websocket(mut socket: WebSocket) {
    // WebSocket handler for hot reload
    // Would send updates when files change
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            // Handle WebSocket messages
            let _ = msg;
        }
    }
}
