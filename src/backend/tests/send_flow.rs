mod support;

use anyhow::Result;
use serde_json::json;
use vurl_backend::{
    domain::http::{AuthCredentials, AuthInputMode, KeyValueEntry, RequestBodyDraft},
    services::request_execution::{
        ExecuteRequestInput, REQUEST_TIMEOUT_MS, RequestAuth, execute_request,
    },
};

use self::support::{ENV, PROJECT, ScriptedResponse, TestContext, spawn_test_server};

fn environment_yaml(base_url: &str, auth_token: &str) -> String {
    format!(
        r#"
name: {ENV}
constants:
  base_url:
    value: {base_url}
  tenant_id:
    value: tenant-a
variables:
  auth_token:
    value: "{auth_token}"
    mask: xxx
  user_id:
    value: "42"
"#
    )
}

fn fixed_auth_yaml() -> &'static str {
    r#"
environments:
  local:
    mode: fixed
    credentials:
      presets:
        - name: preset-user
          id: known-user
    mappings:
      items:
        - id: known-user
          variables:
            auth_token: fixed-token
      default:
        variables:
          auth_token: default-token
"#
}

fn http_auth_yaml(base_url: &str) -> String {
    format!(
        r#"
environments:
  local:
    mode: http
    credentials:
      presets:
        - name: qa-user
          id: qa-user
          password: qa-password
    request:
      method: POST
      url: "{base_url}/auth/login"
      headers:
        - key: Content-Type
          value: application/x-www-form-urlencoded
      body:
        type: form
        form:
          - key: id
            value: "{{{{auth.id}}}}"
          - key: password
            value: "{{{{auth.password}}}}"
          - key: tenant
            value: "{{{{tenant_id}}}}"
    response:
      inject:
        - from: $.token
          to: auth_token
"#
    )
}

fn default_request() -> ExecuteRequestInput {
    ExecuteRequestInput {
        project: PROJECT.to_string(),
        environment: ENV.to_string(),
        path: "users/get-user.yaml".to_string(),
        method: "GET".to_string(),
        url_path: "/users/{{user_id}}".to_string(),
        query: vec![KeyValueEntry {
            key: "verbose".to_string(),
            value: "true".to_string(),
        }],
        headers: vec![KeyValueEntry {
            key: "Authorization".to_string(),
            value: "Bearer {{auth_token}}".to_string(),
        }],
        body: RequestBodyDraft::Json {
            text: String::new(),
        },
        auth: RequestAuth {
            enabled: true,
            input_mode: AuthInputMode::Preset,
            preset_name: Some("preset-user".to_string()),
            credentials: AuthCredentials::default(),
        },
    }
}

#[actix_web::test]
async fn fixed_auth_updates_env_before_sending_request() -> Result<()> {
    let server = spawn_test_server().await?;
    server
        .state
        .enqueue_send_response(ScriptedResponse::json(200, json!({ "ok": true })));

    let ctx = TestContext::new(
        &environment_yaml(&server.base_url, ""),
        fixed_auth_yaml(),
        &[],
    )?;
    let response = execute_request(&ctx.store, default_request()).await?;

    assert_eq!(response.status, 200);
    assert!(!response.retried_auth);
    assert!(response.notifications.is_empty());

    let sent = server.state.send_requests();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].path, "/users/42");
    assert_eq!(sent[0].query, "verbose=true");
    assert!(
        sent[0]
            .headers
            .iter()
            .any(|(k, v)| k == "authorization" && v == "Bearer fixed-token")
    );

    let env_state = ctx.store.env_state(PROJECT, ENV)?;
    assert_eq!(
        env_state.variables.get("auth_token").map(String::as_str),
        Some("fixed-token")
    );

    server.stop().await;
    Ok(())
}

#[actix_web::test]
async fn fixed_auth_uses_default_mapping_when_id_does_not_match() -> Result<()> {
    let server = spawn_test_server().await?;
    let ctx = TestContext::new(
        &environment_yaml(&server.base_url, ""),
        fixed_auth_yaml(),
        &[],
    )?;

    let mut request = default_request();
    request.auth.credentials = AuthCredentials {
        id: "unknown-user".to_string(),
        password: String::new(),
    };
    request.auth.input_mode = AuthInputMode::Manual;
    request.auth.preset_name = None;

    let response = execute_request(&ctx.store, request).await?;
    assert_eq!(response.status, 200);

    let sent = server.state.send_requests();
    assert_eq!(sent.len(), 1);
    assert!(
        sent[0]
            .headers
            .iter()
            .any(|(k, v)| k == "authorization" && v == "Bearer default-token")
    );

    server.stop().await;
    Ok(())
}

#[actix_web::test]
async fn http_auth_authenticates_and_injects_token() -> Result<()> {
    let server = spawn_test_server().await?;
    server.state.enqueue_auth_response(ScriptedResponse::json(
        200,
        json!({ "token": "http-token-1" }),
    ));
    server
        .state
        .enqueue_send_response(ScriptedResponse::json(200, json!({ "ok": true })));

    let ctx = TestContext::new(
        &environment_yaml(&server.base_url, ""),
        &http_auth_yaml(&server.base_url),
        &[],
    )?;

    let mut request = default_request();
    request.auth.preset_name = Some("qa-user".to_string());

    let response = execute_request(&ctx.store, request).await?;
    assert_eq!(response.status, 200);
    assert_eq!(response.notifications.len(), 1);

    let auth_calls = server.state.auth_requests();
    assert_eq!(auth_calls.len(), 1);
    assert_eq!(auth_calls[0].path, "/auth/login");
    assert_eq!(
        auth_calls[0].body,
        "id=qa-user&password=qa-password&tenant=tenant-a"
    );

    let send_calls = server.state.send_requests();
    assert_eq!(send_calls.len(), 1);
    assert!(
        send_calls[0]
            .headers
            .iter()
            .any(|(k, v)| k == "authorization" && v == "Bearer http-token-1")
    );

    server.stop().await;
    Ok(())
}

#[actix_web::test]
async fn retries_auth_once_after_401_and_resends_request() -> Result<()> {
    let server = spawn_test_server().await?;
    server
        .state
        .enqueue_send_response(ScriptedResponse::text(401, "expired"));
    server.state.enqueue_auth_response(ScriptedResponse::json(
        200,
        json!({ "token": "retried-token" }),
    ));
    server
        .state
        .enqueue_send_response(ScriptedResponse::json(200, json!({ "ok": true })));

    let ctx = TestContext::new(
        &environment_yaml(&server.base_url, "stale-token"),
        &http_auth_yaml(&server.base_url),
        &[],
    )?;

    let mut request = default_request();
    request.auth.preset_name = Some("qa-user".to_string());

    let response = execute_request(&ctx.store, request).await?;
    assert_eq!(response.status, 200);
    assert!(response.retried_auth);
    assert_eq!(response.notifications.len(), 1);

    let auth_calls = server.state.auth_requests();
    assert_eq!(auth_calls.len(), 1);

    let send_calls = server.state.send_requests();
    assert_eq!(send_calls.len(), 2);
    assert!(
        send_calls[0]
            .headers
            .iter()
            .any(|(k, v)| k == "authorization" && v == "Bearer stale-token")
    );
    assert!(
        send_calls[1]
            .headers
            .iter()
            .any(|(k, v)| k == "authorization" && v == "Bearer retried-token")
    );

    server.stop().await;
    Ok(())
}

#[actix_web::test]
async fn timeout_returns_error_from_execute_request() -> Result<()> {
    let server = spawn_test_server().await?;
    server
        .state
        .enqueue_send_response(ScriptedResponse::delayed_text(
            200,
            "slow",
            REQUEST_TIMEOUT_MS + 200,
        ));

    let ctx = TestContext::new(
        &environment_yaml(&server.base_url, ""),
        fixed_auth_yaml(),
        &[],
    )?;

    let error = execute_request(&ctx.store, default_request())
        .await
        .expect_err("request should timeout");
    assert!(
        error
            .chain()
            .filter_map(|cause| cause.downcast_ref::<reqwest::Error>())
            .any(reqwest::Error::is_timeout)
    );

    server.stop().await;
    Ok(())
}

#[actix_web::test]
async fn request_log_masks_token_value() -> Result<()> {
    let server = spawn_test_server().await?;
    server.state.enqueue_send_response(ScriptedResponse::text(
        200,
        "response auth_token=fixed-token",
    ));

    let ctx = TestContext::new(
        &environment_yaml(&server.base_url, ""),
        fixed_auth_yaml(),
        &[],
    )?;

    let response = execute_request(&ctx.store, default_request()).await?;
    let log_text = std::fs::read_to_string(&response.current_log_file)?;
    assert!(log_text.contains("Bearer xxx"));
    assert!(log_text.contains("auth_token=xxx"));
    assert!(!log_text.contains("fixed-token"));

    server.stop().await;
    Ok(())
}
