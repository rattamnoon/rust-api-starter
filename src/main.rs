use std::io;

use actix_web::{App, HttpServer, middleware::from_fn, web};
use rust_api_starter::{
    app, config::settings::Settings, db::pool::create_pool, logging, middleware,
    shared::state::AppState,
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

    let bind_address = format!("{}:{}", settings.server_host, settings.server_port);
    let state = AppState::new(settings.clone(), pool)
        .await
        .map_err(io::Error::other)?;

    tracing::info!("starting server at http://{bind_address}");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(from_fn(middleware::rate_limit::rate_limit))
            .wrap(from_fn(middleware::request_logging::log_request))
            .wrap(actix_cors::Cors::permissive())
            .configure(app::configure)
    })
    .bind(bind_address)?
    .run()
    .await
}
