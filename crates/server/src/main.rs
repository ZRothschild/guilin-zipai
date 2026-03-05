use guilin_paizi_server::GameServer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "guilin_paizi_server=info,warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let server = GameServer::new();
    server.run("127.0.0.1:8080").await?;

    Ok(())
}
