use axum_browser_adapter::{
    wasm_request_to_axum_request,
    axum_response_to_wasm_response,
    wasm_compat,
    WasmResponse,
};
use axum::Router;
use axum::routing::get;
use wasm_bindgen::prelude::wasm_bindgen;
use tower_service::Service;

pub use axum_browser_adapter::WasmRequest;

#[wasm_compat]
pub async fn index() -> &'static str {
    r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <title>Axum Browser Adapter</title>
    </head>
    <body>
    Rendered by Axum compiled to WASM
    </body>
    </html>
    "#
}

#[wasm_bindgen]
pub async fn wasm_app(wasm_request: WasmRequest) -> WasmResponse {
    let mut router: Router = Router::new().route("/", get(index));

    let request = wasm_request_to_axum_request(&wasm_request).unwrap();

    let axum_response = router.call(request).await.unwrap();

    let response = axum_response_to_wasm_response(axum_response).await.unwrap();

    response
}