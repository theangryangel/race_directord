use crate::config::web::Web;
use axum::Router;

pub(crate) fn start(config: &Web) {
    let app = Router::new();

    tracing::info!("Web listening on {:?}", &config.address);

    tokio::task::spawn(axum::Server::bind(&config.address.unwrap()).serve(app.into_make_service()));
}
