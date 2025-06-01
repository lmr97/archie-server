use bytes::Bytes;
use axum::{
    body::Body,
    response::{IntoResponse, Response}
};
use mysql_common::chrono::DateTime;
use vite_rs::ViteFile;
use crate::utils::err_handling::make_500_resp;

#[derive(vite_rs::Embed)]
#[root = "../"]
pub struct VitePage;

struct ViteFileWrapper(ViteFile);

impl IntoResponse for ViteFileWrapper {
    fn into_response(self) -> Response {

        let vf: ViteFile = self.0;
        
        let resp_start = Response::builder()
            .header("Content-Type", vf.content_type.to_string())
            .header("Content-Length", vf.content_length);

        let resp_with_opt_last_mod = match vf.last_modified {
            Some(date_int) => {
                resp_start.header(
                    "Last-Modified",
                    DateTime::from_timestamp(date_int.try_into().unwrap(), 0)
                        .unwrap()
                        .format("%a, %d %b %Y %H:%M:%S GMT")
                        .to_string()
                )
            },
            None => resp_start
        };
        
        let resp_body = if vf.content_type.starts_with("text") {
            Body::from(String::from_utf8(vf.bytes.to_vec()).unwrap())
        } else {
            Body::from(Bytes::copy_from_slice(&vf.bytes.to_vec()))
        };

        resp_with_opt_last_mod.body(resp_body).unwrap()
    }
}


fn get_vite_page(path: &str) -> Response {
    let page_option = VitePage::get(path);

    match page_option {
        Some(page) => ViteFileWrapper(page).into_response(),
        None => make_500_resp()
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