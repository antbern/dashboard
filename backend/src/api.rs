use axum::{
    body::Body,
    extract::{Path, State},
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use common::{backend::RunId, WidgetEnum, WidgetId};
use std::{borrow::BorrowMut, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::{fs, sync::RwLock};
use tower::{ServiceBuilder, ServiceExt};

use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::{
    config::Config,
    database::{Database, DatabaseError, InMemoryDatabase},
    widget::{self, BackendStateStorage},
};
use common::backend::BackendRun;

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
    let api_router = Router::new()
        .route("/widgets", get(get_widgets))
        .route("/widget/:widget_id", get(get_widget))
        .route("/widget/:widget_id/run/:run_id", get(get_run))
        .route("/widget/:widget_id/runs", get(get_runs))
        .route("/widget/:widget_id/latest", get(get_last_run))
        .route("/widget/:widget_id/trigger", get(trigger_widget_run));

    let app = Router::new()
        .nest("/api", api_router)
        // Fallback to serving index.html for paths that were not found (to allow the yew SPA to work correctly)
        // See: https://robert.kra.hn/posts/2022-04-03_rust-web-wasm/
        .fallback(get(|req| async move {
            match ServeDir::new("../dist").oneshot(req).await {
                Ok(res) => {
                    let status = res.status();
                    match status {
                        StatusCode::NOT_FOUND => {
                            let index_path = PathBuf::from("../dist").join("index.html");
                            let index_content = match fs::read_to_string(index_path).await {
                                Err(_) => {
                                    return Response::builder()
                                        .status(StatusCode::NOT_FOUND)
                                        .body(Body::from("index file not found"))
                                        .unwrap()
                                }
                                Ok(index_content) => index_content,
                            };

                            Response::builder()
                                .status(StatusCode::OK)
                                .body(Body::from(index_content))
                                .unwrap()
                        }
                        _ => res.map(Body::new),
                    }
                }
                Err(err) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(format!("error: {err}")))
                    .expect("error response"),
            }
        }))
        .with_state(Arc::new(shared_state))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    // Run our app with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::debug!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

#[axum::debug_handler]
async fn get_widgets(State(state): State<Arc<AppState>>) -> Json<Vec<WidgetEnum>> {
    Json(state.widgets.to_vec())
}

#[axum::debug_handler]
async fn get_widget(
    Path(widget_id): Path<WidgetId>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<WidgetEnum>, DatabaseError> {
    let widget: &WidgetEnum = state
        .widgets
        .iter()
        .find(|w| {
            let id = match w {
                WidgetEnum::Weather(w) => &w.id,
            };

            *id == widget_id
        })
        .ok_or(DatabaseError::InvalidWidgetId)?;

    Ok(Json(widget.clone()))
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
    let widget: &WidgetEnum = state
        .widgets
        .iter()
        .find(|w| {
            let id = match w {
                WidgetEnum::Weather(w) => &w.id,
            };

            *id == widget_id
        })
        .ok_or(DatabaseError::InvalidWidgetId)?;
    let run = {
        let mut backend_state = state.backend_state.write().await;

        match widget {
            WidgetEnum::Weather(w) => widget::run(w, backend_state.borrow_mut()),
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
