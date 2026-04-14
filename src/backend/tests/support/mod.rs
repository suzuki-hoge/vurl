#![allow(dead_code)]

use std::{
    collections::VecDeque,
    fs,
    net::TcpListener,
    sync::{Arc, Mutex},
};

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, dev::ServerHandle, web};
use anyhow::Result;
use serde_json::json;
use tempfile::TempDir;
use vurl_backend::{
    config::paths::AppPaths, runtime::store::RuntimeStore, state::app_state::AppState,
};

pub const PROJECT: &str = "project-1";
pub const ENV: &str = "local";

pub struct TestContext {
    _tmp: TempDir,
    pub store: Arc<RuntimeStore>,
}

impl TestContext {
    pub fn new(
        environment_yaml: &str,
        auth_yaml: &str,
        request_files: &[(&str, &str)],
    ) -> Result<Self> {
        let tmp = tempfile::tempdir()?;
        let root = tmp.path().to_path_buf();
        let requests_dir = root.join("defs").join(PROJECT).join("requests");
        let env_dir = root.join("defs").join(PROJECT).join("environments");
        fs::create_dir_all(&requests_dir)?;
        fs::create_dir_all(&env_dir)?;

        fs::write(env_dir.join(format!("{ENV}.yaml")), environment_yaml)?;
        fs::write(env_dir.join("auth.yaml"), auth_yaml)?;

        for (path, body) in request_files {
            let full = requests_dir.join(path);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(full, body)?;
        }

        let paths = AppPaths::new(&root)?;
        let store = RuntimeStore::load(paths)?;

        Ok(Self { _tmp: tmp, store })
    }

    #[allow(dead_code)]
    pub fn app_state(&self) -> AppState {
        AppState::new(Arc::clone(&self.store), "http://127.0.0.1:1357".to_string())
    }
}

#[derive(Clone, Debug)]
pub struct CapturedRequest {
    #[allow(dead_code)]
    pub method: String,
    pub path: String,
    pub query: String,
    #[allow(dead_code)]
    pub headers: Vec<(String, String)>,
    pub body: String,
}

#[derive(Clone)]
pub struct TestServer {
    pub base_url: String,
    pub state: Arc<ServerState>,
    handle: ServerHandle,
}

impl TestServer {
    pub async fn stop(self) {
        self.handle.stop(true).await;
    }
}

#[derive(Default)]
pub struct ServerState {
    auth_requests: Mutex<Vec<CapturedRequest>>,
    send_requests: Mutex<Vec<CapturedRequest>>,
    auth_script: Mutex<ResponseScript>,
    send_script: Mutex<ResponseScript>,
}

impl ServerState {
    #[allow(dead_code)]
    pub fn auth_requests(&self) -> Vec<CapturedRequest> {
        self.auth_requests
            .lock()
            .expect("auth_requests poisoned")
            .clone()
    }

    #[allow(dead_code)]
    pub fn send_requests(&self) -> Vec<CapturedRequest> {
        self.send_requests
            .lock()
            .expect("send_requests poisoned")
            .clone()
    }

    #[allow(dead_code)]
    pub fn enqueue_auth_response(&self, response: ScriptedResponse) {
        self.auth_script
            .lock()
            .expect("auth_script poisoned")
            .responses
            .push_back(response);
    }

    pub fn enqueue_send_response(&self, response: ScriptedResponse) {
        self.send_script
            .lock()
            .expect("send_script poisoned")
            .responses
            .push_back(response);
    }
}

#[derive(Default)]
struct ResponseScript {
    responses: VecDeque<ScriptedResponse>,
}

#[derive(Clone)]
pub struct ScriptedResponse {
    pub status: u16,
    pub body: String,
    pub content_type: Option<String>,
    pub delay_ms: u64,
}

impl ScriptedResponse {
    pub fn json(status: u16, body: serde_json::Value) -> Self {
        Self {
            status,
            body: body.to_string(),
            content_type: Some("application/json".to_string()),
            delay_ms: 0,
        }
    }

    #[allow(dead_code)]
    pub fn text(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            body: body.into(),
            content_type: Some("text/plain".to_string()),
            delay_ms: 0,
        }
    }

    pub fn delayed_text(status: u16, body: impl Into<String>, delay_ms: u64) -> Self {
        Self {
            status,
            body: body.into(),
            content_type: Some("text/plain".to_string()),
            delay_ms,
        }
    }
}

pub async fn spawn_test_server() -> Result<TestServer> {
    let state = Arc::new(ServerState::default());
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let port = listener.local_addr()?.port();
    let state_for_server = Arc::clone(&state);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&state_for_server)))
            .default_service(web::route().to(handle_request))
    })
    .listen(listener)?
    .run();
    let handle = server.handle();
    actix_web::rt::spawn(server);

    Ok(TestServer {
        base_url: format!("http://127.0.0.1:{port}"),
        state,
        handle,
    })
}

async fn handle_request(
    req: HttpRequest,
    body: web::Bytes,
    state: web::Data<Arc<ServerState>>,
) -> HttpResponse {
    let captured = CapturedRequest {
        method: req.method().to_string(),
        path: req.path().to_string(),
        query: req.query_string().to_string(),
        headers: req
            .headers()
            .iter()
            .map(|(key, value)| {
                (
                    key.to_string(),
                    value.to_str().unwrap_or_default().to_string(),
                )
            })
            .collect(),
        body: String::from_utf8_lossy(&body).into_owned(),
    };

    let script = if req.path().starts_with("/auth") {
        state
            .auth_requests
            .lock()
            .expect("auth_requests poisoned")
            .push(captured.clone());
        &state.auth_script
    } else {
        state
            .send_requests
            .lock()
            .expect("send_requests poisoned")
            .push(captured.clone());
        &state.send_script
    };

    let response = script
        .lock()
        .expect("script poisoned")
        .responses
        .pop_front()
        .unwrap_or_else(|| {
            ScriptedResponse::json(
                200,
                json!({
                    "path": captured.path,
                    "query": captured.query,
                    "body": captured.body
                }),
            )
        });

    if response.delay_ms > 0 {
        actix_web::rt::time::sleep(std::time::Duration::from_millis(response.delay_ms)).await;
    }

    let mut builder = actix_web::HttpResponse::build(
        actix_web::http::StatusCode::from_u16(response.status)
            .expect("status code should be valid"),
    );
    if let Some(content_type) = response.content_type {
        builder.content_type(content_type);
    }
    builder.body(response.body)
}
