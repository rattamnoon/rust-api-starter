use std::io;

use futures_util::StreamExt;
use lapin::{message::Delivery, options::BasicAckOptions};
use rust_api_starter::{
    config::settings::Settings,
    db::pool::create_pool,
    logging,
    modules::{
        events::{repository::EventRepository, service::EventService},
        jobs::{repository::JobRepository, service::WorkerJobService},
        orders::repository::OrderRepository,
        receipts::{repository::ReceiptRepository, service::ReceiptService},
        uploads::repository::UploadRepository,
        users::repository::UserRepository,
    },
    shared::queue::{QueueJobMessage, RabbitMqClient},
    shared::{email::EmailService, file_storage::LocalFileStorage},
};

#[actix_web::main]
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

    let queue = RabbitMqClient::connect(
        &settings.rabbitmq_url,
        &settings.rabbitmq_queue_name,
        &settings.rabbitmq_dead_letter_queue,
    )
    .await
    .map_err(io::Error::other)?;
    let receipt_service = ReceiptService::new(
        settings.clone(),
        ReceiptRepository::new(pool.clone()),
        OrderRepository::new(pool.clone()),
        UploadRepository::new(pool.clone()),
        UserRepository::new(pool.clone()),
        EmailService::new(&settings),
        EventService::new(&settings, EventRepository::new(pool.clone())),
        LocalFileStorage::new(settings.upload_dir.clone().into()),
    );
    let service = WorkerJobService::new(
        JobRepository::new(pool),
        queue.clone(),
        receipt_service,
        settings.job_max_retries,
    );

    tracing::info!(
        "starting worker with concurrency {}",
        settings.worker_concurrency
    );
    let mut consumer = queue
        .consumer_with_channel("rust-api-starter-worker", settings.worker_concurrency)
        .await
        .map_err(io::Error::other)?;

    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                let service = service.clone();
                tokio::spawn(async move {
                    if let Err(error) = handle_delivery(&service, delivery).await {
                        tracing::error!("worker delivery failed: {error}");
                    }
                });
            }
            Err(error) => {
                tracing::error!("worker consumer stream error: {error}");
            }
        }
    }

    Ok(())
}

async fn handle_delivery(service: &WorkerJobService, delivery: Delivery) -> Result<(), io::Error> {
    let message: QueueJobMessage =
        serde_json::from_slice(&delivery.data).map_err(io::Error::other)?;
    match service.process_message(message).await {
        Ok(()) => {
            delivery
                .ack(BasicAckOptions::default())
                .await
                .map_err(io::Error::other)?;
        }
        Err(error) => {
            tracing::error!("job processing failed: {error}");
            delivery
                .ack(BasicAckOptions::default())
                .await
                .map_err(io::Error::other)?;
        }
    }
    Ok(())
}
