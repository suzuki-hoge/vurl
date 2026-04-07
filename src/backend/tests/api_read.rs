mod support;

use actix_web::{App, body::to_bytes, http::StatusCode, test, web};
use anyhow::Result;
use serde_json::Value;
use vurl_backend::handlers::api;

use self::support::{ENV, PROJECT, TestContext};

async fn read_json(resp: actix_web::dev::ServiceResponse) -> Result<Value> {
    let bytes = to_bytes(resp.into_body())
        .await
        .expect("response body should be readable");
    Ok(serde_json::from_slice(&bytes)?)
}

fn environment_yaml() -> &'static str {
    r#"
name: local
order: 2
constants:
  base_url:
    value: http://localhost:18080
variables:
  user_id:
    value: "42"
"#
}

fn ordered_environment_yaml() -> &'static str {
    r#"
name: alpha
order: 1
constants:
  base_url:
    value: http://localhost:18081
variables: {}
"#
}

fn auth_yaml() -> &'static str {
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
"#
}

fn request_yaml() -> &'static str {
    r#"
name: Get User
method: GET
path: /users/{{user_id}}
auth: true
request:
  query: []
  headers:
    - key: Accept
      value: application/json
  body:
    type: json
    text: ""
"#
}

#[actix_web::test]
async fn read_endpoints_return_runtime_project_and_definition_data() -> Result<()> {
    let ctx = TestContext::new(
        environment_yaml(),
        auth_yaml(),
        &[("users/get-user.yaml", request_yaml())],
    )?;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(ctx.app_state()))
            .service(api::runtime)
            .service(api::projects)
            .service(api::environments)
            .service(api::tree)
            .service(api::definition),
    )
    .await;

    let runtime_resp = test::call_service(
        &app,
        test::TestRequest::get().uri("/api/runtime").to_request(),
    )
    .await;
    assert_eq!(runtime_resp.status(), StatusCode::OK);
    let runtime = read_json(runtime_resp).await?;
    assert_eq!(runtime["projects"][0]["name"], PROJECT);
    assert_eq!(runtime["backend_url"], "http://127.0.0.1:1357");

    let projects_resp = test::call_service(
        &app,
        test::TestRequest::get().uri("/api/projects").to_request(),
    )
    .await;
    assert_eq!(projects_resp.status(), StatusCode::OK);
    let projects = read_json(projects_resp).await?;
    assert_eq!(projects[0]["name"], PROJECT);

    let env_resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/environments?project=project-1")
            .to_request(),
    )
    .await;
    assert_eq!(env_resp.status(), StatusCode::OK);
    let environments = read_json(env_resp).await?;
    assert_eq!(environments[0]["name"], ENV);
    assert_eq!(environments[0]["auth_presets"][0]["name"], "preset-user");

    let tree_resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/tree?project=project-1")
            .to_request(),
    )
    .await;
    assert_eq!(tree_resp.status(), StatusCode::OK);
    let tree = read_json(tree_resp).await?;
    assert_eq!(tree["project"], PROJECT);
    assert_eq!(tree["nodes"][0]["type"], "directory");

    let definition_resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/definition?project=project-1&path=users/get-user.yaml")
            .to_request(),
    )
    .await;
    assert_eq!(definition_resp.status(), StatusCode::OK);
    let definition = read_json(definition_resp).await?;
    assert_eq!(definition["path"], "users/get-user.yaml");
    assert_eq!(definition["definition"]["name"], "Get User");

    Ok(())
}

#[actix_web::test]
async fn new_log_endpoint_creates_log_file() -> Result<()> {
    let ctx = TestContext::new(
        environment_yaml(),
        auth_yaml(),
        &[("users/get-user.yaml", request_yaml())],
    )?;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(ctx.app_state()))
            .service(api::new_log),
    )
    .await;

    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/logs/new")
            .set_json(serde_json::json!({ "project": PROJECT }))
            .to_request(),
    )
    .await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json(resp).await?;
    let file = body["current_log_file"]
        .as_str()
        .expect("log file path should exist");
    assert_eq!(body["project"], PROJECT);
    assert!(std::path::Path::new(file).exists());
    Ok(())
}

#[actix_web::test]
async fn environments_endpoint_returns_items_in_order_sequence() -> Result<()> {
    let ctx = TestContext::new(
        environment_yaml(),
        auth_yaml(),
        &[("users/get-user.yaml", request_yaml())],
    )?;

    std::fs::write(
        ctx.store
            .paths
            .defs_root
            .join(PROJECT)
            .join("environments")
            .join("alpha.yaml"),
        ordered_environment_yaml(),
    )?;

    let store = vurl_backend::runtime::store::RuntimeStore::load(ctx.store.paths.clone())?;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(vurl_backend::state::app_state::AppState {
                store,
                backend_url: "http://127.0.0.1:1357".to_string(),
            }))
            .service(api::environments),
    )
    .await;

    let env_resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/environments?project=project-1")
            .to_request(),
    )
    .await;
    assert_eq!(env_resp.status(), StatusCode::OK);

    let environments = read_json(env_resp).await?;
    assert_eq!(environments[0]["name"], "alpha");
    assert_eq!(environments[1]["name"], "local");

    Ok(())
}
