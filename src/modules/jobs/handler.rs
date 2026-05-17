use actix_web::{HttpResponse, web};
use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    modules::jobs::dto::JobsQuery,
    shared::{extractor::CurrentUser, response::ErrorResponseBody, state::AppState},
};

#[utoipa::path(
    get,
    path = "/api/v1/jobs",
    tag = "Jobs",
    security(
        ("bearer_auth" = [])
    ),
    params(JobsQuery),
    responses(
        (status = 200, description = "List jobs", body = crate::modules::jobs::dto::JobsListResponse),
        (status = 403, description = "Forbidden", body = ErrorResponseBody)
    )
)]
pub async fn list_jobs(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    query: web::Query<JobsQuery>,
) -> Result<HttpResponse, AppError> {
    let response = state
        .job_service
        .list_jobs(&current_user.0, query.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/{id}",
    tag = "Jobs",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = Uuid, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Job detail", body = crate::modules::jobs::dto::JobDetailResponse),
        (status = 403, description = "Forbidden", body = ErrorResponseBody),
        (status = 404, description = "Not found", body = ErrorResponseBody)
    )
)]
pub async fn get_job(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    job_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let response = state
        .job_service
        .get_job(&current_user.0, job_id.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/jobs/{id}/retry",
    tag = "Jobs",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = Uuid, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Retry queued", body = crate::modules::jobs::dto::JobResponse),
        (status = 400, description = "Bad request", body = ErrorResponseBody),
        (status = 403, description = "Forbidden", body = ErrorResponseBody)
    )
)]
pub async fn retry_job(
    state: web::Data<AppState>,
    current_user: CurrentUser,
    job_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let response = state
        .job_service
        .retry_job(&current_user.0, job_id.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/charts/summary",
    tag = "Jobs",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Chart summary", body = crate::modules::jobs::dto::JobsChartSummaryResponse),
        (status = 403, description = "Forbidden", body = ErrorResponseBody)
    )
)]
pub async fn chart_summary(
    state: web::Data<AppState>,
    current_user: CurrentUser,
) -> Result<HttpResponse, AppError> {
    let response = state.job_service.chart_summary(&current_user.0).await?;
    Ok(HttpResponse::Ok().json(response))
}
