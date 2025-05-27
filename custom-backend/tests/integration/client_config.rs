use std::{fs::File, io::Read};
use reqwest::{self, Certificate, header};
use custom_backend::utils::init_utils::{
    get_env_var, 
    process_cli_args, 
    RunMode
};

pub fn get_base_url() -> (String, String) {

    /* Sort out protocol to use */
    let protocol = match process_cli_args().unwrap() {
        RunMode::NoTls => "http",
        _ => "https"
    }.to_string();

    let domain = get_env_var("CLIENT_SOCKET").unwrap();

    (protocol.clone(), format!("{protocol}://{domain}"))
}

pub fn config_client(protocol: String) -> reqwest::Client {

    let mut cont_type_header = header::HeaderMap::new();

    cont_type_header.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_str("application/json").unwrap()
    );

    let client_base = reqwest::Client::builder()
        .default_headers(cont_type_header);

    if protocol == "https" {
        
        let client_pk_file = get_env_var("CLIENT_CRT_FILE").unwrap();
        let mut pk_buf = Vec::new();
        File::open(client_pk_file)
            .unwrap()
            .read_to_end(&mut pk_buf)
            .unwrap();
        
        let cert = Certificate::from_pem(&pk_buf).unwrap();
        client_base.add_root_certificate(cert).build().unwrap()
    
    } else {
        client_base.build().unwrap()
    }
}
