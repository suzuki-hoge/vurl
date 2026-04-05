use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};
use anyhow::Result;

use crate::{
    cli::Cli,
    config::paths::AppPaths,
    handlers::{api, frontend},
    runtime::store::RuntimeStore,
    state::app_state::AppState,
};

pub struct BackendApp {
    state: AppState,
}

pub fn build_app(_cli: Cli) -> Result<BackendApp> {
    let paths = AppPaths::from_default_root()?;
    let store = RuntimeStore::load(paths)?;
    let backend_url = "http://127.0.0.1:1357".to_string();

    Ok(BackendApp {
        state: AppState {
            store: Arc::clone(&store),
            backend_url,
        },
    })
}

impl BackendApp {
    pub async fn run(self) -> Result<()> {
        tracing::info!(
            host = %"127.0.0.1",
            port = 1357,
            root = %self.state.store.paths.root.display(),
            "starting vurl-backend"
        );

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(self.state.clone()))
                .wrap(
                    Cors::default()
                        .allow_any_header()
                        .allow_any_method()
                        .allowed_origin("http://127.0.0.1:3000")
                        .allowed_origin("http://localhost:3000"),
                )
                .wrap(middleware::Logger::default())
                .service(api::runtime)
                .service(api::projects)
                .service(api::environments)
                .service(api::tree)
                .service(api::definition)
                .service(api::send)
                .service(api::new_log)
                .route("/", web::get().to(frontend::frontend_index))
                .route("/{path:.*}", web::get().to(frontend::frontend_asset))
                .default_service(web::route().to(api::not_found))
        })
        .bind(("127.0.0.1", 1357))?
        .run()
        .await?;

        Ok(())
    }
}
