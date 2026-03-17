//! Contains modules for the raw API data types

use reqwest::IntoUrl;
use serde::Serialize;

pub mod device_groups;
pub mod topics;

/// Trait implemented for Raw FCM device management operations
pub trait Operation: Serialize {
    /// The default URL for this operation
    const DEFAULT_URL: &'static str;

    ///URL to use for this operation given the set of available urls
    fn get_url<T: IntoUrl + Clone>(urls: &crate::APIURLs<T>) -> T;
}
