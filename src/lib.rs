//! Axum Browser Adapter
//!
//! A collection of tools to make integrating Axum with the browser easier
//!
//! ## Example
//!
//! ```rust
//! use axum_browser_adapter::{
//!     wasm_request_to_axum_request,
//!     axum_response_to_wasm_response,
//!     wasm_compat,
//!     WasmRequest,
//!     WasmResponse
//! };
//! use axum::Router;
//! use axum::routing::get;
//! use wasm_bindgen::prelude::wasm_bindgen;
//! use tower_service::Service;
//!
//! #[wasm_compat]
//! pub async fn index() -> &'static str {
//!     "Hello World"
//! }
//!
//! #[wasm_bindgen]
//! pub async fn wasm_app(wasm_request: WasmRequest) -> WasmResponse {
//!    let mut router: Router = Router::new().route("/", get(index));
//!
//!    let request = wasm_request_to_axum_request(&wasm_request).unwrap();
//!
//!    let axum_response = router.call(request).await.unwrap();
//!
//!    let response = axum_response_to_wasm_response(axum_response).await.unwrap();
//!
//!    response
//! }
//!```
//! Integrating w/ the browser
//!
//! ```html
//! <!DOCTYPE html>
//! <html lang="en">
//! <head>
//!     <meta charset="UTF-8">
//!     <title></title>
//! </head>
//! <body>
//! <script type="module">
//!     import init, {wasm_app, WasmRequest} from './dist/example.js';
//!
//!     (async function () {
//!         await init();
//!
//!         const wasmRequest = new WasmRequest("GET", "/", {}, undefined);
//!         let response = await wasm_app(wasmRequest);
//!
//!         document.write(response.body)
//!     }())
//! </script>
//! </body>
//! </html>
//! ```
//!
//! ## Recipes
//! You might want to override fetch or use a service worker to intercept HTTP calls in order to the
//! call the Axum WASM app instead of the a HTTP server.
//!
//! **Converting a JavaScript Request to a WasmRequest**
//! ```javascript
//!  async function requestToWasmRequest(request) {
//!     const method = request.method;
//!     const url = request.url;
//!     const headers = Object.fromEntries(request.headers.entries());
//!
//!     let body = null;
//!     if (request.body !== null) {
//!         body = await request.text();
//!     }
//!     return new WasmRequest(method, url, headers, body);
//! }
//! ```
//!
//! **Converting a WasmResponse to a JavaScript Response**
//!
//! ```javascript
//! function wasmResponseToJsResponse(wasmResponse) {
//!    const body = wasmResponse.body;
//!    const status = parseInt(wasmResponse.status_code);
//!    const jsHeaders = new Headers();
//!    const headers = wasmResponse.headers;
//!    for (let [key, value] of headers) {
//!        jsHeaders.append(key, value);
//!    }
//!    return new Response(body, {status: status, headers: jsHeaders});
//! }
//! ```

use std::collections::HashMap;
use std::str::FromStr;
use axum::body::Body;
use axum::http;
use axum::response::Response;
use axum::http::{Method, Request, Uri};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

pub use axum_wasm_macros::wasm_compat;

#[wasm_bindgen]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WasmRequest {
    #[wasm_bindgen(skip)]
    pub method: String,
    #[wasm_bindgen(skip)]
    pub url: String,
    #[wasm_bindgen(skip)]
    pub headers: HashMap<String, String>,
    #[wasm_bindgen(skip)]
    pub body: Option<String>,
}

#[wasm_bindgen]
impl WasmRequest {
    #[wasm_bindgen(constructor)]
    pub fn new(method: String, url: String, headers_js_value: JsValue, body: Option<String>) -> WasmRequest {
        let headers: HashMap<String, String> = from_value(headers_js_value).unwrap();

        WasmRequest { method, url, headers, body }
    }
}

pub fn wasm_request_to_axum_request(wasm_request: &WasmRequest) -> Result<Request<Body>, Box<dyn std::error::Error>> {
    let method = Method::from_str(&wasm_request.method)?;

    let uri = Uri::try_from(&wasm_request.url)?;

    let mut request_builder = Request::builder()
        .method(method)
        .uri(uri);

    for (k, v) in &wasm_request.headers {
        let header_name = http::header::HeaderName::from_bytes(k.as_bytes())?;
        let header_value = http::header::HeaderValue::from_str(v)?;
        request_builder = request_builder.header(header_name, header_value);
    }

    let request = match &wasm_request.body {
        Some(body_str) => request_builder.body(Body::from(body_str.to_owned()))?,
        None => request_builder.body(Body::empty())?,
    };

    Ok(request)
}

#[wasm_bindgen]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WasmResponse {
    #[wasm_bindgen(skip)]
    pub status_code: String,
    #[wasm_bindgen(skip)]
    pub headers: HashMap<String, String>,
    #[wasm_bindgen(skip)]
    pub body: Option<String>,
}

#[wasm_bindgen]
impl WasmResponse {
    #[wasm_bindgen(getter)]
    pub fn status_code(&self) -> String {
        self.status_code.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn body(&self) -> Option<String> {
        self.body.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn headers(&self) -> JsValue {
        to_value(&self.headers).unwrap()
    }
}

pub async fn axum_response_to_wasm_response(mut response: Response) -> Result<WasmResponse, Box<dyn std::error::Error>> {
    let status_code = response.status().to_string();

    let mut headers = HashMap::new();
    for (name, value) in response.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.insert(name.as_str().to_owned(), value_str.to_owned());
        }
    }

    let bytes = match http_body::Body::data(response.body_mut()).await {
        None => vec![],
        Some(body_bytes) => match body_bytes {
            Ok(bytes) => bytes.to_vec(),
            Err(_) => vec![]
        },
    };
    let body_str = String::from_utf8(bytes)?;

    Ok(WasmResponse {
        status_code,
        headers,
        body: Some(body_str),
    })
}