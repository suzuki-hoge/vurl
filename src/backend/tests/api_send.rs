mod support;

use actix_web::{App, body::to_bytes, http::StatusCode, test, web};
use anyhow::Result;
use serde_json::{Value, json};
use vurl_backend::handlers::api;

use self::support::{ENV, PROJECT, ScriptedResponse, TestContext, spawn_test_server};

fn environment_yaml(base_url: &str) -> String {
    format!(
        r#"
name: {ENV}
constants:
  base_url:
    value: {base_url}
variables:
  auth_token:
    value: ""
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

fn payload() -> Value {
    json!({
        "project": PROJECT,
        "environment": ENV,
        "path": "users/get-user.yaml",
        "method": "GET",
        "url_path": "/users/{{user_id}}",
        "query": [],
        "headers": [
            {
                "key": "Authorization",
                "value": "Bearer {{auth_token}}"
            }
        ],
        "body": {
            "type": "json",
            "text": ""
        },
        "auth_enabled": true,
        "auth_input_mode": "preset",
        "auth_preset_name": "preset-user",
        "auth_credentials": {
            "id": "",
            "password": ""
        }
    })
}

#[actix_web::test]
async fn send_handler_returns_json_response() -> Result<()> {
    let server = spawn_test_server().await?;
    server
        .state
        .enqueue_send_response(ScriptedResponse::json(200, json!({ "ok": true })));
    let ctx = TestContext::new(&environment_yaml(&server.base_url), fixed_auth_yaml(), &[])?;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(ctx.app_state()))
            .service(api::send),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/send")
        .set_json(payload())
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = to_bytes(resp.into_body())
        .await
        .expect("response body should be readable");
    let body: Value = serde_json::from_slice(&body_bytes)?;
    assert_eq!(body["status"], 200);
    assert_eq!(body["retried_auth"], false);
    assert!(body["body"].as_str().is_some());

    server.stop().await;
    Ok(())
}

#[actix_web::test]
async fn send_handler_converts_timeout_to_internal_error_response() -> Result<()> {
    let server = spawn_test_server().await?;
    server
        .state
        .enqueue_send_response(ScriptedResponse::delayed_text(
            200,
            "slow",
            vurl_backend::services::request_execution::REQUEST_TIMEOUT_MS + 200,
        ));
    let ctx = TestContext::new(&environment_yaml(&server.base_url), fixed_auth_yaml(), &[])?;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(ctx.app_state()))
            .service(api::send),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/send")
        .set_json(payload())
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body_bytes = to_bytes(resp.into_body())
        .await
        .expect("response body should be readable");
    let body: Value = serde_json::from_slice(&body_bytes)?;
    assert_eq!(body["status"], 500);
    assert_eq!(body["notifications"][0]["code"], "timeout");

    server.stop().await;
    Ok(())
}
