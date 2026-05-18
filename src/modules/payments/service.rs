use std::collections::BTreeMap;

use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::json;
use sha2::Sha256;
use uuid::Uuid;

use crate::{
    config::settings::Settings,
    errors::app_error::AppError,
    modules::{
        events::service::EventService,
        jobs::service::JobService,
        orders::{
            dto::OrderResponse,
            model::{Order, OrderStatus},
            repository::OrderRepository,
        },
        payments::{
            dto::{CheckoutOrderRequest, CheckoutSessionResponse, PaymentWebhookAcceptedResponse},
            model::PaymentStatus,
            repository::{NewPayment, PaymentRepository},
        },
    },
    shared::{extractor::AuthenticatedUser, types::UserRole},
};

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct PaymentService {
    settings: Settings,
    order_repository: OrderRepository,
    payment_repository: PaymentRepository,
    job_service: JobService,
    event_service: EventService,
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct StripeCheckoutSessionResponse {
    id: String,
    url: String,
    payment_intent: Option<String>,
}

#[derive(Deserialize)]
struct StripeEventEnvelope {
    id: String,
    #[serde(rename = "type")]
    event_type: String,
    data: StripeEventData,
}

#[derive(Deserialize)]
struct StripeEventData {
    object: StripeCheckoutSessionObject,
}

#[derive(Deserialize)]
struct StripeCheckoutSessionObject {
    id: String,
    payment_intent: Option<String>,
    payment_status: Option<String>,
    amount_total: Option<i64>,
    currency: Option<String>,
    metadata: Option<BTreeMap<String, String>>,
}

impl PaymentService {
    pub fn new(
        settings: Settings,
        order_repository: OrderRepository,
        payment_repository: PaymentRepository,
        job_service: JobService,
        event_service: EventService,
    ) -> Self {
        Self {
            settings,
            order_repository,
            payment_repository,
            job_service,
            event_service,
            client: reqwest::Client::new(),
        }
    }

    pub async fn create_checkout_session(
        &self,
        actor: &AuthenticatedUser,
        order_id: Uuid,
        request: CheckoutOrderRequest,
    ) -> Result<CheckoutSessionResponse, AppError> {
        let order = self
            .order_repository
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("order was not found".into()))?;
        ensure_owner_or_admin(actor, &order)?;
        ensure_checkout_allowed(&order)?;

        if self.settings.stripe_secret_key.is_empty() {
            return Err(AppError::Config(
                "STRIPE_SECRET_KEY is required to create checkout sessions".into(),
            ));
        }

        let items = self.order_repository.list_items(order.id).await?;
        let mut form = vec![
            ("mode".to_string(), "payment".to_string()),
            (
                "success_url".to_string(),
                request
                    .success_url
                    .unwrap_or_else(|| self.settings.stripe_success_url.clone()),
            ),
            (
                "cancel_url".to_string(),
                request
                    .cancel_url
                    .unwrap_or_else(|| self.settings.stripe_cancel_url.clone()),
            ),
            ("metadata[order_id]".to_string(), order.id.to_string()),
        ];

        for (index, item) in items.iter().enumerate() {
            form.push((
                format!("line_items[{index}][price_data][currency]"),
                order.currency.clone(),
            ));
            form.push((
                format!("line_items[{index}][price_data][unit_amount]"),
                item.unit_price_amount.to_string(),
            ));
            form.push((
                format!("line_items[{index}][price_data][product_data][name]"),
                item.product_name_snapshot.clone(),
            ));
            form.push((
                format!("line_items[{index}][quantity]"),
                item.quantity.to_string(),
            ));
        }

        let session = self
            .client
            .post("https://api.stripe.com/v1/checkout/sessions")
            .basic_auth(&self.settings.stripe_secret_key, Some(""))
            .form(&form)
            .send()
            .await?
            .error_for_status()?
            .json::<StripeCheckoutSessionResponse>()
            .await?;

        self.order_repository
            .mark_pending_payment(order.id, &session.id)
            .await?;
        if let Some(payment_intent) = session.payment_intent.as_deref() {
            self.order_repository
                .attach_payment_intent(order.id, payment_intent)
                .await?;
        }
        self.payment_repository
            .upsert_checkout_payment(NewPayment {
                order_id: order.id,
                provider: "stripe".to_string(),
                provider_payment_id: session.payment_intent.clone(),
                provider_session_id: Some(session.id.clone()),
                status: PaymentStatus::Pending.as_str().to_string(),
                amount: order.total_amount,
                currency: order.currency.clone(),
                raw_payload: json!({
                    "checkout_session_id": session.id,
                    "checkout_url": session.url,
                }),
            })
            .await?;

        let updated_order = self
            .order_repository
            .find_by_id(order.id)
            .await?
            .ok_or_else(|| AppError::NotFound("order was not found".into()))?;
        let updated_items = self.order_repository.list_items(order.id).await?;

        Ok(CheckoutSessionResponse {
            order: OrderResponse::from_parts(updated_order, updated_items)?,
            provider: "stripe".to_string(),
            checkout_session_id: session.id,
            checkout_url: session.url,
        })
    }

    pub async fn handle_stripe_webhook(
        &self,
        signature: Option<&str>,
        payload: &[u8],
    ) -> Result<PaymentWebhookAcceptedResponse, AppError> {
        self.verify_stripe_signature(signature, payload)?;

        let event: StripeEventEnvelope = serde_json::from_slice(payload)
            .map_err(|error| AppError::BadRequest(format!("invalid stripe payload: {error}")))?;
        let inserted = self
            .payment_repository
            .register_webhook_event(
                &event.id,
                &event.event_type,
                serde_json::from_slice(payload)
                    .map_err(|error| AppError::Internal(error.to_string()))?,
            )
            .await?;

        if !inserted {
            return Ok(PaymentWebhookAcceptedResponse {
                accepted: true,
                job_id: None,
                published_event_types: Vec::new(),
                status: None,
            });
        }

        if event.event_type != "checkout.session.completed" {
            return Ok(PaymentWebhookAcceptedResponse {
                accepted: true,
                job_id: None,
                published_event_types: Vec::new(),
                status: None,
            });
        }

        let order_id = event
            .data
            .object
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("order_id"))
            .ok_or_else(|| AppError::BadRequest("stripe metadata.order_id is required".into()))?
            .parse::<Uuid>()
            .map_err(|_| AppError::BadRequest("stripe metadata.order_id is invalid".into()))?;

        let order = self
            .order_repository
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("order was not found".into()))?;

        let payment_status = match event.data.object.payment_status.as_deref() {
            Some("paid") => PaymentStatus::Succeeded,
            Some("unpaid") | None => PaymentStatus::Pending,
            Some(_) => PaymentStatus::Failed,
        };

        self.payment_repository
            .upsert_checkout_payment(NewPayment {
                order_id,
                provider: "stripe".to_string(),
                provider_payment_id: event.data.object.payment_intent.clone(),
                provider_session_id: Some(event.data.object.id.clone()),
                status: payment_status.as_str().to_string(),
                amount: event.data.object.amount_total.unwrap_or(order.total_amount),
                currency: event
                    .data
                    .object
                    .currency
                    .clone()
                    .unwrap_or(order.currency.clone()),
                raw_payload: serde_json::from_slice(payload)
                    .map_err(|error| AppError::Internal(error.to_string()))?,
            })
            .await?;

        if let Some(payment_intent_id) = event.data.object.payment_intent.as_deref() {
            self.order_repository
                .attach_payment_intent(order_id, payment_intent_id)
                .await?;
        }

        let (job_id, published_event_types) = if payment_status == PaymentStatus::Succeeded {
            self.order_repository.mark_paid(order_id).await?;
            let paid_order = self
                .order_repository
                .find_by_id(order_id)
                .await?
                .ok_or_else(|| AppError::NotFound("order was not found".into()))?;
            let published_event_type = self
                .event_service
                .record_order_paid(&paid_order, event.data.object.payment_intent.as_deref())
                .await?;
            let job = self
                .job_service
                .enqueue_receipt_generation(order_id, &event.id, Some(order.user_id))
                .await?;
            (Some(job.id), vec![published_event_type])
        } else {
            (None, Vec::new())
        };

        Ok(PaymentWebhookAcceptedResponse {
            accepted: true,
            job_id,
            published_event_types,
            status: Some(payment_status),
        })
    }

    fn verify_stripe_signature(
        &self,
        signature: Option<&str>,
        payload: &[u8],
    ) -> Result<(), AppError> {
        if self.settings.stripe_webhook_secret.is_empty() {
            return Err(AppError::Config(
                "STRIPE_WEBHOOK_SECRET is required to verify webhooks".into(),
            ));
        }
        let signature = signature
            .ok_or_else(|| AppError::Unauthorized("stripe signature header is required".into()))?;
        let mut timestamp = None;
        let mut v1 = None;
        for part in signature.split(',') {
            if let Some((key, value)) = part.split_once('=') {
                match key {
                    "t" => timestamp = Some(value),
                    "v1" => v1 = Some(value),
                    _ => {}
                }
            }
        }

        let timestamp = timestamp.ok_or_else(|| {
            AppError::Unauthorized("stripe signature timestamp is missing".into())
        })?;
        let signature_hash =
            v1.ok_or_else(|| AppError::Unauthorized("stripe signature hash is missing".into()))?;

        let payload_to_sign = format!("{timestamp}.{}", String::from_utf8_lossy(payload));
        let mut mac = HmacSha256::new_from_slice(self.settings.stripe_webhook_secret.as_bytes())
            .map_err(|error| AppError::Internal(error.to_string()))?;
        mac.update(payload_to_sign.as_bytes());
        let expected = hex::encode(mac.finalize().into_bytes());
        if expected != signature_hash {
            return Err(AppError::Unauthorized(
                "stripe signature verification failed".into(),
            ));
        }

        let timestamp = timestamp
            .parse::<i64>()
            .map_err(|_| AppError::Unauthorized("stripe signature timestamp is invalid".into()))?;
        if (Utc::now().timestamp() - timestamp).abs() > 300 {
            return Err(AppError::Unauthorized(
                "stripe signature is outside the allowed time window".into(),
            ));
        }

        Ok(())
    }
}

fn ensure_owner_or_admin(actor: &AuthenticatedUser, order: &Order) -> Result<(), AppError> {
    if actor.role == UserRole::Admin || actor.user_id == order.user_id {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "you can only access your own orders".into(),
        ))
    }
}

fn ensure_checkout_allowed(order: &Order) -> Result<(), AppError> {
    match order.status().map_err(AppError::Internal)? {
        OrderStatus::Draft | OrderStatus::PendingPayment => Ok(()),
        _ => Err(AppError::BadRequest(
            "only draft or pending orders can be checked out".into(),
        )),
    }
}
