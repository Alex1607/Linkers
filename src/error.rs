use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Https Request was not successful. Http Error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Serde wasn't able to decode the response. Serde Error: {0}")]
    Json(#[from] serde_json::Error),
}
