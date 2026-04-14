use actix_web::{HttpResponse, Responder, get, post, web};
use anyhow::Error as AnyhowError;
use serde_json::json;

use crate::{
    domain::auth::AuthEnvironment,
    handlers::api_types::{
        AuthPresetSummary, DefinitionQuery, DefinitionResponse, EnvironmentSummary,
        LogFileResponse, ProjectQuery, ProjectSummary, ReloadResponse, RuntimeInfo, SendRequest,
        SendResponse, TreeResponse,
    },
    runtime::store::sorted_environments,
    services::{
        logging::create_manual_log_file,
        request_execution::{
            ExecuteRequestResult, REQUEST_TIMEOUT_MS, ResponseNotification,
            ResponseNotificationCode, ResponseNotificationKind, execute_request,
        },
    },
    state::app_state::AppState,
};

#[get("/api/runtime")]
pub async fn runtime(state: web::Data<AppState>) -> actix_web::Result<impl Responder> {
    let store = state.store();
    let project_list: Vec<ProjectSummary> = store
        .project_names()
        .into_iter()
        .map(|name| ProjectSummary { name })
        .collect();

    Ok(web::Json(RuntimeInfo {
        root: store.paths.root.display().to_string(),
        projects: project_list,
        backend_url: state.backend_url.clone(),
    }))
}

#[get("/api/projects")]
pub async fn projects(state: web::Data<AppState>) -> actix_web::Result<impl Responder> {
    let store = state.store();
    let projects: Vec<_> = store
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
    let store = state.store();
    let project = store
        .project(&query.project)
        .map_err(actix_web::error::ErrorBadRequest)?;
    let items = sorted_environments(&project.environments)
        .into_iter()
        .map(|(name, _)| {
            let auth_presets = match project.auth.environments.get(name) {
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
            EnvironmentSummary {
                name: name.clone(),
                auth_presets,
            }
        })
        .collect::<Vec<_>>();
    Ok(web::Json(items))
}

#[get("/api/tree")]
pub async fn tree(
    state: web::Data<AppState>,
    query: web::Query<ProjectQuery>,
) -> actix_web::Result<impl Responder> {
    let store = state.store();
    let nodes = store
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
    let store = state.store();
    let definition = store
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
    let store = state.store();
    match execute_request(store.as_ref(), payload.into_inner().into()).await {
        Ok(response) => Ok(HttpResponse::Ok().json(SendResponse::from(response))),
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
    let store = state.store();
    let file = create_manual_log_file(store.as_ref(), &payload.project)
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(web::Json(LogFileResponse {
        project: payload.project.clone(),
        current_log_file: file.display().to_string(),
    }))
}

#[post("/api/reload")]
pub async fn reload(state: web::Data<AppState>) -> actix_web::Result<impl Responder> {
    let store = state.reload().map_err(actix_web::error::ErrorBadRequest)?;

    Ok(web::Json(ReloadResponse {
        success: true,
        message: "reload completed".to_string(),
        project_count: store.project_names().len(),
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

    SendResponse::from(ExecuteRequestResult {
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
    })
}
