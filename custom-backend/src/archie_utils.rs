use std::env;
use axum::{
    extract::rejection::JsonRejection, 
    http::StatusCode,
    response::{Response, IntoResponse}
};


pub static LOCAL_ROOT: &str = "/home/martin/archie-server";

pub enum WebsiteError {
    DatabaseErrorGeneral(mysql::error::Error), 
    DatabaseErrorUrl(mysql::UrlError),
    JsonError(JsonRejection)
}

impl From<mysql::error::Error> for WebsiteError {
    fn from(db_err: mysql::Error) -> Self {
        Self::DatabaseErrorGeneral(db_err)
    }
}

impl From<mysql::UrlError> for WebsiteError {
    fn from(db_err_url: mysql::UrlError) -> Self {
        Self::DatabaseErrorUrl(db_err_url)
    }
}

impl From<JsonRejection> for WebsiteError {
    fn from(rejection: JsonRejection) -> Self {
        Self::JsonError(rejection)
    }
}

impl IntoResponse for WebsiteError {
    fn into_response(self) -> Response {
        let body = match self {
            WebsiteError::DatabaseErrorGeneral(err) => {
                err; // log error here
                "Something went wrong with our database. Sorry for the trouble."
            },
            WebsiteError::DatabaseErrorUrl(err) => {
                err; // log error here
                "The server is looking in the wrong place for the database. This is an easy fix, and will be resolved very soon."
            },
            WebsiteError::JsonError(err) => {
                err; // log error here
                "Some data could not be parsed properly. I'll address this soon, so try again later."
            }
        };

        // it's often easiest to implement `IntoResponse` by calling other implementations
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    } 
}

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
