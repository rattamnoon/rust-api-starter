use std::io;

use rust_api_starter::{config::settings::Settings, logging};

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();

    let settings = Settings::from_env().map_err(io::Error::other)?;
    logging::init(&settings).map_err(io::Error::other)?;

    tracing::info!(
        namespace = %settings.temporal_namespace,
        task_queue = %settings.temporal_task_queue,
        temporal_server_url = %settings.temporal_server_url,
        "temporal worker bootstrap placeholder is configured; commerce orchestration is started by payment webhooks"
    );

    Ok(())
}
