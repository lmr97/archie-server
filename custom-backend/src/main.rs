use std::{
    env,
    error::Error as StdError,
    fs,
    io::prelude::*,
    net::{TcpListener, TcpStream}, 
    path::Path,
    sync::Arc
};

use rustls::{
    pki_types::{ 
        pem::PemObject, 
        CertificateDer, 
        PrivateKeyDer, 
    },
    ServerConnection
};

// The core of this is borrowed straight from 
// the Rust Book here: https://doc.rust-lang.org/book/ch20-01-single-threaded.html


fn main() -> Result<(), Box<dyn StdError>> {

    println!("Loading certificates and keys...");
    let mut configed_server = config_auth()
        .expect("Server could not be configured with TLS.");
    println!("Certificates and keys loaded!");

    let listener = TcpListener::bind("0.0.0.0:443").unwrap();

    for stream_res in listener.incoming() {
        
        let mut stream = stream_res.unwrap();
        // complete handshake
        println!("Establishing secure connection...");
        match configed_server.complete_io(&mut stream) {
            Ok(_bytes_written) => (),
            Err(e) => {
                println!("The following error occured with this connection:\n\t{e:?}");
                println!("\nListening for the next connection..."); 
                continue;
            }
        }
        println!("Secure connection established.");

        let mut tls_stream: rustls::Stream<'_, ServerConnection, TcpStream> 
            = rustls::Stream::new(
            &mut configed_server,
            &mut stream
        );

        let mut buf = Vec::<u8>::new();
        tls_stream.read_to_end(&mut buf).unwrap();
        let request = String::from_utf8(buf).unwrap();

        respond(request, tls_stream);
        println!("Response sent!");
    }

    Ok(())
}


fn config_auth() -> Result<ServerConnection, rustls::Error> {

    // Modified version of simpleserver.rs example from Rustls docs
    // load in certs from environment filepaths
    let cert_file = env::var_os("CRT_FILE")
        .expect("Certificates filepath variable not found in environment.")
        .into_string()
        .unwrap();
    let private_key_file = env::var_os("PK_FILE")
        .expect("Private keys filepath variable not found in environment.")
        .into_string()
        .unwrap();

    let certs: Vec<CertificateDer> = CertificateDer::pem_file_iter(&cert_file)
        .expect(
            &format!(
                "Certificate file not found at {} (or other file issue).", 
                &cert_file
            )
        )
        .map(|cert| cert.expect("Error in reading a certificate."))
        .collect();

    let private_key = PrivateKeyDer::from_pem_file(&private_key_file)
        .expect(
            &format!(
                "Private key file not found at {} (or other file issue).", 
                private_key_file
            )
        );
    
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, private_key)?;
    
    rustls::ServerConnection::new(Arc::new(config))
}


// stream variable is needed to pass along to serve_*() functions
fn respond(request_str: String, stream: rustls::Stream<'_, ServerConnection, TcpStream>) {

    let request = request_str
        .split("\n")
        .collect::<Vec<&str>>();

    // this chunk is for fun, to see what kind of user agents are 
    // viewing my website!
    if let Some(user_agent) = request
        .iter()
        .filter(|el| el.starts_with("User-Agent"))
        .next()     
    {
        println!("Processing request from user agent: {}...", user_agent);
    } else {
        println!("Processing request from unspecified user agent...")
    }

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


fn serve_html(mut stream: rustls::Stream<'_, ServerConnection, TcpStream>, status_line: &str, filepath: String) {

    let contents = fs::read_to_string(filepath).unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}


// special case of serve_html() used frequently enough
// to warrant its own wrapper function
fn serve_404(stream: rustls::Stream<'_, ServerConnection, TcpStream>) {
    serve_html(
        stream, 
        "HTTP/1.1 404 Not Found", 
        String::from("/home/martin/archie-server/errors/404.html")
    );
}

fn serve_image(mut stream: rustls::Stream<'_, ServerConnection, TcpStream>, filepath: String) {
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