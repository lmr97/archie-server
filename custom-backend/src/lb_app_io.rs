/* This file is the interface to a Python app I made to
collect the data from a given film list on letterboxd.com,
formatting it into a CSV file.

The original program had a CLI, so instead of opening the server
up to injections onto its command line, I reworked the program 
to take a byte stream from a socket as its input. This file 
handles the I/O to/from the Python program, streaming out 
its output to the client. */

use std::convert::Infallible;
use std::io::{Read, Write};
use std::net::TcpStream;
use futures_util::stream::{self, Stream};
use axum::response::{Sse, sse::Event};
use axum_extra::extract::Query;
use futures_util::StreamExt;
use mysql_common::serde_json;
use tracing::{debug, error, info};

use crate::err_handling::ServerError;
use crate::archie_utils::get_env_var;

#[derive(Debug, serde::Serialize, serde::Deserialize)] 
pub struct ListInfo {
    list_name: String,
    author_user: String,
    attrs: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct ListRow {
    curr_row: u16,
    total_rows: usize,
    row: String,
}

enum ErrMsg {
    Err400,
    Err500,
}

fn build_err_event(mut json: ListRow, err: ErrMsg) -> Event {

    let msg = match err {
        ErrMsg::Err400 => "400 BAD REQUEST",
        ErrMsg::Err500 => "500 INTERNAL SERVER ERROR"
    };

    json.row = String::from(msg);

    // all data in this statement's JSON is ASCII: 
    // - curr_row and total_rows fields are integers (or rendered as such)
    // - row field can only take on the string values hard-coded above 
    // so the JSON data is guaranteed serializable, and will not panic
    Event::default()
        .event("error")
        .json_data(json)
        .unwrap() 
}


// Gets a row of the CSV data streamed from the Python container,
// converts it to a String, then packages it up as an `Event`
// and returns the `Event`.It returns an event of type "complete" 
// when the conversion is done.
//
// The `Event` is packaged up with both the row number that was converted
// (1-indexed), as well as the total rows. Sending the total rows every time
// is a little bit of overhead, but it helps simplify the code here,
// keeping me from having an `Sse` struct with a `Stream` of two different 
// types somehow concatenated together.
//
// To make the Event, this function first reads 2 bytes, which indicate 
// the length of the row (in bytes), and then it reads an amount of bytes 
// equal to the number comprised of the first 2 bytes received.
// 
// Will return an event of type "error" if there are any issues reading
// from the Python container, or converting the output to a UTF-8 string.
fn get_list_row(conn: &mut TcpStream, mut row_json: ListRow) -> Event {

    let mut row_length_buf = [0; 2];

    
    /* READ ROW LENGTH BYTES */

    // Any read errors need to be manually handled right here;
    // they cannot be passed up through the callers, and I'd prefer
    // to not crash the server with an unwrap()
    //
    // After the emission of an error event, the connection will be 
    // terminated by the client
    match conn.read_exact(&mut row_length_buf) {
        // the kind of error encountered here is usually that
        // the buffer was not filled, which ends up working out. 
        // Catching any other errors here and logging them
        // in case new ones arise.
        Ok(_) => {},
        Err(e) => {error!("Error on read of row-size bytes: {e:?}");}
    };
    debug!("Bytes received: {row_length_buf:?}");


    /* INTERPRET LENGTH BYTES */
    let row_length_u16 = u16::from_be_bytes(row_length_buf);
    let row_length = usize::from(row_length_u16);
    debug!("Indiv. row length received: {:?}", row_length);


    /* READ ROW DATA BYTES */
    let mut row_data_buf = vec![0; row_length];

    match conn.read_exact(&mut row_data_buf) {
        Ok(_) => {},
        Err(e) => {
            error!("I/O Error: reading a CSV line from Python container failed: {e:?}");
            return build_err_event(row_json, ErrMsg::Err500);    
        }         
    };

    debug!("{row_data_buf:?}");


    /* CONVERT ROW BYTES TO STRING */
    let Ok(row_data) = String::from_utf8(row_data_buf) 
    else {
        error!("Conversion Error: bytes read from Python container could not be converted into a (UTF-8) string.");
        error!("Run on Debug mode to see bytes read.");
        return build_err_event(row_json, ErrMsg::Err500);  
    };
    debug!("Indiv. row data received: {:?}", row_data);
    

    /* SEND APPROPRIATE EVENT */
    if row_data.starts_with("-- 500 INTERNAL SERVER ERROR --") {

        error!("Python exception was raised: {row_data}");
        return build_err_event(row_json, ErrMsg::Err500);  

    } else if row_data.starts_with("-- 400 BAD REQUEST --") {

        error!("Python was unable to handle request: {row_data}");
        error!("The row data in question: {row_data:?}");
        return build_err_event(row_json, ErrMsg::Err400);  

    }
    else if row_data.starts_with("done!") {

        debug!("Event sent from DONE! block");
        // signal list completion to the client. Has no data.
        Event::default()
            .event("complete")
            .data("done!")

    } else {

        row_json.row = row_data;
        match Event::default().json_data(&row_json) {
        
            Ok(event) => {debug!("Event built successfully."); event},
            Err(e) => {
                error!("Error in serializing row JSON: {e:?}");
                error!("The row in question: {row_json:?}");

                row_json.row = String::new();
                build_err_event(row_json, ErrMsg::Err500)
            }
        }
    }
}


// First writes out query as JSON string to Python container, then
// streams data from Python container on to the client, row by row, 
// via server-sent events.
// If there are any issues, the event type returned is an "error" type, instead
// of a "message" type, and it is expected that the connection will be terminated
// by the client in such a case (the way the Python app is written, an error
// on one line means there will be an error on every line, so continuing is no use).
pub async fn convert_lb_list(list_info: Query<ListInfo>) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ServerError> {
    
    let py_cont_sock = get_env_var("PY_CONT_SOCK")?;
    let mut conn = TcpStream::connect(py_cont_sock)?;
    info!("Connection with Python container established.");

    let list_info = list_info.0;
    let stringified_json = serde_json::to_string(&list_info)?;
    conn.write_all(stringified_json.as_bytes())?;
    debug!("Request sent to Python container.");

    // first thing from Python container will be 2 bytes that hold 
    // total list length (excluding header). I am assuming that lists
    // will not exceed ~65k films.
    //
    // Type conversions here are so I can get a `usize` that can be used 
    // in stream.take().
    // Bytes are sent over in big-endian order.
    let mut list_length_buf: [u8; 2] = [0; 2];
    conn.read_exact(&mut list_length_buf)?;
    let list_length_u16 = u16::from_be_bytes(list_length_buf);

    let total_rows = usize::from(list_length_u16) + 2;  // +1 for header, +1 to read "done!" signal
    debug!("Total row length received: {:?}", total_rows);

    let mut curr_row: u16 = 0;
    let row_stream = stream::repeat_with(
        move || {
            // when the "done!" signal is received, `curr_row` == total_rows+1,
            // but the id field of the "complete" event will be ignored 
            // by the client-side JS.
            curr_row += 1;
            let list_row = ListRow {
                curr_row, 
                total_rows, 
                row: String::new()
            };
            get_list_row(&mut conn, list_row)          
        })
        .map(Ok);

    // connection closed when `conn` is dropped
    
    Ok(
        Sse::new(
            row_stream.take(total_rows)
        )
    )
}