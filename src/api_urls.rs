use reqwest::{IntoUrl, Url};

/// A set of URLs to use for the operations
///
/// Use this struct to override the default firebase urls
#[derive(Debug, Clone)]
pub struct APIURLs<T: IntoUrl + Clone> {
    /// URL to use for device group operations
    pub device_groups: T,
    /// URL to use for topic BatchAdd
    pub add_to_topic: T,
    /// Url to use for topic BatchRemove operations
    pub remove_from_topic: T,
}
impl<T: IntoUrl + Clone> APIURLs<T> {
    pub(crate) fn into_cannonical(self) -> reqwest::Result<APIURLs<Url>> {
        let APIURLs {
            device_groups,
            add_to_topic,
            remove_from_topic,
        } = self;

        Ok(APIURLs {
            device_groups: device_groups.into_url()?,
            add_to_topic: add_to_topic.into_url()?,
            remove_from_topic: remove_from_topic.into_url()?,
        })
    }
}
