use bytes::Bytes;
use axum::{
    body::Body,
    response::{IntoResponse, Response}
};
use mysql_common::chrono::DateTime;
use axum::http::StatusCode;
use tracing::error;
use vite_rs::ViteFile;
use crate::utils::err_handling::make_500_resp;

#[derive(vite_rs::Embed)]
#[root = "../"]
pub struct VitePage;

struct ViteFileWrapper(ViteFile, String);

impl IntoResponse for ViteFileWrapper {
    fn into_response(self) -> Response {

        let vf: ViteFile = self.0;
        let url_path: String = self.1;   // captured for debugging purposes
        
        let resp_start = Response::builder()
            .header("Content-Type", vf.content_type.clone())
            .header("Content-Length", vf.content_length);

        // add date
        let resp_with_opt_last_mod = match vf.last_modified {
            Some(date_int) => {
                resp_start.header(
                    "Last-Modified",
                    // the unwraps here are because date_int is a u64 and is only getting coerced into 
                    // a signed integer. The lost most-significant bit will not become an issue for...
                    // a VERY long time.
                    DateTime::from_timestamp(date_int.try_into().unwrap(), 0)
                        .unwrap()
                        .format("%a, %d %b %Y %H:%M:%S GMT")
                        .to_string()
                )
            },
            None => resp_start
        };
        
        let mut status = StatusCode::OK;

        // build body (and set status to 500 if necessary)
        let resp_body = if vf.content_type.starts_with("text") {
            let str_conv_opt = String::from_utf8(vf.bytes.to_vec());
            match str_conv_opt {
                Ok(content_string) => Body::from(content_string),
                Err(_) => {
                    error!("vite_io.rs: the path {} resulted in a failed UTF-8 parse", url_path);
                    status = StatusCode::INTERNAL_SERVER_ERROR;
                    Body::from("500 INTERNAL SERVER ERROR: failed to parse UTF-8 string on server side.")
                }
            }
        } else {
            Body::from(Bytes::copy_from_slice(&vf.bytes.to_vec()))
        };

        resp_with_opt_last_mod.status(status).body(resp_body).unwrap()
    }
}


fn get_vite_page(path: &str) -> Response {
    let page_option = VitePage::get(path);

    match page_option {
        Some(page) => ViteFileWrapper(page, String::from(path)).into_response(),
        None => {
            error!("Page {} was not found", path);
            make_500_resp()   // 500 because this mod is only called with dev-defined input
        }
    }
}

pub async fn homepage() -> Response {
    get_vite_page("index.html")
}

pub async fn lb_app_page() -> Response {
    get_vite_page("pages/lb-list-app.html")
}

pub async fn guestbook_page() -> Response {
    get_vite_page("pages/guestbook.html")
}