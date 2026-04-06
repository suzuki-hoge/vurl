use actix_web::{HttpResponse, Responder, get, post, web};
use anyhow::Error as AnyhowError;
use serde::Deserialize;
use serde_json::json;

use crate::{
    domain::api::{
        AuthPresetSummary, DefinitionResponse, EnvironmentSummary, LogFileResponse, ProjectSummary,
        ResponseNotification, ResponseNotificationCode, ResponseNotificationKind, RuntimeInfo,
        SendRequest, SendResponse, TreeResponse,
    },
    domain::auth::AuthEnvironment,
    services::{
        logging::create_manual_log_file,
        request_execution::{REQUEST_TIMEOUT_MS, execute_request},
    },
    state::app_state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ProjectQuery {
    pub project: String,
}

#[derive(Debug, Deserialize)]
pub struct DefinitionQuery {
    pub project: String,
    pub path: String,
}

#[get("/api/runtime")]
pub async fn runtime(state: web::Data<AppState>) -> actix_web::Result<impl Responder> {
    let project_list: Vec<ProjectSummary> = state
        .store
        .project_names()
        .into_iter()
        .map(|name| ProjectSummary { name })
        .collect();

    Ok(web::Json(RuntimeInfo {
        root: state.store.paths.root.display().to_string(),
        projects: project_list,
        backend_url: state.backend_url.clone(),
    }))
}

#[get("/api/projects")]
pub async fn projects(state: web::Data<AppState>) -> actix_web::Result<impl Responder> {
    let projects: Vec<_> = state
        .store
        .project_names()
        .into_iter()
        .map(|name| ProjectSummary { name })
        .collect();
    Ok(web::Json(projects))
}

#[get("/api/environments")]
pub async fn environments(
    state: web::Data<AppState>,
    query: web::Query<ProjectQuery>,
) -> actix_web::Result<impl Responder> {
    let project = state
        .store
        .project(&query.project)
        .map_err(actix_web::error::ErrorBadRequest)?;
    let mut names: Vec<_> = project.environments.keys().cloned().collect();
    names.sort();
    let items = names
        .into_iter()
        .map(|name| {
            let auth_presets = match project.auth.environments.get(&name) {
                Some(AuthEnvironment::Fixed { credentials, .. }) => credentials
                    .presets
                    .iter()
                    .map(|preset| AuthPresetSummary {
                        name: preset.name.clone(),
                    })
                    .collect(),
                Some(AuthEnvironment::Http { credentials, .. }) => credentials
                    .as_ref()
                    .map(|value| {
                        value
                            .presets
                            .iter()
                            .map(|preset| AuthPresetSummary {
                                name: preset.name.clone(),
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                None => Vec::new(),
            };
            EnvironmentSummary { name, auth_presets }
        })
        .collect::<Vec<_>>();
    Ok(web::Json(items))
}

#[get("/api/tree")]
pub async fn tree(
    state: web::Data<AppState>,
    query: web::Query<ProjectQuery>,
) -> actix_web::Result<impl Responder> {
    let nodes = state
        .store
        .tree(&query.project)
        .map_err(actix_web::error::ErrorBadRequest)?;
    Ok(web::Json(TreeResponse {
        project: query.project.clone(),
        nodes,
    }))
}

#[get("/api/definition")]
pub async fn definition(
    state: web::Data<AppState>,
    query: web::Query<DefinitionQuery>,
) -> actix_web::Result<impl Responder> {
    let definition = state
        .store
        .request_definition(&query.project, &query.path)
        .map_err(actix_web::error::ErrorBadRequest)?;
    Ok(web::Json(DefinitionResponse {
        path: query.path.clone(),
        definition,
    }))
}

#[post("/api/send")]
pub async fn send(
    state: web::Data<AppState>,
    payload: web::Json<SendRequest>,
) -> actix_web::Result<impl Responder> {
    match execute_request(&state.store, payload.into_inner()).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(error) if is_timeout_error(&error) => {
            Ok(HttpResponse::InternalServerError().json(build_timeout_response()))
        }
        Err(error) => Err(actix_web::error::ErrorBadRequest(error)),
    }
}

#[post("/api/logs/new")]
pub async fn new_log(
    state: web::Data<AppState>,
    payload: web::Json<ProjectQuery>,
) -> actix_web::Result<impl Responder> {
    let file = create_manual_log_file(&state.store, &payload.project)
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(web::Json(LogFileResponse {
        project: payload.project.clone(),
        current_log_file: file.display().to_string(),
    }))
}

pub async fn not_found() -> impl Responder {
    HttpResponse::NotFound().body("not found")
}

fn is_timeout_error(error: &AnyhowError) -> bool {
    error
        .chain()
        .filter_map(|cause| cause.downcast_ref::<reqwest::Error>())
        .any(reqwest::Error::is_timeout)
}

fn build_timeout_response() -> SendResponse {
    let body = json!({
        "error": "request_timeout",
        "message": format!("request timed out after {}ms", REQUEST_TIMEOUT_MS),
    })
    .to_string();

    SendResponse {
        status: 500,
        headers: Vec::new(),
        content_type: Some("application/json".to_string()),
        body,
        body_base64: None,
        retried_auth: false,
        notifications: vec![ResponseNotification {
            code: ResponseNotificationCode::Timeout,
            kind: ResponseNotificationKind::Error,
            message: format!(
                "リクエスト先との通信が {REQUEST_TIMEOUT_MS}ms でタイムアウトしました"
            ),
        }],
        current_log_file: String::new(),
    }
}
