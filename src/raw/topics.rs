//! API types to manage device groups.
//!
//! See <https://firebase.google.com/docs/cloud-messaging/manage-topic-subscriptions#manage-topic-subscriptions-admin-sdk>.
//! These REST APIs are not officially supported by google but they seem to use the old
//! [iid API](https://developers.google.com/instance-id/reference/server#manage_relationship_maps_for_multiple_app_instances).

use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
/// Batch add devices to a topic
///
/// If the topic does not exist, it will be created automaticially
pub struct BatchAdd {
    /// Topic to add devices to
    #[serde(rename = "to")]
    pub topic: String,
    /// List of registration tokens to add to that topic
    pub registration_tokens: Vec<String>,
}

impl super::Operation for BatchAdd {
    const DEFAULT_URL: &str = "https://iid.googleapis.com/iid/v1:batchAdd";

    fn get_url<T: reqwest::IntoUrl + Clone>(urls: &crate::APIURLs<T>) -> T {
        urls.add_to_topic.clone()
    }
}

///Remove registration tokens from a topic
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchRemove {
    /// Topic to remove devices from
    #[serde(rename = "to")]
    pub topic: String,
    /// List of registration tokens to remove from that topic
    pub registration_tokens: Vec<String>,
}

impl super::Operation for BatchRemove {
    const DEFAULT_URL: &str = "https://iid.googleapis.com/iid/v1:batchRemove";

    fn get_url<T: reqwest::IntoUrl + Clone>(urls: &crate::APIURLs<T>) -> T {
        urls.remove_from_topic.clone()
    }
}

/// Response from a batch topic operation
#[derive(Debug, Deserialize)]
pub(crate) struct TopicResults {
    /// Results for each registration token, in the same order as the request
    #[serde(deserialize_with = "deserialize_topic_results")]
    pub results: Vec<Result<(), TopicError>>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TopicResultHelper {
    Error { error: TopicError },
    Ok {},
}

fn deserialize_topic_results<'de, D>(deserializer: D) -> Result<Vec<Result<(), TopicError>>, D::Error>
where
    D: Deserializer<'de>,
{
    let helpers = Vec::<TopicResultHelper>::deserialize(deserializer)?;
    Ok(helpers
        .into_iter()
        .map(|r| match r {
            TopicResultHelper::Error { error } => Err(error),
            TopicResultHelper::Ok {} => Ok(()),
        })
        .collect())
}

/// Error returned for a single token in a batch topic operation
#[derive(Debug, Deserialize, Error)]
pub enum TopicError {
    /// The token does not belong to this project or lacks permissions
    #[serde(rename = "PERMISSION_DENIED")]
    #[error("permission denied")]
    PermissionDenied,
    /// An unrecognized error string from the API
    #[serde(untagged)]
    #[error("{0}")]
    Unknown(String),
}
