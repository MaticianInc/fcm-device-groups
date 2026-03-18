#![warn(missing_docs)]
//! A crate for using Firebase Cloud Messaging Devices.
//!
//! This crate supports adding and removing devices from both
//! [device groups](https://firebase.google.com/docs/cloud-messaging/device-group)
//! and [topics](https://firebase.google.com/docs/cloud-messaging/android/topic-messaging).
//!
//! Note that you will have to manually depend on a `reqwest` TLS feature if the `default-tls` feature is disabled.

pub use google_apis_common::{
    GetToken,
    auth::{GetTokenClone, NoToken},
};
use reqwest::{
    Client as HttpClient, IntoUrl, RequestBuilder, Response, Url,
    header::{self, HeaderMap, HeaderValue},
};

use raw::{
    Operation,
    device_groups::{Operation as DeviceGroupsOperation, OperationResponse},
    topics::{BatchAdd, BatchRemove, TopicResults},
};

use error::operation_errors::OperationResult;

pub use api_urls::APIURLs;

use crate::raw::topics::TopicError;
mod api_urls;
pub mod error;
pub mod raw;

// Topics dont seem to require their own scope for some reason
const FCM_DEVICE_GROUP_SCOPES: &[&str] = &["https://www.googleapis.com/auth/firebase.messaging"];

/// Set of default URLs to use with firebase
pub const DEFAULT_URLS: APIURLs<&str> = APIURLs {
    device_groups: DeviceGroupsOperation::DEFAULT_URL,
    add_to_topic: BatchAdd::DEFAULT_URL,
    remove_from_topic: BatchRemove::DEFAULT_URL,
};

/// Client to use fcm device groups
#[derive(Clone)]
pub struct FCMDevicesClient {
    urls: APIURLs<Url>,
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

impl FCMDevicesClient {
    /// Creates a new `FCMDevicesClient` with the default url and the provided bearer auth string
    pub fn new(
        sender_id: &str,
        auth: impl GetToken + 'static,
    ) -> Result<Self, error::FCMDevicesClientCreationError> {
        Self::with_url(DEFAULT_URLS, sender_id, auth)
    }

    /// Creates a new `FCMDevicesClient` with the given url and the provided bearer auth string
    pub fn with_url(
        urls: APIURLs<impl IntoUrl + Clone>,
        sender_id: &str,
        auth: impl GetToken + 'static,
    ) -> Result<Self, error::FCMDevicesClientCreationError> {
        let mut headers = HeaderMap::new();
        headers.insert("project_id", header::HeaderValue::try_from(sender_id)?);
        headers.insert(
            "access_token_auth",
            header::HeaderValue::from_static("true"),
        );

        Self::with_client(
            HttpClient::builder()
                .default_headers(headers)
                .connection_verbose(true)
                .build()?,
            urls,
            auth,
        )
    }

    /// Creates a new `FCMDevicesClient` with the given url and client. Note that the creator of the client
    /// is responsible for adding authorization headers
    pub fn with_client(
        client: HttpClient,
        urls: APIURLs<impl IntoUrl + Clone>,
        auth: impl GetToken + 'static,
    ) -> Result<Self, error::FCMDevicesClientCreationError> {
        Ok(Self {
            urls: urls.into_cannonical()?,
            client,
            auth: Box::new(auth),
        })
    }

    /// Apply the given operation with with the client.
    pub async fn apply(
        &self,
        operation: impl Operation,
    ) -> Result<
        OperationResponse,
        error::FCMDevicesRequestError<error::FCMDevicesBadRequest>,
    > {
        let response = self.apply_raw(operation).await?;
        error::FCMDevicesRequestError::json_response(response).await
    }

    /// Create a new group with the provided name and ID
    pub async fn create_group(
        &self,
        notification_key_name: String,
        registration_ids: Vec<String>,
    ) -> OperationResult<FCMDeviceGroup, error::operation_errors::CreateGroupError> {
        self.apply_device_groups_operation(DeviceGroupsOperation::Create {
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
        self.apply_device_groups_operation(DeviceGroupsOperation::Add {
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
        self.apply_device_groups_operation(DeviceGroupsOperation::Remove {
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
            .get(self.urls.device_groups.clone())
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
            error::FCMDevicesRequestError::<error::operation_errors::GetKeyError>::json_response::<OperationResponse>(response)
                .await?;
        Ok(FCMDeviceGroup {
            notification_key_name,
            notification_key: response.notification_key,
        })
    }

    /// Add a set of devices to an FCM Topic
    pub async fn add_devices_to_topic(
        &self,
        topic: String,
        registration_tokens: Vec<String>,
    ) -> OperationResult<Vec<Result<(), TopicError>>, error::operation_errors::TopicsError> {
        self.apply_topics_operation(BatchAdd {
            topic,
            registration_tokens,
        }).await
    }

    /// Remove a set of devices from an FCM Topic
    pub async fn remove_devices_from_topic(
        &self,
        topic: String,
        registration_tokens: Vec<String>,
    ) -> OperationResult<Vec<Result<(), TopicError>>, error::operation_errors::TopicsError> {
        self.apply_topics_operation(BatchRemove {
            topic,
            registration_tokens,
        }).await
    }

    async fn apply_raw<Op: Operation>(&self, operation: Op) -> Result<Response, error::RawError> {
        let request = self.client.post(Op::get_url(&self.urls)).json(&operation);

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

    async fn apply_device_groups_operation<E: error::FCMDevicesError>(
        &self,
        operation: DeviceGroupsOperation,
    ) -> OperationResult<FCMDeviceGroup, E> {
        let key_name = match &operation {
            DeviceGroupsOperation::Create {
                notification_key_name,
                ..
            } => notification_key_name.to_owned(),
            DeviceGroupsOperation::Add {
                notification_key_name,
                ..
            } => notification_key_name
                .as_ref()
                .expect("Applying an operation should always have a key name")
                .to_owned(),
            DeviceGroupsOperation::Remove {
                notification_key_name,
                ..
            } => notification_key_name
                .as_ref()
                .expect("Applying an operation should always have a key name")
                .to_owned(),
        };
        let response = self.apply_raw(operation).await?;
        let response =
            error::FCMDevicesRequestError::<E>::json_response::<OperationResponse>(response)
                .await?;
        Ok(FCMDeviceGroup {
            notification_key_name: key_name,
            notification_key: response.notification_key,
        })
    }

    async fn apply_topics_operation(&self, op: impl Operation) -> OperationResult<Vec<Result<(), TopicError>>, error::operation_errors::TopicsError> {
        let response = self.apply_raw(op).await?;

        let topic_results = error::FCMDevicesRequestError::<error::operation_errors::TopicsError>::json_response::<TopicResults>(response)
                .await?;
        Ok(topic_results.results)
    }
}
