// the paths I want to use on my URLs are slightly different
// from the filepaths that were used to define and built th
// Vite assets, so this file helps map the URLs I want to 
// change onto the filepaths that Vite knows for my files. 
use std::collections::HashMap;
use axum::{extract, http::Uri, response::Response};
use vite_rs;
use vite_rs_axum_0_8::ViteServe;

// the server is usually run from the crate directory, and vite.config.ts is in the repo root
#[derive(vite_rs::Embed)]
#[root = "../"]     
struct StaticAssets;

pub async fn serve_statics(mut req: extract::Request) -> Response {
    
    // "/" gets mapped to "index.html" in ViteServe.serve()
    // by default
    let url_map = HashMap::from([
        ("/guestbook",    "/pages/guestbook.html"),
        ("/lb-list-conv", "/pages/lb-list-app.html")
    ]);

    let given_uri = req.uri().path();

    if url_map.contains_key(given_uri) {

        *req.uri_mut() = Uri::from_static(url_map[given_uri]);
    }

    ViteServe::new(StaticAssets::boxed())
        .serve(req)
        .await
}