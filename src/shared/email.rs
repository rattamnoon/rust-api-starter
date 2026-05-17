use reqwest::StatusCode;
use serde::Serialize;

use crate::{config::settings::Settings, errors::app_error::AppError};

#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub html: String,
}

#[derive(Debug, Clone)]
pub struct EmailDeliveryResult {
    pub provider: String,
    pub provider_message_id: Option<String>,
}

#[derive(Clone)]
pub struct EmailService {
    provider: String,
    from: String,
    resend_api_key: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct ResendEmailRequest<'a> {
    from: &'a str,
    to: Vec<&'a str>,
    subject: &'a str,
    html: &'a str,
}

impl EmailService {
    pub fn new(settings: &Settings) -> Self {
        Self {
            provider: settings.email_provider.clone(),
            from: settings.email_from.clone(),
            resend_api_key: settings.resend_api_key.clone(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn send(&self, message: &EmailMessage) -> Result<EmailDeliveryResult, AppError> {
        match self.provider.as_str() {
            "resend" => self.send_resend(message).await,
            _ => {
                tracing::info!(
                    recipient = %message.to,
                    subject = %message.subject,
                    "simulated email delivery"
                );
                Ok(EmailDeliveryResult {
                    provider: "log".to_string(),
                    provider_message_id: None,
                })
            }
        }
    }

    async fn send_resend(&self, message: &EmailMessage) -> Result<EmailDeliveryResult, AppError> {
        if self.resend_api_key.is_empty() {
            return Err(AppError::Config(
                "RESEND_API_KEY is required when EMAIL_PROVIDER=resend".into(),
            ));
        }

        let response = self
            .client
            .post("https://api.resend.com/emails")
            .bearer_auth(&self.resend_api_key)
            .json(&ResendEmailRequest {
                from: &self.from,
                to: vec![&message.to],
                subject: &message.subject,
                html: &message.html,
            })
            .send()
            .await?;

        if response.status() != StatusCode::OK && response.status() != StatusCode::CREATED {
            return Err(AppError::Internal(format!(
                "email provider rejected request with status {}",
                response.status()
            )));
        }

        let body: serde_json::Value = response.json().await?;
        Ok(EmailDeliveryResult {
            provider: "resend".to_string(),
            provider_message_id: body
                .get("id")
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string),
        })
    }
}
