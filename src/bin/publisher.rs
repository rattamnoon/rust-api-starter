use std::io;

use rust_api_starter::{
    config::settings::Settings, db::pool::create_pool, logging,
    modules::events::repository::EventRepository, shared::kafka::KafkaClient,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();

    let settings = Settings::from_env().map_err(io::Error::other)?;
    logging::init(&settings).map_err(io::Error::other)?;

    let pool = create_pool(&settings.database_url)
        .await
        .map_err(io::Error::other)?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(io::Error::other)?;

    let repository = EventRepository::new(pool);
    let kafka = KafkaClient::new(&settings).map_err(io::Error::other)?;

    tracing::info!("starting kafka outbox publisher");

    loop {
        let events = repository
            .list_pending(100)
            .await
            .map_err(io::Error::other)?;
        if events.is_empty() {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            continue;
        }

        for event in events {
            let key = event.aggregate_id.to_string();
            match kafka.publish_json(&event.topic, &key, &event.payload).await {
                Ok(()) => {
                    repository
                        .mark_published(event.id)
                        .await
                        .map_err(io::Error::other)?;
                    tracing::info!(
                        event_id = %event.id,
                        topic = %event.topic,
                        event_type = %event.event_type,
                        "published domain event to kafka"
                    );
                }
                Err(error) => {
                    repository
                        .mark_failed(event.id, &error.to_string())
                        .await
                        .map_err(io::Error::other)?;
                    tracing::error!(
                        event_id = %event.id,
                        topic = %event.topic,
                        event_type = %event.event_type,
                        "failed to publish domain event: {error}"
                    );
                }
            }
        }
    }
}
