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
use crate::utils::init_utils::get_env_var;

#[derive(Debug)]
pub enum ServerError {
    JsonRejection(JsonRejection),
    JsonParseError(mysql_common::serde_json::Error),
    HttpError(axum::http::Error),
    IoError(std::io::Error),
    EnvVarError,
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

impl From<std::env::VarError> for ServerError {
    fn from(_var_err: std::env::VarError) -> Self {
        Self::EnvVarError
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        
        error!("{self:?}");
        make_500_resp()
    } 
}

// extracting this as public function to use elsewhere
// breaking up this huge error type into more specific errors 
// seems like a better idea
pub fn make_500_resp() -> Response {
    let html_500_err = match get_env_var("SERVER_ROOT") {

        Ok(sr) => {
            // get nice 500 error page...
            read_to_string(format!("{sr}/static/errors/500.html"))
                .unwrap_or(     
                    // ...or the quick 'n' dirty version if otherwise
                    "<!DOCTYPE html>\n\
                    <html><head>\n\
                    <title>500 Error</title>\n\
                    </head>\n\
                    <p>500 Error: Internal Server Error</p>\n\
                    </html>".to_string()
                )
        },
        // if we can't even find the env var, we're sending the short version
        Err(_) => {
            "<!DOCTYPE html>\n\
            <html><head>\n\
            <title>500 Error</title>\n\
            </head>\n\
            <p>500 Error: Internal Server Error</p>\n\
            </html>".to_string()
        }
    };
    
    let mut err_resp = Html(html_500_err).into_response();
    *err_resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

    err_resp
}