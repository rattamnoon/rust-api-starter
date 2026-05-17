use chrono::Utc;
use uuid::Uuid;

use crate::{
    config::settings::Settings,
    errors::app_error::AppError,
    modules::{
        orders::{model::OrderStatus, repository::OrderRepository},
        receipts::{
            dto::ReceiptResponse,
            model::{Receipt, ReceiptStatus},
            repository::ReceiptRepository,
        },
        uploads::repository::{NewUploadedFile, UploadRepository},
        users::repository::UserRepository,
    },
    shared::{
        email::{EmailMessage, EmailService},
        extractor::AuthenticatedUser,
        file_storage::LocalFileStorage,
        pdf::{ReceiptPdfInput, ReceiptPdfLineItem, render_receipt_pdf},
        types::UserRole,
    },
};

#[derive(Clone)]
pub struct ReceiptService {
    settings: Settings,
    receipt_repository: ReceiptRepository,
    order_repository: OrderRepository,
    upload_repository: UploadRepository,
    user_repository: UserRepository,
    email_service: EmailService,
    file_storage: LocalFileStorage,
}

impl ReceiptService {
    pub fn new(
        settings: Settings,
        receipt_repository: ReceiptRepository,
        order_repository: OrderRepository,
        upload_repository: UploadRepository,
        user_repository: UserRepository,
        email_service: EmailService,
        file_storage: LocalFileStorage,
    ) -> Self {
        Self {
            settings,
            receipt_repository,
            order_repository,
            upload_repository,
            user_repository,
            email_service,
            file_storage,
        }
    }

    pub async fn generate_and_send_receipt(
        &self,
        order_id: Uuid,
        external_event_id: &str,
    ) -> Result<Receipt, AppError> {
        let order = self
            .order_repository
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("order was not found".into()))?;
        let order_status = order.status().map_err(AppError::Internal)?;
        if !matches!(order_status, OrderStatus::Paid | OrderStatus::Fulfilled) {
            return Err(AppError::BadRequest(
                "receipt can only be generated for paid orders".into(),
            ));
        }

        if let Some(existing) = self.receipt_repository.find_by_order_id(order_id).await?
            && matches!(
                existing.status().map_err(AppError::Internal)?,
                ReceiptStatus::Generated | ReceiptStatus::Emailed
            )
        {
            return Ok(existing);
        }

        let user = self
            .user_repository
            .find_by_id(order.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user was not found".into()))?;
        let items = self.order_repository.list_items(order_id).await?;
        let receipt_number = format!(
            "{}-{}-{}",
            self.settings.receipt_prefix,
            Utc::now().format("%Y%m%d"),
            &order_id.to_string()[..8]
        );
        let receipt = self
            .receipt_repository
            .create_or_get(order_id, &receipt_number)
            .await?;

        let pdf_items = items
            .iter()
            .map(|item| ReceiptPdfLineItem {
                name: &item.product_name_snapshot,
                quantity: item.quantity,
                unit_price_amount: item.unit_price_amount,
                line_total_amount: item.line_total_amount,
            })
            .collect::<Vec<_>>();
        let pdf_bytes = render_receipt_pdf(&ReceiptPdfInput {
            receipt_number: &receipt.receipt_number,
            issued_at: receipt.issued_at,
            customer_name: &user.name,
            customer_email: &user.email,
            order_id: &order.id.to_string(),
            currency: &order.currency,
            total_amount: order.total_amount,
            payment_reference: order.stripe_payment_intent_id.as_deref().or(Some(external_event_id)),
            items: &pdf_items,
        });

        let stored_filename = format!("{}.pdf", Uuid::now_v7());
        let stored_path = self
            .file_storage
            .store("receipts", &stored_filename, &pdf_bytes)
            .await?;
        let uploaded_file = self
            .upload_repository
            .create(NewUploadedFile {
                sub_folder: "receipts".to_string(),
                original_filename: format!("{}.pdf", receipt.receipt_number),
                stored_filename: stored_filename.clone(),
                content_type: Some("application/pdf".to_string()),
                size_bytes: pdf_bytes.len() as i64,
                storage_path: stored_path.to_string_lossy().to_string(),
                uploaded_by: order.user_id,
            })
            .await?;

        let generated_receipt = self
            .receipt_repository
            .mark_generated(receipt.id, uploaded_file.id)
            .await?;

        let subject = format!("Receipt {}", generated_receipt.receipt_number);
        let delivery = self
            .receipt_repository
            .create_email_delivery(generated_receipt.id, &user.email, &subject)
            .await?;
        let email_html = format!(
            "<p>Hello {},</p><p>Your payment was successful.</p><p>Receipt number: <strong>{}</strong></p><p>Download PDF: <a href=\"{}/static/receipts/{}\">receipt PDF</a></p>",
            user.name,
            generated_receipt.receipt_number,
            self.settings.public_base_url,
            stored_filename
        );

        match self
            .email_service
            .send(&EmailMessage {
                to: user.email.clone(),
                subject,
                html: email_html,
            })
            .await
        {
            Ok(result) => {
                let _ = self
                    .receipt_repository
                    .update_email_delivery(
                        delivery.id,
                        "sent",
                        result.provider_message_id.as_deref(),
                        None,
                    )
                    .await?;
                self.receipt_repository
                    .mark_delivery_status(generated_receipt.id, ReceiptStatus::Emailed)
                    .await
                    .map_err(Into::into)
            }
            Err(error) => {
                let _ = self
                    .receipt_repository
                    .update_email_delivery(
                        delivery.id,
                        "failed",
                        None,
                        Some(&error.to_string()),
                    )
                    .await?;
                let _ = self
                    .receipt_repository
                    .mark_delivery_status(generated_receipt.id, ReceiptStatus::EmailFailed)
                    .await?;
                Err(error)
            }
        }
    }

    pub async fn get_receipt(
        &self,
        actor: &AuthenticatedUser,
        receipt_id: Uuid,
    ) -> Result<ReceiptResponse, AppError> {
        let receipt = self
            .receipt_repository
            .find_by_id(receipt_id)
            .await?
            .ok_or_else(|| AppError::NotFound("receipt was not found".into()))?;
        self.ensure_access(actor, &receipt).await?;
        let pdf_url = self.pdf_url(&receipt).await?;
        ReceiptResponse::from_model(receipt, pdf_url)
    }

    pub async fn get_pdf_upload_id(
        &self,
        actor: &AuthenticatedUser,
        receipt_id: Uuid,
    ) -> Result<Uuid, AppError> {
        let receipt = self
            .receipt_repository
            .find_by_id(receipt_id)
            .await?
            .ok_or_else(|| AppError::NotFound("receipt was not found".into()))?;
        self.ensure_access(actor, &receipt).await?;
        receipt
            .upload_id
            .ok_or_else(|| AppError::NotFound("receipt PDF was not found".into()))
    }

    pub async fn resend_receipt(
        &self,
        actor: &AuthenticatedUser,
        receipt_id: Uuid,
    ) -> Result<ReceiptResponse, AppError> {
        let receipt = self
            .receipt_repository
            .find_by_id(receipt_id)
            .await?
            .ok_or_else(|| AppError::NotFound("receipt was not found".into()))?;
        self.ensure_access(actor, &receipt).await?;
        self.generate_and_send_receipt(receipt.order_id, "manual-resend")
            .await?;
        let updated = self
            .receipt_repository
            .find_by_id(receipt_id)
            .await?
            .ok_or_else(|| AppError::NotFound("receipt was not found".into()))?;
        let pdf_url = self.pdf_url(&updated).await?;
        ReceiptResponse::from_model(updated, pdf_url)
    }

    async fn ensure_access(&self, actor: &AuthenticatedUser, receipt: &Receipt) -> Result<(), AppError> {
        let order = self
            .order_repository
            .find_by_id(receipt.order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("order was not found".into()))?;
        if actor.role == UserRole::Admin || actor.user_id == order.user_id {
            Ok(())
        } else {
            Err(AppError::Forbidden(
                "you can only access your own receipts".into(),
            ))
        }
    }

    async fn pdf_url(&self, receipt: &Receipt) -> Result<Option<String>, AppError> {
        let Some(upload_id) = receipt.upload_id else {
            return Ok(None);
        };
        let upload = self
            .upload_repository
            .find_by_id(upload_id)
            .await?
            .ok_or_else(|| AppError::NotFound("receipt PDF was not found".into()))?;
        Ok(Some(format!(
            "/static/{}/{}",
            upload.sub_folder, upload.stored_filename
        )))
    }
}
