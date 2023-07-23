use std::env;

use http::header::{CONTENT_TYPE, COOKIE, USER_AGENT};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::Error;

static PRO_CLIENT: once_cell::sync::OnceCell<ProClient> = once_cell::sync::OnceCell::new();

struct ProClient {
    http_client: Client,
    cookies: String,
    nonce: String,
    user_agent: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    #[serde(rename = "type")]
    pub message_type: String,
    pub id: i32,
    #[serde(rename = "itemId")]
    pub item_id: Option<i32>,
    created: i64,
    pub message: String,
    pub read: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MessageCollection {
    pub messages: Vec<Message>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Comment {
    pub id: i32,
    pub parent: i32,
    pub content: String,
    created: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Post {
    pub comments: Vec<Comment>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Inbox {
    pub mentions: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sync {
    pub inbox: Inbox,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PostComment {
    pub comment: String,
    #[serde(rename = "parentId")]
    pub parent_id: i32,
    #[serde(rename = "itemId")]
    pub item_id: i32,
    #[serde(rename = "_nonce")]
    pub nonce: String,
}

pub async fn get_latest_messages() -> Result<MessageCollection, Error> {
    let client = PRO_CLIENT.get_or_init(init_pro_client);

    let resp = client
        .http_client
        .get("https://pr0gramm.com/api/inbox/all")
        .header(COOKIE, &client.cookies)
        .header(USER_AGENT, &client.user_agent)
        .send()
        .await?
        .text()
        .await?;

    Ok(serde_json::from_str::<MessageCollection>(resp.as_str())?)
}

pub async fn get_post(item_id: i32) -> Result<Post, Error> {
    let client = PRO_CLIENT.get_or_init(init_pro_client);

    let resp = client
        .http_client
        .get(format!(
            "https://pr0gramm.com/api/items/info?itemId={}",
            item_id
        ))
        .header(COOKIE, &client.cookies)
        .header(USER_AGENT, &client.user_agent)
        .send()
        .await?
        .text()
        .await?;

    Ok(serde_json::from_str::<Post>(resp.as_str())?)
}

pub async fn reply_comment(item_id: i32, parent_comment: i32, message: String) {
    let client = PRO_CLIENT.get_or_init(init_pro_client);

    let comment = serde_urlencoded::to_string(PostComment {
        comment: message,
        parent_id: parent_comment,
        item_id,
        nonce: client.nonce.to_string(),
    });

    let Ok(comment_text) = comment else {
        println!("Unable to encode comment: Error: {:?}", comment);
        return;
    };

    let response = client
        .http_client
        .post("https://pr0gramm.com/api/comments/post")
        .header(COOKIE, &client.cookies)
        .header(USER_AGENT, &client.user_agent)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(comment_text)
        .send()
        .await;

    match response {
        Ok(res) => {
            println!(
                "Posted comment on post {} with status code: {:?}",
                item_id,
                res.status().as_u16()
            )
        }
        Err(res) => {
            println!(
                "Error while posting comment on post {} with error: {}",
                item_id, res
            )
        }
    }
}

pub async fn has_unread_messages() -> Result<bool, Error> {
    let client = PRO_CLIENT.get_or_init(init_pro_client);

    let resp = client
        .http_client
        .get("https://pr0gramm.com/api/user/sync?offset=9999999")
        .header(COOKIE, &client.cookies)
        .header(USER_AGENT, &client.user_agent)
        .send()
        .await?
        .text()
        .await?;

    println!("Sync Response: {}", resp.as_str());

    Ok(serde_json::from_str::<Sync>(resp.as_str())?.inbox.mentions > 0)
}

fn init_pro_client() -> ProClient {
    let client = Client::new();

    ProClient {
        http_client: client,
        cookies: env::var("LINKERS_COOKIES")
            .expect("Cookies not set. Exiting as the bot won't be able to run."),
        nonce: env::var("LINKERS_NONCE")
            .expect("Nonce not set. Exiting as the bot won't be able to run."),
        user_agent: "Linkers Nutzer-Bot".to_string(),
    }
}
