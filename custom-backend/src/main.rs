use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

// The core of this is borrowed straight from 
// the Rust Book here: https://doc.rust-lang.org/book/ch20-01-single-threaded.html

enum FileType {
    HTML,
    image,
}

fn main() {
    
    let listener = TcpListener::bind("0.0.0.0:80").unwrap();
    
    for stream in listener.incoming() {

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

    let (status_line, filename, filetype) = match method {
        "GET" => {
            let request_path = parsed_line0.next().unwrap();
            
            match request_path {
                "/"      => {
                    filepath.push_str("/home.html");
                    ("HTTP/1.1 200 OK", filepath, FileType::HTML)
                }
                // "/stats" => {
                //     filepath.push_str("/stats.html");
                //     ("HTTP/1.1 200 OK", filepath)
                // }
                "/data/images/the-server.jpg" => {
                    filepath.push_str(request_path);
                    ("HTTP/1.1 200 OK", filepath, FileType::image)
                }
                "/data/images/arch-logo.png" => {
                    filepath.push_str(request_path);
                    ("HTTP/1.1 200 OK", filepath, FileType::image)
                }
                _ => {
                    filepath.push_str("/errors/404.html");
                    ("HTTP/1.1 404 Not Found", filepath, FileType::HTML)
                }
            }
        }
        _ => {
            filepath.push_str("/errors/405.html");
            ("HTTP/1.1 405 Method Not Allowed", filepath, FileType::HTML)
        }
    };

    let response_raw = match filetype {
        FileType::HTML => {
            let contents = fs::read_to_string(filename).unwrap();
            let length = contents.len();

            let response =
                format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

            response.as_bytes()
        }
        FileType::image => {
            let contents = fs::read(filename).unwrap();
            contents.as_slice()
        }
    };
    
    stream.write_all(response_raw).unwrap();
}
