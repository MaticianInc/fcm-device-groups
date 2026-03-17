use std::path::PathBuf;

use clap::{Parser, Subcommand};
use fcm_device_group::{
    APIURLs, FCMDeviceGroup, FCMDeviceGroupClient,
    raw::{device_groups::Operation as DeviceGroupsOperation, topics::TopicError},
};
use yup_oauth2::ServiceAccountAuthenticator;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, env = "GOOGLE_APPLICATION_CREDENTIALS")]
    auth_file: PathBuf,
    #[arg(long)]
    sender_id: String,
    #[arg(long, default_value = fcm_device_group::DEFAULT_URLS.device_groups)]
    device_groups_url: String,
    #[arg(long, default_value = fcm_device_group::DEFAULT_URLS.add_to_topic)]
    add_to_topic_url: String,
    #[arg(long, default_value = fcm_device_group::DEFAULT_URLS.remove_from_topic)]
    remove_from_topic_url: String,
    #[command(subcommand)]
    operation: DeviceGroupOperation,
}

#[derive(Debug, Subcommand)]
pub enum DeviceGroupOperation {
    /// Create a new device group with the following name
    /// See <https://firebase.google.com/docs/cloud-messaging/android/device-group#creating_a_device_group>
    Create {
        /// Name of the dev device group
        notification_key_name: String,
        /// registration IDS to create the device group with
        registration_ids: Vec<String>,
    },
    /// Add a devices to the device group
    /// See <https://firebase.google.com/docs/cloud-messaging/android/device-group#adding_and_removing_devices_from_a_device_group>
    Add {
        /// Key name of the device group.
        /// notification_key_name is not required for adding/removing registration tokens, but including it protects you against
        /// accidentally using the incorrect notification_key.
        #[arg(long)]
        notification_key_name: Option<String>,
        /// Device group notification key
        notification_key: String,
        /// Registration IDS to add
        registration_ids: Vec<String>,
    },
    /// Remove a device from a device group
    /// See <https://firebase.google.com/docs/cloud-messaging/android/device-group#adding_and_removing_devices_from_a_device_group>
    Remove {
        /// Key name of the device group.
        /// notification_key_name is not required for adding/removing registration tokens, but including it protects you against
        /// accidentally using the incorrect notification_key.
        #[arg(long)]
        notification_key_name: Option<String>,
        /// Device group notification key
        notification_key: String,
        /// Registration IDS to add
        registration_ids: Vec<String>,
    },
    GetKey {
        name: String,
    },
    AddDeviceToTopic {
        /// Topic to Add Devices to.
        ///
        /// This program adds the required `/topics/` prefix.
        topic_name: String,
        /// Ids to add to the topic
        registration_ids: Vec<String>,
    },
    RemoveDeviceFromTopic {
        /// Topic to remove devices from.
        ///
        /// This program adds the required `/topics/` prefix.
        topic_name: String,
        /// Ids to remove from the topic
        registration_ids: Vec<String>,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();

    let args = Args::parse();

    let secret = tokio::fs::read(args.auth_file).await.unwrap();
    let secret = yup_oauth2::parse_service_account_key(secret).unwrap();

    let auth = ServiceAccountAuthenticator::builder(secret)
        .build()
        .await
        .unwrap();

    let fcm_client = FCMDeviceGroupClient::with_url(
        APIURLs {
            device_groups: &args.device_groups_url,
            add_to_topic: &args.add_to_topic_url,
            remove_from_topic: &args.remove_from_topic_url,
        },
        &args.sender_id,
        auth,
    )
    .unwrap();

    log::info!("Running Request");
    match args.operation {
        DeviceGroupOperation::Create {
            notification_key_name,
            registration_ids,
        } => {
            fcm_client
                .create_group(notification_key_name, registration_ids)
                .await
                .unwrap();
        }
        DeviceGroupOperation::Add {
            notification_key_name: None,
            notification_key,
            registration_ids,
        } => {
            fcm_client
                .apply(DeviceGroupsOperation::Add {
                    notification_key_name: None,
                    notification_key,
                    registration_ids,
                })
                .await
                .unwrap();
        }
        DeviceGroupOperation::Add {
            notification_key_name: Some(notification_key_name),
            notification_key,
            registration_ids,
        } => {
            fcm_client
                .add_to_group(
                    FCMDeviceGroup {
                        notification_key_name,
                        notification_key,
                    },
                    registration_ids,
                )
                .await
                .unwrap();
        }
        DeviceGroupOperation::Remove {
            notification_key_name: None,
            notification_key,
            registration_ids,
        } => {
            fcm_client
                .apply(DeviceGroupsOperation::Remove {
                    notification_key_name: None,
                    notification_key,
                    registration_ids,
                })
                .await
                .unwrap();
        }
        DeviceGroupOperation::Remove {
            notification_key_name: Some(notification_key_name),
            notification_key,
            registration_ids,
        } => {
            fcm_client
                .remove_from_group(
                    FCMDeviceGroup {
                        notification_key_name,
                        notification_key,
                    },
                    registration_ids,
                )
                .await
                .unwrap();
        }
        DeviceGroupOperation::GetKey { name } => {
            let device_group = fcm_client.get_key(name).await.unwrap();
            println!("Device group {:?}", device_group);
            return;
        }
        DeviceGroupOperation::AddDeviceToTopic {
            registration_ids,
            topic_name,
        } => {
            let results = fcm_client
                .add_devices_to_topic(format!("/topics/{}", topic_name), registration_ids)
                .await
                .unwrap();
            let results = results.into_iter().collect::<Result<Vec<()>, TopicError>>().unwrap();
            println!("Added {} devices to {}", results.len(), topic_name);
        }
        DeviceGroupOperation::RemoveDeviceFromTopic {
            registration_ids,
            topic_name,
        } => {
            let results = fcm_client
                .remove_devices_from_topic(format!("/topics/{}", topic_name), registration_ids)
                .await
                .unwrap();
            let results = results.into_iter().collect::<Result<Vec<()>, TopicError>>().unwrap();
            println!("Removed {} devices from {}", results.len(), topic_name);
        }
    };
}
