use core::fmt;

use common::models::Deck;
use reqwasm::{http::Request, http::Response, Error};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug)]
pub struct ApiError(String);

impl ApiError {
    pub fn new(msg: String) -> Self {
        log::error!("{}", msg);
        ApiError(msg)
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<Error> for ApiError {
    fn from(e: Error) -> ApiError {
        ApiError::new(e.to_string())
    }
}

async fn handle_request(request: Request) -> Result<Response, ApiError> {
    match request.send().await {
        Ok(response) => {
            if !response.ok() {
                // TODO not too sure about this approach.
                return Err(ApiError(response.text().await.unwrap()));
            }
            Ok(response)
        }
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn deserialize<T: DeserializeOwned>(response: Response) -> Result<T, ApiError> {
    response.json().await.map_err(ApiError::from)
}

pub async fn get<T: DeserializeOwned>(url: &str) -> Result<T, ApiError> {
    let response = handle_request(Request::get(url)).await?;
    deserialize(response).await
}

// TODO need a better name than `post_vanilla`
pub async fn post_vanilla(url: &str, payload: Value) -> Result<Response, ApiError> {
    let payload = serde_json::to_string(&payload).unwrap();
    let request = Request::post(url)
        .header("Content-Type", "application/json")
        .body(payload);

    handle_request(request).await
}

pub async fn post<T: DeserializeOwned>(url: &str, payload: Value) -> Result<T, ApiError> {
    let response = post_vanilla(url, payload).await?;
    deserialize(response).await
}

pub async fn delete(url: &str) -> Result<Response, ApiError> {
    handle_request(Request::delete(url)).await
}

// TODO should move to a `common` crate between here and `backend`.
#[derive(Deserialize)]
pub struct Page<T> {
    pub results: Vec<T>,
    pub page_count: i64,
    pub has_more: bool,
}

pub fn get_deck(deck_id: i32, callback: Box<dyn Fn(Deck)>) {
    wasm_bindgen_futures::spawn_local(async move {
        let url = format!("/api/decks/{}/", deck_id);
        if let Ok::<Deck, _>(fetched_deck) = get(&url).await {
            callback(fetched_deck);
        }
    });
}
