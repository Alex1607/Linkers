use http::header::{COOKIE, USER_AGENT};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

static PRO_CLIENT: once_cell::sync::OnceCell<ProClient> = once_cell::sync::OnceCell::new();

struct ProClient {
    http_client: Client,
    cookies: String,
    user_agent: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    #[serde(rename = "type")]
    message_type: String,
    pub id: i32,
    #[serde(rename = "itemId")]
    pub item_id: i32,
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
    pub comments: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sync {
    pub inbox: Inbox,
}

pub async fn get_latest_messages() -> MessageCollection {
    let client = PRO_CLIENT.get_or_init(init_pro_client);

    let resp = client
        .http_client
        .get("https://pr0gramm.com/api/inbox/comments")
        .header(COOKIE, &client.cookies)
        .header(USER_AGENT, &client.user_agent)
        .send()
        .await
        .unwrap()
        .text()
        .await;

    serde_json::from_str::<MessageCollection>(resp.unwrap().as_str()).unwrap()
}

pub async fn get_post(item_id: i32) -> Post {
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
        .await
        .unwrap()
        .text()
        .await;

    serde_json::from_str::<Post>(resp.unwrap().as_str()).unwrap()
}

pub async fn reply_comment(item_id: i32, parent_comment: i32, message: String) {
    let client = PRO_CLIENT.get_or_init(init_pro_client);

    let status_code = client
        .http_client
        .post("https://pr0gramm.com/api/comments/post")
        .header(COOKIE, &client.cookies)
        .header(USER_AGENT, &client.user_agent)
        .body(format!(
            "{{\"comment\": \"{comment}\", \"itemId\": {item_id}, \"parentId\": {parent_id}}}",
            comment = message,
            item_id = item_id,
            parent_id = parent_comment
        ))
        .send()
        .await
        .unwrap()
        .status();

    println!(
        "Posted comment on post {} with status code: {}",
        item_id,
        status_code.as_u16()
    )
}

pub async fn has_unread_messages() -> bool {
    let client = PRO_CLIENT.get_or_init(init_pro_client);

    let resp = client
        .http_client
        .get("https://pr0gramm.com/api/user/sync?offset=9999999")
        .header(COOKIE, &client.cookies)
        .header(USER_AGENT, &client.user_agent)
        .send()
        .await
        .unwrap()
        .text()
        .await;

    serde_json::from_str::<Sync>(resp.unwrap().as_str())
        .unwrap()
        .inbox
        .comments
        > 0
}

fn init_pro_client() -> ProClient {
    let client = Client::new();

    ProClient {
        http_client: client,
        cookies: env::var("LINKERS_COOKIES")
            .expect("Cookies not set. Exiting as the bot won't be able to run."),
        user_agent: "Linkers Nutzer-Bot".to_string(),
    }
}
