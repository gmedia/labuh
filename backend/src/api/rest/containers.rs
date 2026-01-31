use axum::{
    extract::{Extension, Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::api::middleware::auth::CurrentUser;
use crate::domain::runtime::{ContainerInfo, ContainerStats};
use crate::error::Result;
use crate::usecase::stack::StackUsecase;
use tokio::io::AsyncWriteExt;

#[derive(Deserialize)]
pub struct ListContainersQuery {
    #[serde(default)]
    all: bool,
    pub team_id: Option<String>,
}

#[derive(Deserialize)]
pub struct LogsQuery {
    #[serde(default = "default_tail")]
    tail: usize,
}

fn default_tail() -> usize {
    100
}

async fn list_containers(
    State(stack_usecase): State<Arc<StackUsecase>>,
    Extension(user): Extension<CurrentUser>,
    Query(query): Query<ListContainersQuery>,
) -> Result<Json<Vec<ContainerInfo>>> {
    let stacks = stack_usecase.list_stacks(&user.id).await?;
    let stack_ids: std::collections::HashSet<String> = stacks
        .into_iter()
        .filter(|s| {
            if let Some(ref team_id) = query.team_id {
                s.team_id == *team_id
            } else {
                true
            }
        })
        .map(|s| s.id)
        .collect();

    let all_containers = stack_usecase.runtime().list_containers(query.all).await?;
    let filtered: Vec<ContainerInfo> = all_containers
        .into_iter()
        .filter(|c| {
            c.labels
                .get("labuh.stack.id")
                .map(|id| stack_ids.contains(id))
                .unwrap_or(false)
        })
        .collect();

    Ok(Json(filtered))
}

// Handlers for specific container actions that check ownership
async fn start_container(
    State(stack_usecase): State<Arc<StackUsecase>>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    stack_usecase.start_container(&id, &user.id).await?;
    Ok(Json(serde_json::json!({ "status": "started" })))
}

async fn stop_container(
    State(stack_usecase): State<Arc<StackUsecase>>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    stack_usecase.stop_container(&id, &user.id).await?;
    Ok(Json(serde_json::json!({ "status": "stopped" })))
}

async fn restart_container(
    State(stack_usecase): State<Arc<StackUsecase>>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    stack_usecase.restart_container(&id, &user.id).await?;
    Ok(Json(serde_json::json!({ "status": "restarted" })))
}

async fn remove_container(
    State(stack_usecase): State<Arc<StackUsecase>>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    stack_usecase.remove_container(&id, &user.id).await?;
    Ok(Json(serde_json::json!({ "status": "removed" })))
}

async fn get_container_logs(
    State(stack_usecase): State<Arc<StackUsecase>>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
    Query(query): Query<LogsQuery>,
) -> Result<Json<Vec<String>>> {
    let logs = stack_usecase
        .get_container_logs(&id, &user.id, query.tail)
        .await?;
    Ok(Json(logs))
}

async fn get_container_stats(
    State(stack_usecase): State<Arc<StackUsecase>>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<ContainerStats>> {
    let stats = stack_usecase.get_container_stats(&id, &user.id).await?;
    Ok(Json(stats))
}

async fn container_exec(
    ws: axum::extract::ws::WebSocketUpgrade,
    State(stack_usecase): State<Arc<StackUsecase>>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<axum::response::Response> {
    // Verify ownership
    let _ = stack_usecase
        .verify_container_ownership(&id, &user.id)
        .await?;

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, stack_usecase, id)))
}

async fn handle_socket(
    mut socket: axum::extract::ws::WebSocket,
    stack_usecase: Arc<StackUsecase>,
    container_id: String,
) {
    use bollard::container::LogOutput;
    use bollard::exec::StartExecResults;
    use futures::StreamExt;

    // 1. Create exec instance (shell)
    let exec = match stack_usecase
        .runtime()
        .exec_command(&container_id, vec!["/bin/sh".to_string()])
        .await
    {
        Ok(e) => e,
        Err(e) => {
            let _ = socket
                .send(axum::extract::ws::Message::Text(
                    format!("Error: {}", e).into(),
                ))
                .await;
            return;
        }
    };

    // 2. Connect to the exec instance
    let attached = match stack_usecase.runtime().connect_exec(&exec.id).await {
        Ok(StartExecResults::Attached { output, input }) => (output, input),
        Ok(StartExecResults::Detached) => {
            let _ = socket
                .send(axum::extract::ws::Message::Text(
                    "Error: Exec started in detached mode".to_string().into(),
                ))
                .await;
            return;
        }
        Err(e) => {
            let _ = socket
                .send(axum::extract::ws::Message::Text(
                    format!("Error: {}", e).into(),
                ))
                .await;
            return;
        }
    };

    let (docker_rx, mut docker_tx) = attached;
    let mut docker_rx = docker_rx;

    // 3. Bridge the WebSocket and Docker exec stream
    loop {
        tokio::select! {
            // From Docker to WebSocket
            Some(Ok(output)) = docker_rx.next() => {
                match output {
                    LogOutput::StdOut { message } | LogOutput::StdErr { message } | LogOutput::Console { message } => {
                        if socket.send(axum::extract::ws::Message::Binary(message.into())).await.is_err() {
                            break;
                        }
                    }
                    _ => {}
                }
            }
            // From WebSocket to Docker
            Some(res) = socket.next() => {
                let msg = match res {
                    Ok(m) => m,
                    Err(_) => break,
                };

                match msg {
                    axum::extract::ws::Message::Binary(bin) => {
                        if AsyncWriteExt::write_all(&mut docker_tx, &bin).await.is_err() {
                            break;
                        }
                    }
                    axum::extract::ws::Message::Text(txt) => {
                        if AsyncWriteExt::write_all(&mut docker_tx, txt.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                    axum::extract::ws::Message::Close(_) => break,
                    _ => {}
                }
            }
            else => break,
        }
    }
}

pub fn container_routes(stack_usecase: Arc<StackUsecase>) -> Router {
    Router::new()
        .route("/", get(list_containers))
        .route("/{id}/start", post(start_container))
        .route("/{id}/stop", post(stop_container))
        .route("/{id}/restart", post(restart_container))
        .route("/{id}", delete(remove_container))
        .route("/{id}/logs", get(get_container_logs))
        .route("/{id}/stats", get(get_container_stats))
        .route("/{id}/exec", get(container_exec))
        .with_state(stack_usecase)
}
