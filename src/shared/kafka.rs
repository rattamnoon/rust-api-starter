use std::time::Duration;

use kafka::producer::{Producer, Record, RequiredAcks};
use serde_json::Value;

use crate::{config::settings::Settings, errors::app_error::AppError};

pub struct KafkaClient {
    brokers: Vec<String>,
    client_id: String,
}

impl KafkaClient {
    pub fn new(settings: &Settings) -> Result<Self, AppError> {
        let brokers = settings
            .kafka_brokers
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        if brokers.is_empty() {
            return Err(AppError::Config(
                "KAFKA_BROKERS must contain at least one broker".into(),
            ));
        }

        Ok(Self {
            brokers,
            client_id: settings.kafka_client_id.clone(),
        })
    }

    pub async fn publish_json(
        &self,
        topic: &str,
        key: &str,
        payload: &Value,
    ) -> Result<(), AppError> {
        let body = serde_json::to_string(payload)
            .map_err(|error| AppError::Internal(error.to_string()))?;
        let mut producer = Producer::from_hosts(self.brokers.clone())
            .with_client_id(self.client_id.clone())
            .with_ack_timeout(Duration::from_secs(5))
            .with_required_acks(RequiredAcks::One)
            .create()
            .map_err(|error| {
                AppError::Internal(format!("failed to create kafka producer: {error}"))
            })?;

        producer
            .send(&Record::from_key_value(topic, key, body.as_bytes()))
            .map_err(|error| AppError::Internal(format!("kafka publish failed: {error}")))
    }
}
