// This is a catch-all file, at this point largely 
// to fetch environment variables

use std::env::{self, VarError};
use tracing::error;

pub fn get_env_var(env_var: &str) -> Result<String, VarError> {

    match env::var_os(env_var) {
        Some(s) => Ok(s.into_string().unwrap()),    // unwrap cannot panic here
        None => {
            error!("Environment variable {env_var} needs to be set.");
            Err(VarError::NotPresent)
        }
    }
}

// unwraps cannot panic here either; Results don't have errors in them
pub fn get_auth_paths() -> (String, String) {

    // load in certs from environment filepaths
    let cert_file = env::var_os("CRT_FILE")
        .expect("Certificates filepath variable not found in environment.")
        .into_string()
        .unwrap();
    let private_key_file = env::var_os("PK_FILE")
        .expect("Private keys filepath variable not found in environment.")
        .into_string()
        .unwrap();

    (cert_file, private_key_file)
}