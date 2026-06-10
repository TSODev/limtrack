use crate::components::ui::parse_error_response;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

fn build_headers(token: &str, with_json: bool) -> Result<web_sys::Headers, String> {
    let headers = web_sys::Headers::new().map_err(|e| format!("{:?}", e))?;
    headers
        .set("Authorization", &format!("Bearer {}", token))
        .ok();
    if with_json {
        headers.set("Content-Type", "application/json").ok();
    } else {
        headers.set("Cache-Control", "no-cache").ok();
    }
    Ok(headers)
}

fn make_req(method: &str, url: &str, token: &str) -> Result<web_sys::Request, String> {
    let mut opts = web_sys::RequestInit::new();
    opts.method(method);
    let headers = build_headers(token, false)?;
    opts.headers(headers.as_ref());
    web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{:?}", e))
}

fn make_req_body(
    method: &str,
    url: &str,
    token: &str,
    body: &serde_json::Value,
) -> Result<web_sys::Request, String> {
    let mut opts = web_sys::RequestInit::new();
    opts.method(method);
    let headers = build_headers(token, true)?;
    opts.headers(headers.as_ref());
    opts.body(Some(&wasm_bindgen::JsValue::from_str(&body.to_string())));
    web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{:?}", e))
}

async fn execute(req: web_sys::Request) -> Result<web_sys::Response, String> {
    let val = JsFuture::from(leptos::window().fetch_with_request(&req))
        .await
        .map_err(|e| format!("{:?}", e))?;
    val.dyn_into::<web_sys::Response>()
        .map_err(|e| format!("{:?}", e))
}

async fn parse_response<T: for<'de> serde::Deserialize<'de>>(
    resp: web_sys::Response,
) -> Result<T, String> {
    let json = JsFuture::from(resp.json().map_err(|e| format!("{:?}", e))?)
        .await
        .map_err(|e| format!("{:?}", e))?;
    serde_wasm_bindgen::from_value(json).map_err(|e| format!("{:?}", e))
}

// ─── GET ─────────────────────────────────────────────────────────

pub async fn api_get<T: for<'de> serde::Deserialize<'de>>(
    url: &str,
    token: &str,
) -> Result<T, String> {
    let resp = execute(make_req("GET", url, token)?).await?;
    if !resp.ok() {
        return Err(format!("Erreur HTTP : {}", resp.status()));
    }
    parse_response(resp).await
}

// ─── POST ────────────────────────────────────────────────────────

pub async fn api_post(
    url: &str,
    token: &str,
    body: &serde_json::Value,
) -> Result<(), String> {
    let resp = execute(make_req_body("POST", url, token, body)?).await?;
    if resp.ok() {
        Ok(())
    } else {
        Err(parse_error_response(resp).await)
    }
}

/// POST avec corps de réponse désérialisé (ex. : création d'un share code)
pub async fn api_post_response<T: for<'de> serde::Deserialize<'de>>(
    url: &str,
    token: &str,
    body: &serde_json::Value,
) -> Result<T, String> {
    let resp = execute(make_req_body("POST", url, token, body)?).await?;
    if !resp.ok() {
        return Err(parse_error_response(resp).await);
    }
    parse_response(resp).await
}

// ─── PUT ─────────────────────────────────────────────────────────

pub async fn api_put(
    url: &str,
    token: &str,
    body: &serde_json::Value,
) -> Result<(), String> {
    let resp = execute(make_req_body("PUT", url, token, body)?).await?;
    if resp.ok() {
        Ok(())
    } else {
        Err(parse_error_response(resp).await)
    }
}

// ─── PATCH ───────────────────────────────────────────────────────

pub async fn api_patch(
    url: &str,
    token: &str,
    body: &serde_json::Value,
) -> Result<(), String> {
    let resp = execute(make_req_body("PATCH", url, token, body)?).await?;
    if resp.ok() {
        Ok(())
    } else {
        Err(parse_error_response(resp).await)
    }
}

/// PATCH sans corps (ex. : archive / unarchive)
pub async fn api_patch_empty(url: &str, token: &str) -> Result<(), String> {
    let resp = execute(make_req("PATCH", url, token)?).await?;
    if resp.ok() {
        Ok(())
    } else {
        Err(parse_error_response(resp).await)
    }
}

// ─── DELETE ──────────────────────────────────────────────────────

pub async fn api_delete(url: &str, token: &str) -> Result<(), String> {
    let resp = execute(make_req("DELETE", url, token)?).await?;
    if resp.ok() {
        Ok(())
    } else {
        Err(parse_error_response(resp).await)
    }
}

pub async fn api_delete_body(
    url: &str,
    token: &str,
    body: &serde_json::Value,
) -> Result<(), String> {
    let resp = execute(make_req_body("DELETE", url, token, body)?).await?;
    if resp.ok() {
        Ok(())
    } else {
        Err(parse_error_response(resp).await)
    }
}
