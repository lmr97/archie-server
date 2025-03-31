use std::fs::read_to_string;
use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode, 
    response::{
        Html, 
        IntoResponse, 
        Response
    }
};

use tracing::error;

use crate::archie_utils;


#[derive(Debug)]
pub enum ServerError {
    DbErrDriver(mysql::error::DriverError), 
    DbErrUrl(mysql::UrlError),
    DbErrGeneral(mysql::error::Error),
    JsonRejection(JsonRejection),
    JsonParseError(mysql_common::serde_json::Error),
    HttpError(axum::http::Error),
    IoError(std::io::Error),
}

impl From<mysql::error::DriverError> for ServerError {
    fn from(db_err_driver: mysql::DriverError) -> Self {
        Self::DbErrDriver(db_err_driver)
    }
}

impl From<mysql::UrlError> for ServerError {
    fn from(db_err_url: mysql::UrlError) -> Self {
        Self::DbErrUrl(db_err_url)
    }
}

impl From<mysql::error::Error> for ServerError {
    fn from(db_err: mysql::Error) -> Self {
        Self::DbErrGeneral(db_err)
    }
}

impl From<JsonRejection> for ServerError {
    fn from(rejection: JsonRejection) -> Self {
        Self::JsonRejection(rejection)
    }
}

impl From<mysql_common::serde_json::Error> for ServerError {
    fn from(json_err: mysql_common::serde_json::Error) -> Self {
        Self::JsonParseError(json_err)
    }
}

impl From<axum::http::Error> for ServerError {
    fn from(http_err: axum::http::Error) -> Self {
        Self::HttpError(http_err)
    }
}

impl From<std::io::Error> for ServerError {
    fn from(io_err: std::io::Error) -> Self {
        Self::IoError(io_err)
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::DbErrDriver(e)  => {
                error!("OS emmitted an error via driver: {:?}", e);
            },
            ServerError::DbErrUrl(e)  => {
                error!("Database URL misspecified, or DB inaccessible: {:?}", e);
            },
            ServerError::DbErrGeneral(e) => {
                error!("Error in database I/O: {:?}", e);
            },
            ServerError::JsonRejection(e) => {
                error!("JSON could not be parsed from HTTP request: {:?}", e);
            }
            ServerError::JsonParseError(e) => {
                error!("JSON could not be parsed within server: {:?}", e);
            }
            ServerError::HttpError(e) => {
                error!("Error in HTTP handling: {:?}", e);
            }
            ServerError::IoError(e) => {
                error!("Error in communication across socket: {:?}", e);
            }
        };

        // get nice 500 error page...
        let html_500_err = read_to_string(
            format!(
                "{}/static/errors/500.html", 
                archie_utils::get_env_var("SERVER_ROOT")
            ))
            .unwrap_or(     // ...or the quick 'n' dirty version if otherwise
                "<!DOCTYPE html>\n\
                <html><head>\n\
                <title>500 Error</title>\n\
                </head>\n\
                <p>500 Error: Internal Server Error</p>\n\
                <p>This is most likely due to issues with the database.</p>\n\
                </html>".to_string()
            );

        let mut err_resp = Html(html_500_err).into_response();
        *err_resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

        err_resp
    } 
}