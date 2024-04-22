use anyhow::Result;
use axum::Router;
use axum::routing::get;
use tokio::signal;

pub struct Server {
    address: String,
}

impl Server {
    pub fn new(address: &str) -> Self {
        Self { address: address.to_string() }
    }
    
    pub async fn serve(&self) -> Result<()> {
        let app = Router::new().route("/", get(|| async { "Hello, World!" }));
        let listener = tokio::net::TcpListener::bind(&self.address).await?;
        axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await.unwrap();
        Ok(())
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
        let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
