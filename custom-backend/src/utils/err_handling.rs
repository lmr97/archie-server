use axum::{
    http::StatusCode, 
    response::{Html, IntoResponse, Response}
};

use crate::srv_io::vite_io;

// extracting this as public function to use elsewhere
// breaking up this huge error type into more specific errors 
// seems like a better idea
pub fn make_500_resp() -> Response {
    
    let default_error = "<!DOCTYPE html>\n\
                    <html><head>\n\
                    <title>500 Error</title>\n\
                    </head>\n\
                    <p>500 Error: Internal Server Error</p>\n\
                    </html>".to_string();

    let html_500_err = match vite_io::VitePage::get("static/errors/500.html") {

        Some(err_page) => {
            String::from_utf8(err_page.bytes.to_vec()).unwrap_or(default_error)
        },
        // if we can't even find the env var, we're sending the short version
        None => default_error
    };
    
    let mut err_resp = Html(html_500_err).into_response();
    *err_resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

    err_resp
}