use std::collections::LinkedList;

use http::header::USER_AGENT;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::Error;

static PRO_CLIENT: once_cell::sync::OnceCell<ProClient> = once_cell::sync::OnceCell::new();

struct ProClient {
    http_client: Client,
    user_agent: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RedirectorResponse {
    #[serde(rename = "resultUrl")]
    pub result_url: Option<String>,
    #[serde(rename = "status")]
    pub response_status: ResponseType,
    #[serde(rename = "urls")]
    pub redirect_urls: Option<LinkedList<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ResponseType {
    #[serde(rename = "BAD_REQUEST")]
    BadRequest,
    #[serde(rename = "OK")]
    Ok,
    #[serde(rename = "URL_MALFORMED")]
    UrlMalformed,
}

#[derive(Debug, Deserialize)]
pub struct CanonicalInfo {
    domain: String,
    is_alt: bool,
    pub is_amp: bool,
    is_cached: Option<bool>,
    is_valid: bool,
    #[serde(rename = "type")]
    type_: String,
    pub url: String,
    url_similarity: f64,
}

#[derive(Debug, Deserialize)]
pub struct OriginInfo {
    domain: String,
    pub is_amp: bool,
    is_cached: bool,
    is_valid: bool,
    url: String,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    pub amp_canonical: Option<CanonicalInfo>,
    pub canonical: Option<CanonicalInfo>,
    pub origin: OriginInfo,
}

pub async fn get_redirects(url: &str) -> Result<RedirectorResponse, Error> {
    let client = PRO_CLIENT.get_or_init(init_http_client);

    let link_result = urlencoding::encode(url);

    let resp = client
        .http_client
        .get(format!(
            "https://redirector.pluoi.workers.dev/{}",
            link_result
        ))
        .header(USER_AGENT, &client.user_agent)
        .send()
        .await?
        .text()
        .await?;

    Ok(serde_json::from_str::<RedirectorResponse>(resp.as_str())?)
}

pub async fn check_for_amp(url: &str) -> Result<Vec<Item>, Error> {
    let client = PRO_CLIENT.get_or_init(init_http_client);

    let resp = client
        .http_client
        .get(format!(
            "https://www.amputatorbot.com/api/v1/convert?gac=true&md=3&q={}",
            url
        ))
        .header(USER_AGENT, &client.user_agent)
        .send()
        .await?
        .text()
        .await?;

    println!(
        "{:?}",
        format!(
            "https://www.amputatorbot.com/api/v1/convert?gac=true&md=3&q={}",
            url
        )
    );
    println!("{:?}", resp.as_str());

    Ok(serde_json::from_str::<Vec<Item>>(resp.as_str())?)
}

fn init_http_client() -> ProClient {
    let client = Client::new();

    ProClient {
        http_client: client,
        user_agent: "Linkers URL Cleaner Bot".to_string(),
    }
}
