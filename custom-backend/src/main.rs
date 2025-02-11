use std::error::Error as StdError;
use warp::Filter;

mod archie_utils;
mod db_io;


#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {

    println!("Loading certificates and keys...");
    let (cert, pks) = archie_utils::get_auth_paths();
    println!("Authorization loaded!");

    println!("Defining routes...");
    let home = warp::path::end()
        .and(warp::get())
        .and(warp::fs::file(format!("{}/home.html", archie_utils::LOCAL_ROOT)));
    
    let hit_count = warp::path("hits")
        .and(warp::get())
        .then(db_io::update_hits);

    let static_content = warp::path("static")
        .and(warp::get())
        .and(warp::fs::dir(format!("{}/static/", archie_utils::LOCAL_ROOT)))
        .with(warp::compression::gzip());

    // unused currently, but keeping for potential future features
    let node_modules = warp::get()
        .and(warp::path("node_modules"))
        .and(warp::fs::dir(format!("{}/node_modules/", archie_utils::LOCAL_ROOT)))
        .with(warp::compression::gzip());

    // /guestbook shows guestbook, and /guestbook/entries is the endpoint for 
    // viewing/updating the entries in the guestbook.
    //
    // This ended up looking a little complicated with Warp, even if the logic is 
    // straight-forward, so here's the logical structure of the block below, 
    // in pseudocode:
    //
    // ```
    // if (path.starts_with("guestbook")) {
    //
    //      if (path == "guestbook") {
    //          REPLY with guestbook.html;
    //      }
    //      else if (path.ends_with("entries")) {
    //
    //          if (request.type == POST) {
    //              update_guestbook(new_entry);
    //          } 
    //          else if (request.type == GET) {
    //              get_guestbook();
    //          }
    //          else {
    //              REJECT with MethodNotAllowed; (implicit)
    //          }
    //      } 
    //      else {
    //          REJECT with 404 (implicit, I think);
    //      }
    // } 
    // else {
    //      (onto next filter)
    // }
    // ```
    let guestbook = warp::path("guestbook").and(
        warp::path::end()
            .and(warp::fs::file(format!("{}/guestbook.html", archie_utils::LOCAL_ROOT)))
        .or(  // above this line is the guestbook page proper; below is sorting 
              // requests to root/guestbook/entries into GETs and POSTs,
              // and calling the appropriate functions
        warp::path("entries").and(
            warp::post()
                .and(warp::body::json::<db_io::GuestbookEntry>())
                .then(db_io::update_guestbook)
            .or(
            warp::get()
                .then(db_io::get_guestbook))
            ))
        );

    let routes = home
        .or(static_content)
        .or(hit_count)
        .or(node_modules)
        .or(guestbook)
        .recover(archie_utils::handle_rejection);
    
    println!("Routes defined. Launching server!\n");
    warp::serve(routes)
        .tls()
        .cert_path(cert)
        .key_path(pks)
        .run(([0,0,0,0], 443))
        .await;

    Ok(())
}