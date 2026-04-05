use actix_files::NamedFile;
use actix_web::{Result, error::ErrorNotFound, web};

use crate::state::app_state::AppState;

pub async fn frontend_index(state: web::Data<AppState>) -> Result<NamedFile> {
    let index = state.store.paths.frontend_dist_root.join("index.html");
    NamedFile::open_async(index).await.map_err(ErrorNotFound)
}

pub async fn frontend_asset(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<NamedFile> {
    let requested = state.store.paths.frontend_dist_root.join(path.into_inner());
    if requested.is_file() {
        return NamedFile::open_async(requested)
            .await
            .map_err(ErrorNotFound);
    }

    let fallback = state.store.paths.frontend_dist_root.join("index.html");
    NamedFile::open_async(fallback).await.map_err(ErrorNotFound)
}
