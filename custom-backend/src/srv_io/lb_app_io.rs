/* This file is the interface to a Python app I made to
collect the data from a given film list on letterboxd.com,
formatting it into a CSV file.

The original program had a CLI, so instead of opening the server
up to injections onto its command line, I reworked the program 
to take a byte stream from a socket as its input. This file 
handles the I/O to/from the Python program, streaming out 
its output to the client. */

use std::convert::Infallible;
use std::env::VarError;
use std::io::{Read, Write};
use std::net::TcpStream;
use futures_util::{StreamExt, stream::{self, Stream}};
use axum::response::{IntoResponse, Sse, sse::Event, Response};
use axum_extra::extract::Query;
use mysql_common::serde_json;
use tracing::{debug, error};

use crate::utils::err_handling::make_500_resp;
use crate::utils::init_utils::get_env_var;
use crate::types::lb_app_types::{ListInfo, ListRow};

// Error type for all pre-stream errors in this file,
// i.e., the ones that occur during the *start* of the 
// streaming process, since these are the ones that can/should be
// converted into stand-alone HTTP responses. The errors that occur 
// in during the stream must be handled in the stream directly, 
// since JavaScript handles server-sent events, and is best-practice 
// to my knowledge. In-stream errors are handled through the 
// `receive_list_row` and `build_server_err_event` functions.
#[derive(Debug)]
pub enum LbConvError {
    EnvVarError(VarError),
    ContainerIoError(std::io::Error),
    JsonParseError(serde_json::Error),
}

impl From<VarError> for LbConvError {
    fn from(ve: VarError) -> Self {
        Self::EnvVarError(ve)
    }
}

impl From<std::io::Error> for LbConvError {
    fn from(ioe: std::io::Error) -> Self {
        Self::ContainerIoError(ioe)
    }
}

impl From<serde_json::Error> for LbConvError {
    fn from(jspe: serde_json::Error) -> Self {
        Self::JsonParseError(jspe)
    }
}

impl IntoResponse for LbConvError {
    fn into_response(self) -> Response {
        
        error!("Error occured prior to stream start: {:?}", self);
        make_500_resp()
    }
}



// currently only sends 500 errors; others are handled either
// by build_python_app_err_event() or the Axum framework itself
// I'm defining this one-liner here because it appears so frequently
fn build_server_err_event() -> Event {

    Event::default()
        .event("error")
        .data("500 INTERNAL SERVER ERROR")
}


// pulling this out into a function for proper logging logic
fn build_python_app_err_event(err_msg: String) -> Event {

    // can't do a match statment here, since I can only reliably
    // check the beginning of the error string, and that requires
    // more logic than a match typically takes
    if err_msg.starts_with("-- 500 INTERNAL SERVER ERROR --") {

        error!("Python exception was raised: {err_msg}"); 
    } 
    else if err_msg.starts_with("-- 502 BAD GATEWAY --") {

        error!("Letterboxd server down: {err_msg}");
    } 
    else if err_msg.starts_with("-- 422 UNPROCESSABLE CONTENT --") {

        error!("Python was unable to handle request: {err_msg}");
        error!("The row data in question: {err_msg:?}");  
    }
    else if err_msg.starts_with("-- 403 FORBIDDEN --") {

        error!("The list requested was too long: {err_msg}");
    }

    Event::default()
        .event("error")
        .data(err_msg)
}


// Gets a row of the CSV data streamed from the Python container,
// converts it to a String, then packages it up as an `Event`
// and returns the `Event`.It returns an event of type "complete" 
// when the conversion is done.
//
// The `Event` is packaged up with the total rows. Sending the total rows 
// every time is a little bit of overhead, but it helps simplify the code here,
// keeping me from having an `Sse` struct with a `Stream` of two different 
// types somehow concatenated together.
//
// To make the Event, this function first reads 2 bytes, which indicate 
// the length of the row (in bytes), and then it reads an amount of bytes 
// equal to the number comprised of the first 2 bytes received.
// 
// Will return an event of type "error" if there are any issues reading
// from the Python container, or converting the output to a UTF-8 string.
fn receive_list_row(conn: &mut TcpStream, total_rows: usize) -> Event {

    let mut row_length_buf = [0; 2];
    let mut row_json = ListRow {
        total_rows: total_rows - 1,         // exclude "done!" signal addition
        row_data: String::new()
    };
    
    /* READ ROW LENGTH BYTES */

    // Any read errors need to be manually handled right here;
    // they cannot be passed up through the callers, and I'd prefer
    // to not crash the server with an unwrap()
    //
    // After the emission of an error event, the connection will be 
    // terminated by the client
    debug!("Listening for row size-bytes...");
    match conn.read_exact(&mut row_length_buf) {
        // the kind of error encountered here is usually that
        // the buffer was not filled, which ends up working out. 
        // Catching any other errors here and logging them
        // in case new ones arise.
        Ok(_) => {},
        Err(e) => {
            error!("Error on read of row size-bytes: {e:?}");
            
            // if the buffer is completely null, however,
            // we do have a problem
            if row_length_buf == [0, 0] {
                return build_server_err_event();
            }
        }
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
            return build_server_err_event();    
        }         
    };

    debug!("{row_data_buf:?}");


    /* CONVERT ROW BYTES TO STRING */
    let Ok(row_data) = String::from_utf8(row_data_buf) 
    else {
        error!("Conversion Error: bytes read from Python container could not be converted into a (UTF-8) string.");
        error!("Run on Debug mode to see bytes read.");
        return build_server_err_event();  
    };
    debug!("Indiv. row data received: {:?}", row_data);
    

    /* SEND APPROPRIATE EVENT */
    // all error messages sent from the Python app start with a double-hyphen
    if  row_data.starts_with("--") {

        return build_python_app_err_event(row_data);
    }
    else if row_data.starts_with("done!") {

        debug!("Event sent from DONE! block");
        // signal list completion to the client. Has no data.
        Event::default()
            .event("complete")
            .data("done!")
    } 
    else {
        row_json.row_data = row_data;
        
        match Event::default().json_data(&row_json) {
        
            Ok(event) => {debug!("Event built successfully."); event},
            Err(e) => {
                error!("Error in serializing row JSON: {e:?}");
                error!("The row in question: {row_json:?}");

                row_json.row_data = String::new();
                build_server_err_event()
            }
        }
    }
}


// First writes out query as JSON string to Python container, then
// streams data from Python container on to the client, row by row, 
// via server-sent events.
//
// If there are any issues, the event type returned is an "error" type, instead
// of a "message" type, and it is expected that the connection will be terminated
// by the client in such a case (the way the Python app is written, an error
// on one line means there will be an error on every line, so continuing is no use).
//
// This beast of a return type comes from the documentation (for axum::response::sse); 
// it's the only way I've found to get the server to compile! 
pub async fn convert_lb_list(list_info: Query<ListInfo>) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, LbConvError> {
    
    let py_cont_sock = get_env_var("PY_CONT_SOCK")?;
    let mut conn = TcpStream::connect(&py_cont_sock)?;
    debug!("Connection with Python container established at {py_cont_sock}");

    let list_info = list_info.0;
    let stringified_json = serde_json::to_string(&list_info)?;
    conn.write_all(stringified_json.as_bytes())?;
    debug!("Request sent to Python container.");

    // first thing from Python container will be 2 bytes that hold 
    // total list length (excluding header). I am assuming that lists
    // will not exceed ~65k films, but I am limiting it to 10k films
    // for simplicity.
    //
    // Type conversions here are so I can get a `usize` that can be used 
    // in stream.take().
    // Bytes are sent over in big-endian order.
    let mut list_length_buf: [u8; 2] = [0; 2];
    conn.read_exact(&mut list_length_buf)?;
    let list_length_u16 = u16::from_be_bytes(list_length_buf);

    let total_rows = usize::from(list_length_u16) + 2;  // +1 for header, +1 for "done!" signal
    debug!("Total row length received: {:?}", total_rows);

    let row_stream  = stream::repeat_with(
        move || Ok(receive_list_row(&mut conn, total_rows))
    );
    
    Ok(
        Sse::new( 
            row_stream.take(total_rows)
        )
    )


}



#[cfg(test)]
mod tests {
    
    // The general form of this unit test comes from an example on the 
    // Axum repository; and I've found it's not really feasible to test  
    // any other way, or with any more granularity. I need to simulate 
    // the server to serve `convert_lb_list`, since neither `Sse` nor 
    // `Event` structs don't allow for comparison. So the "unit" being 
    // tested here is composed of the functions in this module, using a 
    // mocked Python Letterboxd app to supply the data to test.
    // 
    // The point of these tests is to ensure that the server is getting
    // the messages from the Python app, not that these messages are 
    // correct (this is the point of the Python app tests). The Python 
    // app is tested separately, and integration-tested separately from
    // both these tests.

    use super::*;
    use std::{fs::OpenOptions, io::Error};
    use tokio::net::TcpListener;
    use axum::routing::{get, Router};
    use eventsource_stream::Eventsource;


    async fn spawn_app() -> Result<(), Error> {

        let listener = TcpListener::bind("127.0.0.1:8017")
            .await?;
        let lb_list_test = Router::new().route("/", get(convert_lb_list));
        tokio::spawn(async {
            axum::serve(listener, lb_list_test)
            .await
            .unwrap();    // if this little server can't go up here, it should crash the tests
        });
        
        Ok(())
    }

    // sends request and then gets stream as a Vec of Events
    async fn extract_events(url: String) -> Vec<eventsource_stream::Event> {

        // Since these tests run async, I don't know which one will run first,
        // so I don't know which test will need to initialize the miniserver.
        // So I'll just check in every function, and ignore connection error 
        // from spawn_app. This is because if there 
        // is an error, it means the miniserver is already up, and 
        // if there isn't, it soon will be. Anything catastrophic
        // will be caught by the unwrap inside that function and crash
        // the tests.
        match spawn_app().await {
            Ok(_) => (),
            Err(_) => ()
        };

        let mut event_stream= reqwest::Client::new()
            .get(url)
            .send()
            .await
            .unwrap()
            .bytes_stream()
            .eventsource();

        let mut event_data: Vec<eventsource_stream::Event> = vec![];
        while let Some(event) = event_stream.next().await {
            match event {
                Ok(event) => {
                    // break the loop at the end of SSE stream
                    if event.data == "done!" {
                        break;
                    }
                    event_data.push(event);
                }
                Err(_) => {
                    panic!("Error in event stream");
                }
            }
        }

        event_data
    }

    #[tokio::test]
    async fn too_long_list_req() {

        // This is the data used in the URL query, in JSON format:
        // {
        //     list_name: "list-too-long",
        //     author_user: "user_exists",
        //     attrs: ["casting", "watches", "likes"]    // all valid attributes
        // }

        let mini_svr_url= "http://127.0.0.1:8017?\
            list_name=list-too-long&\
            author_user=user_exists&\
            attrs=casting&\
            attrs=watches&\
            attrs=likes";

        let event_data = extract_events(String::from(mini_svr_url)).await;

        assert!(event_data[0].event == "error");
        assert!(event_data[0].data.contains("403 FORBIDDEN"));
    }

    // for when Letterboxd.com is down, testing sending 502 status to client
    #[tokio::test]
    async fn lb_server_down_req(){

        let mini_svr_url= "http://127.0.0.1:8017?\
            list_name=server-down&\
            author_user=some_user&\
            attrs=stuff";

        let event_data = extract_events(String::from(mini_svr_url)).await;

        assert!(event_data[0].event == "error");
        assert!(event_data[0].data.contains("502 BAD GATEWAY"));
    }

    #[tokio::test]
    async fn bad_list_req(){

        // if the user doesn't exist, the list doesn't, 
        // because it'll turn up a bad URL regardless.
        // So it only matters whether the list exists or not.

        let mini_svr_url= "http://127.0.0.1:8017?\
            list_name=list-no-exist&\
            author_user=user_may_exist&\
            attrs=casting&\
            attrs=watches&\
            attrs=likes";

        let event_data = extract_events(String::from(mini_svr_url)).await;

        assert!(event_data[0].event == "error");
        assert!(event_data[0].data.contains("422 UNPROCESSABLE CONTENT"));
    }

    #[tokio::test]
    async fn req_to_crashed_app() {

        let mini_svr_url= "http://127.0.0.1:8017?\
            list_name=this-hurts-you&\
            author_user=user_may_exist&\
            attrs=casting&\
            attrs=watches&\
            attrs=likes";

        let event_data = extract_events(String::from(mini_svr_url)).await;

        assert!(event_data[0].event == "error");
        assert!(event_data[0].data.contains("500 INTERNAL SERVER ERROR"));
    }

    #[tokio::test]
    async fn bad_attr_req() {

        let mini_svr_url= "http://127.0.0.1:8017?\
            list_name=list-exists&\
            author_user=user_exists&\
            attrs=casting&\
            attrs=bingus&\
            attrs=likes";

        let event_data = extract_events(String::from(mini_svr_url)).await;

        assert!(event_data[0].event == "error");
        assert!(event_data[0].data.contains("422 UNPROCESSABLE CONTENT"));

    }

    #[tokio::test]
    async fn no_attr_req() {

        let mini_svr_url= "http://127.0.0.1:8017?\
            list_name=list-exists&\
            author_user=user_exists&\
            attrs=none";

        let event_data = extract_events(String::from(mini_svr_url)).await;

        let correct_rows = vec![
            ListRow {
                total_rows: 5,
                row_data: String::from("Title,Year"),
            },
            ListRow {
                total_rows: 5,
                row_data: String::from("2001: A Space Odyssey,1968"),
            },
            ListRow {
                total_rows: 5,
                row_data: String::from("Blade Runner,1982"),
            },
            ListRow {
                total_rows: 5,
                row_data: String::from("The Players vs. Ángeles Caídos,1969"),
            },
            ListRow {
                total_rows: 5,
                row_data: String::from("8½,1963"),
            },
        ];

        let received_rows: Vec<ListRow> = event_data
            .iter()
            .map(| ev | { 
                serde_json::from_str::<ListRow>(&ev.data)
                    .unwrap() 
            })
            .collect();

        assert_eq!(correct_rows, received_rows);
    }

    // This exists to test very long (probably maximally long) rows,
    // with a very long list (~1.5k films)
    #[tokio::test]
    async fn big_list_req(){

        let url = "http://127.0.0.1:8017?\
            list_name=the-big-one&\
            author_user=user_exists&\
            attrs=all-of-em";

        let mut test_file_reader = OpenOptions::new()
            .read(true)
            .open("../test-helpers/big-list-test.csv")
            .unwrap();

        let mut correct_list = String::new();
        test_file_reader
            .read_to_string(&mut correct_list)
            .unwrap();

        let event_data = extract_events(String::from(url)).await;

        // Extract row data itself, not entire ListRow structs
        let received_list  = event_data
            .iter()
            .map(| ev | { 
                let li = serde_json::from_str::<ListRow>(&ev.data)
                    .unwrap();
                li.row_data
            })
            .collect::<Vec<String>>()
            .join("");

        assert_eq!(correct_list, received_list);
    }
}