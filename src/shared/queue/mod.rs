use chrono::{DateTime, Utc};
use lapin::{
    BasicProperties, Channel, Connection, ConnectionProperties, Consumer,
    options::{BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::app_error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueueJobType {
    SendWelcomeEmail,
    ProcessUploadedFile,
    RetryWebhook,
}

impl QueueJobType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SendWelcomeEmail => "send_welcome_email",
            Self::ProcessUploadedFile => "process_uploaded_file",
            Self::RetryWebhook => "retry_webhook",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueJobMessage {
    pub job_id: Uuid,
    pub job_type: QueueJobType,
    pub attempt: i32,
    pub created_at: DateTime<Utc>,
    pub trace_id: Option<String>,
}

#[derive(Clone)]
pub struct RabbitMqClient {
    channel: Channel,
    queue_name: String,
    dead_letter_queue: String,
}

impl RabbitMqClient {
    pub async fn connect(
        url: &str,
        queue_name: &str,
        dead_letter_queue: &str,
    ) -> Result<Self, AppError> {
        let connection = Connection::connect(url, ConnectionProperties::default()).await?;
        Self::from_connection(connection, queue_name, dead_letter_queue).await
    }

    async fn from_connection(
        connection: Connection,
        queue_name: &str,
        dead_letter_queue: &str,
    ) -> Result<Self, AppError> {
        let channel = connection.create_channel().await?;
        channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;
        channel
            .queue_declare(
                dead_letter_queue,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        Ok(Self {
            channel,
            queue_name: queue_name.to_string(),
            dead_letter_queue: dead_letter_queue.to_string(),
        })
    }

    pub async fn publish_job(&self, message: &QueueJobMessage) -> Result<(), AppError> {
        let payload =
            serde_json::to_vec(message).map_err(|error| AppError::Internal(error.to_string()))?;
        self.channel
            .basic_publish(
                "",
                &self.queue_name,
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default().with_delivery_mode(2),
            )
            .await?
            .await?;
        Ok(())
    }

    pub async fn publish_dead_letter(&self, message: &QueueJobMessage) -> Result<(), AppError> {
        let payload =
            serde_json::to_vec(message).map_err(|error| AppError::Internal(error.to_string()))?;
        self.channel
            .basic_publish(
                "",
                &self.dead_letter_queue,
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default().with_delivery_mode(2),
            )
            .await?
            .await?;
        Ok(())
    }

    pub async fn consumer(&self, consumer_tag: &str) -> Result<Consumer, AppError> {
        self.channel
            .basic_consume(
                &self.queue_name,
                consumer_tag,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(Into::into)
    }

    pub async fn consumer_with_channel(
        &self,
        consumer_tag: &str,
        prefetch_count: u16,
    ) -> Result<Consumer, AppError> {
        use lapin::options::BasicQosOptions;

        self.channel
            .basic_qos(prefetch_count, BasicQosOptions::default())
            .await?;
        self.consumer(consumer_tag).await
    }
}
