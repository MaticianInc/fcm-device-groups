#![warn(missing_docs)]
//! A crate for using Firebase Cloud Messaging device groups.
//! See <https://firebase.google.com/docs/cloud-messaging/android/topic-messaging>
//!
//! Note that you will have to manually depend on a `reqwest` TLS feature if the default-tls feature is disabled.
use google_apis_common::GetToken;
use reqwest::{
    Client as HttpClient, IntoUrl, RequestBuilder, Response, Url,
    header::{self, HeaderMap, HeaderValue},
};

pub use raw::{Operation, OperationResponse};

use error::operation_errors::OperationResult;

pub mod error;
mod raw;

/// Default URL used for FCM device groups
pub const FIREBASE_NOTIFICATION_URL: &str = "https://fcm.googleapis.com/fcm/notification";

const FCM_DEVICE_GROUP_SCOPES: &[&str] = &["https://www.googleapis.com/auth/firebase.messaging"];

/// Client to use fcm device groups
#[derive(Clone)]
pub struct FCMDeviceGroupClient {
    url: Url,
    client: HttpClient,
    auth: Box<dyn GetToken + 'static>,
}

/// A Representation of an FCM Device group
#[derive(Debug)]
pub struct FCMDeviceGroup {
    /// Name of the device group
    pub notification_key_name: String,
    /// Key for this device group.
    ///
    /// Note that one device group may have multiple keys
    pub notification_key: String,
}

impl FCMDeviceGroupClient {
    /// Creates a new `FCMDeviceGroupClient` with the default url and the provided bearer auth string
    pub fn new(
        sender_id: &str,
        auth: impl GetToken + 'static,
    ) -> Result<Self, error::FCMDeviceGroupClientCreationError> {
        Self::with_url(FIREBASE_NOTIFICATION_URL, sender_id, auth)
    }

    /// Creates a new `FCMDeviceGroupClient` with the given url and the provided bearer auth string
    pub fn with_url(
        url: impl IntoUrl,
        sender_id: &str,
        auth: impl GetToken + 'static,
    ) -> Result<Self, error::FCMDeviceGroupClientCreationError> {
        let mut headers = HeaderMap::new();
        headers.insert("project_id", header::HeaderValue::try_from(sender_id)?);
        headers.insert(
            "access_token_auth",
            header::HeaderValue::from_static("true"),
        );

        Ok(Self {
            url: url.into_url().unwrap(),
            client: HttpClient::builder()
                .default_headers(headers)
                .connection_verbose(true)
                .build()?,
            auth: Box::new(auth),
        })
    }

    /// Creates a new `FCMDeviceGroupClient` with the given url and client. Note that the creator of the client
    /// is responsible for adding authorization headers
    pub fn with_client(
        client: HttpClient,
        url: impl IntoUrl,
        auth: impl GetToken + 'static,
    ) -> Self {
        Self {
            url: url.into_url().unwrap(),
            client,
            auth: Box::new(auth),
        }
    }

    /// Apply the given operation with with the client.
    pub async fn apply(
        &self,
        operation: Operation,
    ) -> Result<
        OperationResponse,
        error::FCMDeviceGroupsRequestError<error::FCMDeviceGroupsBadRequest>,
    > {
        let response = self.apply_raw(operation).await?;
        error::FCMDeviceGroupsRequestError::json_response(response).await
    }

    /// Create a new group with the provided name and ID
    pub async fn create_group(
        &self,
        notification_key_name: String,
        registration_ids: Vec<String>,
    ) -> OperationResult<FCMDeviceGroup, error::operation_errors::CreateGroupError> {
        self.apply_operation(Operation::Create {
            notification_key_name: notification_key_name.clone(),
            registration_ids,
        })
        .await
    }

    /// Add a set of registration IDS to the group
    pub async fn add_to_group(
        &self,
        group: FCMDeviceGroup,
        registration_ids: Vec<String>,
    ) -> OperationResult<FCMDeviceGroup, error::operation_errors::ChangeGroupMembersError> {
        self.apply_operation(Operation::Add {
            notification_key_name: Some(group.notification_key_name),
            notification_key: group.notification_key,
            registration_ids,
        })
        .await
    }

    /// Remove a set of registration IDS to the group
    pub async fn remove_from_group(
        &self,
        group: FCMDeviceGroup,
        registration_ids: Vec<String>,
    ) -> OperationResult<FCMDeviceGroup, error::operation_errors::ChangeGroupMembersError> {
        self.apply_operation(Operation::Remove {
            notification_key_name: Some(group.notification_key_name),
            notification_key: group.notification_key,
            registration_ids,
        })
        .await
    }

    /// Use this client to request the notification key for a given name
    pub async fn get_key(
        &self,
        notification_key_name: String,
    ) -> OperationResult<FCMDeviceGroup, error::operation_errors::GetKeyError> {
        let request = self
            .client
            .get(self.url.clone())
            .query(&[("notification_key_name", notification_key_name.as_str())])
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
        let response = self
            .add_token(request)
            .await
            .map_err(error::RawError::GetTokenError)?
            .send()
            .await?;
        let response =
            error::FCMDeviceGroupsRequestError::<error::operation_errors::GetKeyError>::json_response::<OperationResponse>(response)
                .await?;
        Ok(FCMDeviceGroup {
            notification_key_name,
            notification_key: response.notification_key,
        })
    }

    async fn apply_raw(&self, operation: Operation) -> Result<Response, error::RawError> {
        let request = self.client.post(self.url.clone()).json(&operation);

        let request = self
            .add_token(request)
            .await
            .map_err(error::RawError::GetTokenError)?;

        Ok(request.send().await?)
    }

    async fn add_token(
        &self,
        request: RequestBuilder,
    ) -> Result<RequestBuilder, Box<dyn std::error::Error + Send + Sync>> {
        match self.auth.get_token(FCM_DEVICE_GROUP_SCOPES).await? {
            Some(token) => Ok(request.bearer_auth(token)),
            None => Ok(request),
        }
    }

    async fn apply_operation<E: error::FCMDeviceGroupError>(
        &self,
        operation: Operation,
    ) -> OperationResult<FCMDeviceGroup, E> {
        let key_name = match &operation {
            Operation::Create {
                notification_key_name,
                ..
            } => notification_key_name.to_owned(),
            Operation::Add {
                notification_key_name,
                ..
            } => notification_key_name
                .as_ref()
                .expect("Applying an operation should always have a key name")
                .to_owned(),
            Operation::Remove {
                notification_key_name,
                ..
            } => notification_key_name
                .as_ref()
                .expect("Applying an operation should always have a key name")
                .to_owned(),
        };
        let response = self.apply_raw(operation).await?;
        let response =
            error::FCMDeviceGroupsRequestError::<E>::json_response::<OperationResponse>(response)
                .await?;
        Ok(FCMDeviceGroup {
            notification_key_name: key_name,
            notification_key: response.notification_key,
        })
    }
}
