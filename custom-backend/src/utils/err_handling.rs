use std::fs::read_to_string;
use axum::{
    http::StatusCode, 
    response::{
        Html, 
        IntoResponse, 
        Response
    }
};

use crate::utils::init_utils::get_env_var;

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