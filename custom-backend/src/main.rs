use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

// Most of the core of this is borrowed straight from 
// the Rust Book here: https://doc.rust-lang.org/book/ch20-01-single-threaded.html

fn main() {
    
    let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    
    for stream in listener.incoming() {

        println!("atomics checked");
        let stream = stream.unwrap();
        
        println!("Connection established!");
        
        let request = stream_to_string(&stream);
        respond(request, stream);
        println!("Response sent!");
    }
    
}

fn stream_to_string(stream: &TcpStream) -> Vec<String> {
    let buf_reader = BufReader::new(stream);
    let http_request: Vec<String> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    http_request
}

fn respond(request: Vec<String>, mut stream: TcpStream) {
    let mut filepath = String::from("/home/martin/archie-server");
    let mut parsed_line0 = request[0].split_ascii_whitespace();
    let method = parsed_line0.next().unwrap(); 

    let (status_line, filename) = match method {
        "GET" => {
            let request_path = parsed_line0.next().unwrap();
            
            match request_path {
                "/"      => {
                    filepath.push_str("/home.html");
                    ("HTTP/1.1 200 OK", filepath)
                }
                // "/stats" => {
                //     filepath.push_str("/stats.html");
                //     ("HTTP/1.1 200 OK", filepath)
                // }
                _        => {
                    filepath.push_str("/404.html");
                    ("HTTP/1.1 404 Not Found", filepath)
                }
            }
        }
    
        _ => {
            filepath.push_str("/405.html");
            ("HTTP/1.1 405 Method Not Allowed", filepath)
        }
    };
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
