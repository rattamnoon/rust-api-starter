use std::net::SocketAddr;

use actix_web::{
    Error, HttpResponse,
    body::{BoxBody, EitherBody, MessageBody},
    dev::{ServiceRequest, ServiceResponse},
    http::header::{HeaderValue, RETRY_AFTER},
    middleware::Next,
    web,
};

use crate::shared::{
    response::{ErrorDetails, ErrorResponseBody},
    state::AppState,
};

pub async fn rate_limit(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<EitherBody<impl MessageBody, BoxBody>>, Error> {
    let Some(state) = req.app_data::<web::Data<AppState>>() else {
        let response = req.into_response(
            HttpResponse::InternalServerError()
                .json(ErrorResponseBody {
                    error: ErrorDetails {
                        code: "internal_error".to_string(),
                        message: "application state is missing".to_string(),
                        details: None,
                    },
                })
                .map_into_right_body(),
        );
        return Ok(response);
    };

    let client_key = client_identifier(req.request());
    let decision = state.rate_limiter.check(&client_key);
    if !decision.allowed {
        let mut response = HttpResponse::TooManyRequests();
        response.insert_header((
            RETRY_AFTER,
            HeaderValue::from_str(&decision.retry_after_seconds.to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("1")),
        ));

        let response = req.into_response(
            response
                .json(ErrorResponseBody {
                    error: ErrorDetails {
                        code: "rate_limited".to_string(),
                        message: "too many requests".to_string(),
                        details: None,
                    },
                })
                .map_into_right_body(),
        );
        return Ok(response);
    }

    let response = next.call(req).await?.map_into_left_body();
    Ok(response)
}

fn client_identifier(request: &actix_web::HttpRequest) -> String {
    request
        .connection_info()
        .realip_remote_addr()
        .map(normalize_client_ip)
        .or_else(|| request.peer_addr().map(|addr| addr.ip().to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}

fn normalize_client_ip(value: &str) -> String {
    if let Ok(socket_addr) = value.parse::<SocketAddr>() {
        return socket_addr.ip().to_string();
    }

    value
        .split(',')
        .next()
        .map(str::trim)
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_port_from_socket_addr() {
        assert_eq!(normalize_client_ip("127.0.0.1:8080"), "127.0.0.1");
    }

    #[test]
    fn handles_forwarded_for_header_like_values() {
        assert_eq!(
            normalize_client_ip("203.0.113.10, 10.0.0.1"),
            "203.0.113.10"
        );
    }
}
