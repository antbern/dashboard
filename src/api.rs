use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use std::{borrow::BorrowMut, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

use crate::{
    config::{Config, WidgetEnum},
    database::{Database, DatabaseError, InMemoryDatabase},
    widget::{BackendRun, BackendStateStorage, RunId, WidgetId},
};

// type SharedState = Arc<RwLock<AppState>>;

// #[derive(Default)]
struct AppState {
    db: Arc<RwLock<dyn Database + Send + Sync>>,
    widgets: Arc<Vec<WidgetEnum>>,
    backend_state: Arc<RwLock<BackendStateStorage>>,
}

/// The main entrypoint for the Axum web server
pub async fn launch_api(config: Config) -> anyhow::Result<()> {
    let shared_state = AppState {
        db: Arc::new(RwLock::new(InMemoryDatabase::new())),
        widgets: Arc::new(config.widgets),
        backend_state: Arc::new(RwLock::new(BackendStateStorage::new())),
    };

    // Build our application by composing routes
    let app = Router::new()
        .route("/widget/:widget_id/run/:run_id", get(get_run))
        .route("/widget/:widget_id/runs", get(get_runs))
        .route("/widget/:widget_id/latest", get(get_last_run))
        .route("/widget/:widget_id/trigger", get(trigger_widget_run))
        // .route("/keys", get(list_keys))
        // // Nest our admin routes under `/admin`
        // .nest("/admin", admin_routes())
        .with_state(Arc::new(shared_state));

    // Run our app with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

#[axum::debug_handler]
async fn get_run(
    Path((widget_id, run_id)): Path<(WidgetId, RunId)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<BackendRun>, DatabaseError> {
    let run = state.db.read().await.get_run(widget_id, run_id)?;

    Ok(Json(run))
}

#[axum::debug_handler]
async fn get_runs(
    Path(widget_id): Path<WidgetId>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<BackendRun>>, DatabaseError> {
    let run = state.db.read().await.get_runs(widget_id)?;

    Ok(Json(run))
}

#[axum::debug_handler]
async fn get_last_run(
    Path(widget_id): Path<WidgetId>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<BackendRun>, DatabaseError> {
    let run = state.db.read().await.get_last_run(widget_id)?;

    Ok(Json(run))
}

#[axum::debug_handler]
async fn trigger_widget_run(
    Path(widget_id): Path<WidgetId>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<RunId>, DatabaseError> {
    // run the backend handler and store the result (on the main thread for now, not optimal)
    // TODO: offload this responsibility to some background service

    // find the widget (this is ridicoulously difficult haha)
    let widgets: &WidgetEnum = state
        .widgets
        .iter()
        .find(|w| {
            let id = match w {
                WidgetEnum::Weather(w) => w.id(),
            };

            id == widget_id
        })
        .ok_or(DatabaseError::InvalidWidgetId)?;
    let run = {
        let mut backend_state = state.backend_state.write().await;

        match widgets {
            WidgetEnum::Weather(w) => w.run(backend_state.borrow_mut()),
        }
    };
    let id = state.db.write().await.insert_run(widget_id, run)?;

    Ok(Json(id))
}

impl IntoResponse for DatabaseError {
    fn into_response(self) -> axum::response::Response {
        match self {
            DatabaseError::InvalidRunId => (StatusCode::NOT_FOUND, "Invalid Run ID"),
            DatabaseError::InvalidWidgetId => (StatusCode::NOT_FOUND, "Invalid Widget ID"),
            DatabaseError::NoneAvailable => (StatusCode::NOT_FOUND, "No Runs available"),
        }
        .into_response()
    }
}
