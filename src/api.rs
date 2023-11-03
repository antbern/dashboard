use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

use crate::widget::{BackendRun, WidgetId};

type SharedState = Arc<RwLock<AppState>>;

#[derive(Default)]
struct AppState {
    runs: HashMap<WidgetId, Vec<BackendRun>>,
}

/// The main entrypoint for the Axum web server
pub async fn launch_api() -> anyhow::Result<()> {
    let shared_state = SharedState::default();

    // Build our application by composing routes
    let app = Router::new()
        .route("/run/:key", get(get_run))
        // .route("/keys", get(list_keys))
        // // Nest our admin routes under `/admin`
        // .nest("/admin", admin_routes())
        .with_state(Arc::clone(&shared_state));

    // Run our app with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn get_run(
    Path(key): Path<String>,
    State(state): State<SharedState>,
) -> Result<Bytes, StatusCode> {
    let runs = &state.read().await.runs;

    // if let Some(value) = db.get(&key) {
    //     Ok(value.clone())
    // } else {
    //     Err(StatusCode::NOT_FOUND)
    // }
    Err(StatusCode::NOT_FOUND)
}
