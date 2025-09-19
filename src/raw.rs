use serde::{Deserialize, Serialize};

/// Represents a POST operation to fcm. See <https://firebase.google.com/docs/cloud-messaging/android/device-group>
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "lowercase")]
pub enum Operation {
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
        #[serde(skip_serializing_if = "Option::is_none")]
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
        #[serde(skip_serializing_if = "Option::is_none")]
        notification_key_name: Option<String>,
        /// Device group notification key
        notification_key: String,
        /// Registration IDS to add
        registration_ids: Vec<String>,
    },
}

/// Response from a POST Operation
#[derive(Debug, Deserialize, PartialEq, Eq, Hash)]
pub struct OperationResponse {
    /// Key of the effected device group
    pub notification_key: String,
}
