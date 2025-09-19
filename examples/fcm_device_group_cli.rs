use std::path::PathBuf;

use clap::{Parser, Subcommand};
use fcm_device_group::{FCMDeviceGroupClient, FIREBASE_NOTIFICATION_URL, Operation};
use reqwest::Url;
use yup_oauth2::ServiceAccountAuthenticator;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, env = "GOOGLE_APPLICATION_CREDENTIALS")]
    auth_file: PathBuf,
    #[arg(long)]
    sender_id: String,
    #[arg(long, default_value =FIREBASE_NOTIFICATION_URL)]
    url: Url,
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
}

const FCM_SCOPES: &[&str] = &["https://www.googleapis.com/auth/firebase.messaging"];

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
    let token = auth.token(FCM_SCOPES).await.unwrap();
    let fcm_client =
        FCMDeviceGroupClient::with_url(args.url, &args.sender_id, token.token().unwrap()).unwrap();

    log::info!("Running Request");
    let notification_key = match args.operation {
        DeviceGroupOperation::Create {
            notification_key_name,
            registration_ids,
        } => fcm_client
            .apply(Operation::Create {
                notification_key_name,
                registration_ids,
            })
            .await
            .unwrap(),
        DeviceGroupOperation::Add {
            notification_key_name,
            notification_key,
            registration_ids,
        } => fcm_client
            .apply(Operation::Add {
                notification_key_name,
                notification_key,
                registration_ids,
            })
            .await
            .unwrap(),
        DeviceGroupOperation::Remove {
            notification_key_name,
            notification_key,
            registration_ids,
        } => fcm_client
            .apply(Operation::Remove {
                notification_key_name,
                notification_key,
                registration_ids,
            })
            .await
            .unwrap(),
        DeviceGroupOperation::GetKey { name } => {
            let device_group = fcm_client.get_key(name).await.unwrap();
            println!("Device group {:?}", device_group);
            return;
        }
    };

    println!("Notification Key is {:?}", notification_key);
}
