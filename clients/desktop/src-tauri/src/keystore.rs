//! OS keychain integration via the `keyring` crate.
//!
//! - Linux: Secret Service API (GNOME Keyring / KWallet)
//! - Windows: Credential Manager

use keyring::Entry;
use serde::{Deserialize, Serialize};

const SERVICE_NAME: &str = "com.chatalot.app";

#[derive(Debug, Serialize, Deserialize)]
pub struct KeystoreError {
    message: String,
}

impl From<keyring::Error> for KeystoreError {
    fn from(err: keyring::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}


/// Store a key in the OS keychain.
#[tauri::command]
pub fn store_key(key_name: String, value: String) -> Result<(), KeystoreError> {
    let entry = Entry::new(SERVICE_NAME, &key_name)?;
    entry.set_password(&value)?;
    Ok(())
}

/// Retrieve a key from the OS keychain.
#[tauri::command]
pub fn get_key(key_name: String) -> Result<Option<String>, KeystoreError> {
    let entry = Entry::new(SERVICE_NAME, &key_name)?;
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Delete a key from the OS keychain.
#[tauri::command]
pub fn delete_key(key_name: String) -> Result<(), KeystoreError> {
    let entry = Entry::new(SERVICE_NAME, &key_name)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
