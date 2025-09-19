//! Errors returns by the FCM device group APIs
//!
use std::fmt::Display;

use reqwest::{StatusCode, header::InvalidHeaderValue};
use serde::{Deserialize, de::DeserializeOwned};
use thiserror::Error;

/// Error when creating FCM Device Groups Client
#[derive(Debug, Error)]
pub enum FCMDeviceGroupClientCreationError {
    #[allow(missing_docs)]
    #[error("Error Making Authorization Header")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    #[allow(missing_docs)]
    #[error("Build Client Error")]
    ClientBuild(#[from] reqwest::Error),
}

#[allow(missing_docs)]
pub trait FCMDeviceGroupError: std::error::Error + Sized {
    fn from_error_str(error: FCMDeviceGroupsBadRequest) -> Option<Self>;
}

/// Error When Making an FCM Device Groups Request
///
/// E is a generic parameter for a request specific error
#[derive(Debug, Error)]
pub enum FCMDeviceGroupsRequestError<E: FCMDeviceGroupError> {
    /// Error http error
    #[error("Error Making HTTP Request with FCM")]
    HttpError(#[from] reqwest::Error),
    /// Parsed Bad Request Error
    #[error("Bad Request")]
    BadRequestError(#[from] E),
}

impl<E: FCMDeviceGroupError> FCMDeviceGroupsRequestError<E> {
    pub(crate) async fn json_response<T: DeserializeOwned>(
        resp: reqwest::Response,
    ) -> Result<T, Self> {
        match resp.error_for_status_ref() {
            Ok(_) => Ok(resp.json::<T>().await?),
            Err(e) => match e.status().unwrap() {
                StatusCode::BAD_REQUEST => {
                    let string_error = resp.json::<FCMDeviceGroupsBadRequest>().await?;
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
pub struct FCMDeviceGroupsBadRequest {
    /// Bad request message body from fcm
    pub error: String,
}

impl FCMDeviceGroupError for FCMDeviceGroupsBadRequest {
    fn from_error_str(error: FCMDeviceGroupsBadRequest) -> Option<Self> {
        Some(error)
    }
}

impl Display for FCMDeviceGroupsBadRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

#[allow(missing_docs)]
pub mod operation_errors {
    //! Operation Specific Errors

    use thiserror::Error;

    use crate::error::FCMDeviceGroupError;

    const ALREADY_EXISTS_MESSAGE: &str = "notification_key already exists";
    const NO_REGISTRATION_ID_MESSAGE: &str = "no valid registration ids";
    const KEY_NAME_AND_KEY_DONT_MATCH: &str =
        "notification_key_name doesn't match the group name of the notification_key";
    const KEY_NOT_FOUND: &str = "notification_key not found";

    pub type OperationResult<T, E> = Result<T, super::FCMDeviceGroupsRequestError<E>>;

    #[derive(Debug, Error)]
    pub enum CreateGroupError {
        #[error("{}", ALREADY_EXISTS_MESSAGE)]
        AlreadyExists,
        #[error("{}", NO_REGISTRATION_ID_MESSAGE)]
        NoValidRegistrationIds,
    }

    impl FCMDeviceGroupError for CreateGroupError {
        fn from_error_str(error: super::FCMDeviceGroupsBadRequest) -> Option<Self> {
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

    impl FCMDeviceGroupError for ChangeGroupMembersError {
        fn from_error_str(error: super::FCMDeviceGroupsBadRequest) -> Option<Self> {
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
    impl FCMDeviceGroupError for GetKeyError {
        fn from_error_str(error: super::FCMDeviceGroupsBadRequest) -> Option<Self> {
            match error.error.as_str() {
                KEY_NOT_FOUND => Some(Self::KeyNotFound),
                _ => None,
            }
        }
    }
}
