//! Errors returns by the FCM device group APIs
//!
use std::fmt::Display;

use reqwest::{StatusCode, header::InvalidHeaderValue};
use serde::{Deserialize, de::DeserializeOwned};
use thiserror::Error;

/// Error when creating FCM Device Groups Client
#[derive(Debug, Error)]
pub enum FCMDevicesClientCreationError {
    #[allow(missing_docs)]
    #[error("Error Making Authorization Header")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    #[allow(missing_docs)]
    #[error("Build Client Error")]
    ClientBuild(#[from] reqwest::Error),
}

#[allow(missing_docs)]
pub trait FCMDevicesError: std::error::Error + Sized {
    fn from_error_str(error: FCMDevicesBadRequest) -> Option<Self>;
}

/// Error When Making an FCM Device Groups Request
///
/// E is a generic parameter for a request specific error
#[derive(Debug, Error)]
pub enum FCMDevicesRequestError<E: FCMDevicesError> {
    /// Error http error
    #[error("Error Making HTTP Request with FCM")]
    HttpError(#[from] reqwest::Error),
    /// Get Token Error
    #[error("Error Getting Auth Token")]
    GetTokenError(Box<dyn std::error::Error + Send + Sync>),
    /// Parsed Bad Request Error
    #[error("Bad Request")]
    BadRequestError(#[from] E),
}

impl<E: FCMDevicesError> FCMDevicesRequestError<E> {
    pub(crate) async fn json_response<T: DeserializeOwned>(
        resp: reqwest::Response,
    ) -> Result<T, Self> {
        match resp.error_for_status_ref() {
            Ok(_) => Ok(resp.json::<T>().await?),
            Err(e) => match e.status().unwrap() {
                StatusCode::BAD_REQUEST => {
                    let string_error = resp.json::<FCMDevicesBadRequest>().await?;
                    Err(match E::from_error_str(string_error) {
                        Some(custom_error) => Self::BadRequestError(custom_error),
                        None => Self::HttpError(e),
                    })
                }
                _ => Err(Self::HttpError(e)),
            },
        }
    }
}

/// Bad Request Error from FCM
#[derive(Debug, Deserialize, Error)]
pub struct FCMDevicesBadRequest {
    /// Bad request message body from fcm
    pub error: String,
}

impl FCMDevicesError for FCMDevicesBadRequest {
    fn from_error_str(error: FCMDevicesBadRequest) -> Option<Self> {
        Some(error)
    }
}

impl Display for FCMDevicesBadRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

#[allow(missing_docs)]
pub mod operation_errors {
    //! Operation Specific Errors

    use thiserror::Error;

    use crate::error::FCMDevicesError;

    const ALREADY_EXISTS_MESSAGE: &str = "notification_key already exists";
    const NO_REGISTRATION_ID_MESSAGE: &str = "no valid registration ids";
    const KEY_NAME_AND_KEY_DONT_MATCH: &str =
        "notification_key_name doesn't match the group name of the notification_key";
    const KEY_NOT_FOUND: &str = "notification_key not found";
    const TOPIC_NAME_FORMAT_INVALID: &str = "Topic name format is invalid";
    const MISSING_IID_TOKEN: &str = "MissingIIdToken";

    pub type OperationResult<T, E> = Result<T, super::FCMDevicesRequestError<E>>;

    #[derive(Debug, Error)]
    pub enum CreateGroupError {
        #[error("{}", ALREADY_EXISTS_MESSAGE)]
        AlreadyExists,
        #[error("{}", NO_REGISTRATION_ID_MESSAGE)]
        NoValidRegistrationIds,
    }

    impl FCMDevicesError for CreateGroupError {
        fn from_error_str(error: super::FCMDevicesBadRequest) -> Option<Self> {
            match error.error.as_str() {
                ALREADY_EXISTS_MESSAGE => Some(Self::AlreadyExists),
                NO_REGISTRATION_ID_MESSAGE => Some(Self::NoValidRegistrationIds),
                _ => None,
            }
        }
    }

    #[derive(Debug, Error)]
    pub enum ChangeGroupMembersError {
        #[error("{}", NO_REGISTRATION_ID_MESSAGE)]
        NoValidRegistrationIds,
        #[error("{KEY_NAME_AND_KEY_DONT_MATCH}")]
        KeyNameAndKeyDontMatch,
        #[error("{KEY_NOT_FOUND}")]
        KeyNotFound,
    }

    impl FCMDevicesError for ChangeGroupMembersError {
        fn from_error_str(error: super::FCMDevicesBadRequest) -> Option<Self> {
            match error.error.as_str() {
                NO_REGISTRATION_ID_MESSAGE => Some(Self::NoValidRegistrationIds),
                KEY_NAME_AND_KEY_DONT_MATCH => Some(Self::KeyNameAndKeyDontMatch),
                KEY_NOT_FOUND => Some(Self::KeyNotFound),
                _ => None,
            }
        }
    }

    #[derive(Debug, Error)]
    pub enum GetKeyError {
        #[error("{KEY_NOT_FOUND}")]
        KeyNotFound,
    }
    impl FCMDevicesError for GetKeyError {
        fn from_error_str(error: super::FCMDevicesBadRequest) -> Option<Self> {
            match error.error.as_str() {
                KEY_NOT_FOUND => Some(Self::KeyNotFound),
                _ => None,
            }
        }
    }

    #[derive(Debug, Error)]
    pub enum TopicsError {
        #[error("{TOPIC_NAME_FORMAT_INVALID}")]
        TopicNameFormatInvalid,
        #[error("No Registration Tokens were provided with this request")]
        NoRegistrationTokens,
    }

    impl FCMDevicesError for TopicsError {
        fn from_error_str(error: super::FCMDevicesBadRequest) -> Option<Self> {
            match error.error.as_str() {
                TOPIC_NAME_FORMAT_INVALID => Some(Self::TopicNameFormatInvalid),
                MISSING_IID_TOKEN => Some(Self::NoRegistrationTokens),
                _ => None,
            }
        }
    }
}

/// Generic Error for all operaturns returned from this library
#[derive(Debug, Error)]
pub enum RawError {
    /// HTTP error response from request
    #[error("Error Making HTTP Request with FCM")]
    HttpError(#[from] reqwest::Error),
    /// Get Token Error
    #[error("Error Getting Auth Token")]
    GetTokenError(Box<dyn std::error::Error + Send + Sync>),
}

impl<E: FCMDevicesError> From<RawError> for FCMDevicesRequestError<E> {
    fn from(raw: RawError) -> Self {
        match raw {
            RawError::HttpError(error) => Self::HttpError(error),
            RawError::GetTokenError(error) => Self::GetTokenError(error),
        }
    }
}
