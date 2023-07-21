use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ProviderDetails {
    #[serde(rename = "urlPattern")]
    url_pattern: String,
    #[serde(rename = "completeProvider")]
    complete_provider: bool,
    rules: Vec<String>,
    #[serde(rename = "referralMarketing")]
    referral_marketing: Vec<String>,
    #[serde(rename = "rawRules")]
    raw_rules: Vec<String>,
    exceptions: Vec<String>,
    redirections: Vec<String>,
    #[serde(rename = "forceRedirection")]
    force_redirection: bool,
}

#[derive(Debug)]
pub struct CompiledProviderDetails {
    pub url_pattern: Regex,
    pub complete_provider: bool,
    pub rules: Vec<Regex>,
    referral_marketing: Vec<Regex>,
    raw_rules: Vec<Regex>,
    pub exceptions: Vec<Regex>,
    redirections: Vec<Regex>,
    force_redirection: bool,
}

#[derive(Debug, Deserialize)]
pub struct Providers {
    pub providers: HashMap<String, ProviderDetails>,
}

impl CompiledProviderDetails {
    pub fn new(details: &ProviderDetails) -> CompiledProviderDetails {
        CompiledProviderDetails {
            url_pattern: Regex::new(details.url_pattern.as_str()).unwrap(),
            complete_provider: details.complete_provider,
            rules: details
                .rules
                .iter()
                .map(|x| Regex::new(x.as_str()).unwrap())
                .collect(),
            referral_marketing: details
                .referral_marketing
                .iter()
                .map(|x| Regex::new(x.as_str()).unwrap())
                .collect(),
            raw_rules: details
                .raw_rules
                .iter()
                .map(|x| Regex::new(x.as_str()).unwrap())
                .collect(),
            exceptions: details
                .exceptions
                .iter()
                .map(|x| Regex::new(x.as_str()).unwrap())
                .collect(),
            redirections: details
                .redirections
                .iter()
                .map(|x| Regex::new(x.as_str()).unwrap())
                .collect(),
            force_redirection: details.force_redirection,
        }
    }
}

pub async fn compile_providers() -> Vec<CompiledProviderDetails> {
    let resp = Client::new()
        .get("https://gitlab.com/ClearURLs/rules/-/raw/master/data.min.json")
        .send()
        .await
        .unwrap()
        .text();

    let providers = serde_json::from_str::<Providers>(resp.await.unwrap().as_str()).unwrap();

    let compiled_providers: Vec<CompiledProviderDetails> = providers
        .providers
        .iter()
        .map(|provider| CompiledProviderDetails::new(provider.1))
        .collect();

    compiled_providers
}
