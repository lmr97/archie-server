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
pub enum WebsiteError {
    DbErrGeneral(mysql::error::Error),
    DbErrDriver(mysql::error::DriverError), 
    DbErrUrl(mysql::UrlError),
    JsonError(JsonRejection)
}

impl From<mysql::error::Error> for WebsiteError {
    fn from(db_err: mysql::Error) -> Self {
        Self::DbErrGeneral(db_err)
    }
}

impl From<mysql::error::DriverError> for WebsiteError {
    fn from(db_err_driver: mysql::DriverError) -> Self {
        Self::DbErrDriver(db_err_driver)
    }
}

impl From<mysql::UrlError> for WebsiteError {
    fn from(db_err_url: mysql::UrlError) -> Self {
        Self::DbErrUrl(db_err_url)
    }
}

impl From<JsonRejection> for WebsiteError {
    fn from(rejection: JsonRejection) -> Self {
        Self::JsonError(rejection)
    }
}

impl IntoResponse for WebsiteError {
    fn into_response(self) -> Response {
        match self {
            WebsiteError::DbErrGeneral(e) => {
                error!("Error in database I/O: {:?}", e);
            },
            WebsiteError::DbErrUrl(e)  => {
                error!("Database URL misspecified, or DB inaccessible: {:?}", e);
            },
            WebsiteError::DbErrDriver(e)  => {
                error!("OS emmitted an error via driver: {:?}", e);
            },
            WebsiteError::JsonError(e)    => {
                error!("JSON could not be parsed: {:?}", e);
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