use std::io::{Read, Write};
use std::net::TcpStream;
use axum::{
    response::Response,
    body::Body,
};
use axum_extra::extract::Query;
use mysql_common::serde_json;
use tracing::{info, debug, error};

use crate::err_handling::ServerError;
use crate::archie_utils::get_env_var;

#[derive(Debug, serde::Serialize, serde::Deserialize)] 
pub struct ListInfo {
    list_name: String,
    author_user: String,
    attrs: Vec<String>,
}

// this function essentially relays the list information to the 
// Python container that actually gathers the data and formats it as a CSV.
//
// This CSV-formatted text is sent back to the client via this function, which
// package the CSV text as an HTTP response.
// 
// Issues in parsing the query are caught by Axum under the hood, automatically 
// responding with a 400 status code.
// 
// Requires the Query struct from axum-extra, not axum (standard), to parse query
// strings with arrays.
pub async fn convert_lb_list(list_info: Query<ListInfo>) -> Result<Response, ServerError> {
    
    let list_info = list_info.0;  // extract info struct contained in Query struct

    debug!("Conversion request received.");
    debug!("{:?}", list_info);
    //return Ok(Response::new(Body::new(String::from("got the query!\n\n"))));
    // establish connection to Python (Docker) container
    let py_cont_sock = get_env_var("PY_CONT_SOCK");
    let mut conn = TcpStream::connect(py_cont_sock)?;
    info!("Connection with Python container established.");

    // send out list info as char byte stream
    let stringified_json = serde_json::to_string(&list_info)?;
    conn.write_all(stringified_json.as_bytes())?;
    debug!("Response received from Python container");

    // receive CSV text into a string
    let mut csv_text = String::new();
    conn.read_to_string(&mut csv_text)?;
    
    // send off response through server, depending on Python
    // component response. 
    // It sends a string literal of "400 BAD REQUEST" if there
    // are any exceptions thrown in the CSV creation process,
    // and send the CSV if it's successful.
    let resp = if csv_text == "400 BAD REQUEST" {
        error!("The Python component threw an exception; see its container log for details.");
        Response::builder()
            .status(400)
            .header("Content-Type", "text/plain")
            .body(
                Body::new(
                    format!("The query for this request: {:?}", list_info)
                )
            )?
    } else {
        info!("Response sent off from server!");
        Response::builder()
            .status(200)
            .header("Content-Type", "text/csv")
            .body(Body::new(csv_text))?
    };

    Ok(resp)
}