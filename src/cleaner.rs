use std::time::Duration;

use regex::{Regex, RegexBuilder};
use url::Url;

use crate::error::Error;
use crate::pro_api::{get_latest_messages, get_post, has_unread_messages, reply_comment, Message};
use crate::providers::{compile_providers, CompiledProviderDetails};

static PROVIDER: async_once_cell::OnceCell<Vec<CompiledProviderDetails>> =
    async_once_cell::OnceCell::new();
static URL_REGEX: once_cell::sync::OnceCell<Regex> = once_cell::sync::OnceCell::new();
static CLIENT_REGEX: once_cell::sync::OnceCell<Regex> = once_cell::sync::OnceCell::new();

pub async fn run_linkers() -> Result<(), Error> {
    let providers = PROVIDER.get_or_init(compile_providers()).await;
    let bot_name_regex = CLIENT_REGEX.get_or_init(|| {
        RegexBuilder::new(r"(@linkers)")
            .case_insensitive(true)
            .build()
            .expect("Cannot build bot name regex.")
    });

    if !has_unread_messages().await? {
        return Ok(());
    }

    let message_collection = get_latest_messages().await?;

    let new_comments: Vec<&Message> = message_collection
        .messages
        .iter()
        .filter(|x1| x1.read == 0)
        .filter(|x1| x1.message_type.eq("comment"))
        .filter(|x| bot_name_regex.is_match(x.message.as_str()))
        .filter(|x| x.item_id.is_some())
        .collect();

    println!("{:?}", new_comments);

    for tag_comment in new_comments {
        let Some(item_id) = tag_comment.item_id else {
            continue;
        };

        tokio::time::sleep(Duration::from_secs(1)).await; //Prevent spamming the API in one go and give it some breathing room

        let post = get_post(item_id).await?;
        let optional_post_comment = post
            .comments
            .iter()
            .find(|comment| comment.id == tag_comment.id);

        println!("post Comment: {:?}", optional_post_comment);

        let Some(post_comment) = optional_post_comment else {
            continue;
        };

        if post_comment.parent == 0 {
            continue;
        }

        let optional_parent_comment = post
            .comments
            .iter()
            .find(|comment| comment.id == post_comment.parent);

        println!("parent Comment: {:?}", optional_parent_comment);

        let Some(parent_comment) = optional_parent_comment else {
            continue;
        };

        let links = cleanup_comment(&parent_comment.content, providers).await;

        println!("answer {:?}", links);

        reply_comment(item_id, post_comment.id, build_response_text(links)).await;
    }

    Ok(())
}

async fn cleanup_comment(input: &str, providers: &[CompiledProviderDetails]) -> Vec<String> {
    let mut output = Vec::new();
    let urls_regex = URL_REGEX.get_or_init(|| {
        Regex::new(r"(https?://\S+)").expect("Cannot build url regex. Bot won't work.")
    });

    for url in urls_regex.find_iter(input) {
        let Some(cleaner_url) = clean_url(url.as_str(), providers) else {
            continue;
        };

        if cleaner_url.len() >= url.as_str().len() {
            continue;
        }

        output.push(cleaner_url);
    }

    output
}

fn clean_url(url: &str, rules: &[CompiledProviderDetails]) -> Option<String> {
    let Ok(mut parsed_url) = Url::parse(url) else {
        return None;
    };

    let provider_list: Vec<&CompiledProviderDetails> = rules
        .iter()
        .filter(|details| details.url_pattern.is_match(url))
        .collect();

    if provider_list.is_empty() {
        return None;
    };

    for details in provider_list {
        if details
            .exceptions
            .iter()
            .any(|exception_regex| exception_regex.is_match(url))
        {
            return None;
        }

        let pairs: Vec<(String, String)> = parsed_url
            .query_pairs()
            .into_owned()
            .filter(|(key, _)| {
                !details
                    .rules
                    .iter()
                    .any(|rule_regex| rule_regex.is_match(key))
            })
            .collect();

        parsed_url.query_pairs_mut().clear().extend_pairs(pairs);
    }

    Some(parsed_url.to_string().trim_end_matches('?').to_string())
}

fn build_response_text(links: Vec<String>) -> String {
    if links.is_empty() {
        "Es wurden keine Links mit Tracking gefunden.".to_string()
    } else {
        let mut answer = if links.len() == 1 {
            "Hier der Link ohne Tracking:\n".to_string()
        } else {
            "Hier die Links ohne Tracking:\n".to_string()
        };

        links
            .iter()
            .for_each(|link| answer += format!("- {}\n", link).as_str());

        answer
    }
}

#[tokio::test]
async fn test() {
    let providers = PROVIDER.get_or_init(compile_providers()).await;

    let option_with_and_without_tracking = cleanup_comment("test1 https://enteentelos.de foo https://www.phoronix.com/scan.php?page=news_item&px=Ioquake3-Auto-Updater&utm_source=feedburner&utm_medium=feed&utm_campaign=Feed%3A+Phoronix+(Phoronix) sfdfasfas", providers).await;
    let option_without_tracking = cleanup_comment("test2 https://enteentelos.de bar https://www.phoronix.com/scan.php?page=news_item&px=Ioquake3-Auto-Updater jkhpoi", providers).await;
    let option_with_multiple_tracking = cleanup_comment("test3 https://enteentelos.de buzz https://www.google.de/search?q=google&source=hp&ei=LgC7ZJb4Oq6Gxc8Pke6SuAw&ved=0ahUKEwiWx7K85qCAAxUuQ_EDHRG3BMcQ4dUDCAs&uact=5&oq=google&gs_lp=Egdnd3Mtd2l6IgZnb29nbGUyERAuGIAEGLEDGIMBGMcBGNEDMgsQABiABBixAxiDATILEAAYgAQYsQMYgwEyCxAAGIAEGLEDGIMBMgsQABiABBixAxiDATILEAAYgAQYsQMYgwEyCxAAGIAEGLEDGIMBMggQABiABBixAzIIEAAYgAQYsQMyCxAAGIAEGLEDGIMBSP4TUIMOWPAScAF4AJABAJgBQaABrgKqAQE2uAEDyAEA-AEBqAIKwgIKEAAYAxiPARjqAsICChAuGAMYjwEY6gLCAgsQLhiKBRixAxiDAcICCxAAGIoFGLEDGIMB&sclient=gws-wiz aft3ge  https://www.phoronix.com/scan.php?page=news_item&px=Ioquake3-Auto-Updater&utm_source=feedburner&utm_medium=feed&utm_campaign=Feed%3A+Phoronix+(Phoronix)", providers).await;

    assert_eq!(option_with_and_without_tracking.len(), 1);
    assert_eq!(
        option_with_and_without_tracking[0],
        "https://www.phoronix.com/scan.php?page=news_item&px=Ioquake3-Auto-Updater"
    );

    assert_eq!(option_without_tracking.len(), 0);

    assert_eq!(option_with_multiple_tracking.len(), 2);
    assert_eq!(
        option_with_multiple_tracking[0],
        "https://www.google.de/search?q=google"
    );
    assert_eq!(
        option_with_multiple_tracking[1],
        "https://www.phoronix.com/scan.php?page=news_item&px=Ioquake3-Auto-Updater"
    );
}
