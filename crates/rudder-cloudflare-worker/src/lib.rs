use axum::{body::Body, http::Response};
use tower_service::Service as _;
use worker::{Context, Env, HttpRequest, Result, event};

mod auth;
mod routes;

#[event(fetch)]
async fn fetch(req: HttpRequest, _env: Env, _ctx: Context) -> Result<Response<Body>> {
    console_error_panic_hook::set_once();
    Ok(routes::router().call(req).await?)
}
