use std::time::Instant;

use actix_web::{
    Error,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
};

pub async fn log_request(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let method = req.method().clone();
    let path = req.path().to_string();
    let start = Instant::now();

    let response = next.call(req).await?;
    let status = response.status().as_u16();
    let elapsed_ms = start.elapsed().as_millis();

    tracing::info!(
        method = %method,
        path = %path,
        status = status,
        elapsed_ms = elapsed_ms,
        "http request completed"
    );

    Ok(response)
}
