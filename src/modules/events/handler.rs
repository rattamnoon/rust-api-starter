use actix_web::{HttpResponse, web};
use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    modules::events::dto::EventsQuery,
    shared::{extractor::CurrentUser, response::ErrorResponseBody, state::AppState},
};

#[utoipa::path(
    get,
    path = "/api/v1/events",
    tag = "Events",
    security(("bearer_auth" = [])),
    params(EventsQuery),
    responses(
        (status = 200, body = crate::modules::events::dto::EventsListResponse),
        (status = 403, body = ErrorResponseBody)
    )
)]
pub async fn list_events(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    query: web::Query<EventsQuery>,
) -> Result<HttpResponse, AppError> {
    let response = state
        .event_service
        .list_events(&current_user.0, query.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/events/{id}",
    tag = "Events",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Event ID")),
    responses(
        (status = 200, body = crate::modules::events::dto::EventResponse),
        (status = 403, body = ErrorResponseBody),
        (status = 404, body = ErrorResponseBody)
    )
)]
pub async fn get_event(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    event_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let response = state
        .event_service
        .get_event(&current_user.0, event_id.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}
