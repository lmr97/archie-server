use std::fs::read_to_string;
use axum::{
    extract::rejection::JsonRejection, 
    response::{Html, IntoResponse, Response}
};

use crate::utils;


#[derive(Debug)]
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
        match self {
            WebsiteError::DatabaseErrorGeneral(e) => {e;},
            WebsiteError::DatabaseErrorUrl(e)  => {e;},
            WebsiteError::JsonError(e)    => {e;}
        };

        // log error

        // get nice 500 error page
        let html_500_err = read_to_string(
            format!("{}/static/errors/500.html", utils::LOCAL_ROOT))
            .unwrap_or(
                "<!DOCTYPE html>\n\
                <html><head>\n\
                <title>500 Error</title>\n\
                </head>\n\
                <p>500 Error: Internal Server Error</p>\n\
                <p>This is most likely due to issues with the database.</p>\n\
                </html>".to_string()
            ); 

        Html(html_500_err).into_response()
    } 
}