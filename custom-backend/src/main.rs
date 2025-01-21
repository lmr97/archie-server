use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}, path::Path,
};

// The core of this is borrowed straight from 
// the Rust Book here: https://doc.rust-lang.org/book/ch20-01-single-threaded.html


fn main() {

    // binding to "all available" IPs (when only one is actually available on the server)
    // for privacy. If the IP the URL is discovered, the device would be exposed
    // if that info is paired with the private IP
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

// stream variable is needed to pass along to serve_*() functions
fn respond(request: Vec<String>, stream: TcpStream) {
    let mut filepath = String::from("/home/martin/archie-server");
    let mut parsed_line0 = request[0].split_ascii_whitespace();
    let method = parsed_line0.next().unwrap(); 

    match method {
        "GET" => {
            let request_path = parsed_line0.next().unwrap();

            if request_path == "/" {
                filepath.push_str("/home.html");
                serve_html(stream, "HTTP/1.1 200 OK", filepath);

                return;  // no need to go further, esp. for most common case
            }

            let request_path_root = request_path
                                                .split("/")
                                                .nth(1) // path starts AFTER /, so first element null
                                                .unwrap();

            // using this pattern in case there are other types of data to be served;
            // it will make them easy to add
            match request_path_root {
                "images" => {
                    filepath.push_str(request_path);
                    serve_image(stream, filepath)
                }
                _ => {
                    serve_404(stream);
                }
            }
        }
        _ => {
            filepath.push_str("/errors/405.html");
            serve_html(stream, "HTTP/1.1 405 Method Not Allowed", filepath);
        }
    };
}


fn serve_html(mut stream: TcpStream, status_line: &str, filepath: String) {

    let contents = fs::read_to_string(filepath).unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}


// special case of serve_html() used frequently enough
// to warant its own wrapper function
fn serve_404(stream: TcpStream) {
    serve_html(
        stream, 
        "HTTP/1.1 404 Not Found", 
        String::from("/home/martin/archie-server/errors/404.html")
    );
}

fn serve_image(mut stream: TcpStream, filepath: String) {
    let status_line = "HTTP/1.1 200 OK";
    
    if !(Path::new(&filepath).exists()) {
        serve_404(stream);
        return;
    }

    // most of this below, aside form the MIME type checking,  is from this Reddit post:
    // https://www.reddit.com/r/learnrust/comments/nt1yec/chapter_20_web_server_project_help_how_do_i_serve/

    // assuming the only . is before the file extension
    // this is probably okay, since the vast majority of these image requests
    // will be from <img> tags, hence they will be for files/paths I personally
    // write, and can ensure they only include a . just before the extension.
    let img_type = filepath.split(".").nth(1).unwrap();
    let content_type = match img_type {
        "png" => "image/png",
        "jpg" => "image/jpeg",
        _     => {
            serve_html(
                stream, 
                "HTTP/1.1 415 Unsupported Media Type", 
                String::from("/home/martin/archie-server/errors/415.html")
            );
            return;
        }
    };

    let contents = fs::read(filepath).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n",
        status_line,
        contents.len(),
        content_type
    );

    stream.write(response.as_bytes()).unwrap();
    stream.write(&contents).unwrap();
}