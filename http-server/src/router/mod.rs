mod by_path;

pub(crate) fn router() -> axum::Router {
    axum::Router::new()
        .route("/by-path/", axum::routing::get(by_path::index_handler))
        .route("/by-path/*path", axum::routing::get(by_path::any_handler))
}
