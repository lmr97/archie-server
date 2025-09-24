use std::{
    env::{self, VarError},
    fs::OpenOptions,
    io::{Error, ErrorKind}
};
use tracing::error;
use tracing_subscriber::{
    fmt::{
        format::{DefaultFields, Format}, 
        SubscriberBuilder
    }, 
    EnvFilter,
    filter::LevelFilter
};

#[derive(Debug, PartialEq)]
pub enum RunMode {
    Tls,
    NoTls,
    PrintHelp
}

pub fn build_stdout_logger(print_prelog: bool) -> SubscriberBuilder<DefaultFields, Format, EnvFilter, fn() -> std::io::Stdout> {
    if print_prelog { println!("[ PRE-LOG ]: Initializing logger..."); }
    
    let ef = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())  // will include INFO level too
        .from_env_lossy();

    error!("stuff!");

    tracing_subscriber::fmt().with_env_filter(ef)
}

pub fn build_logger(log_file_path: String, print_prelog: bool) -> Result<SubscriberBuilder<DefaultFields, Format, EnvFilter, std::fs::File>, Error> {

    if print_prelog { println!("[ PRE-LOG ]: Loading log file at {log_file_path}..."); }

    let log_file = OpenOptions::new()
        .append(true)
        .open(log_file_path)?;

    if print_prelog { println!("[ PRE-LOG ]: Log file loaded!"); }

    if print_prelog { println!("[ PRE-LOG ]: Initializing logger..."); }
    let ef = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())  // will include INFO level too
        .from_env_lossy();
    Ok(tracing_subscriber::fmt()
        .with_env_filter(ef)
        .with_writer(log_file))
}


pub fn get_env_var(env_var: &str) -> Result<String, VarError> {

    match env::var_os(env_var) {
        Some(s) => Ok(s.into_string().unwrap()),    // unwrap cannot panic here
        None => {
            error!("Environment variable {env_var} needs to be set.");
            Err(VarError::NotPresent)
        }
    }
}

// unwraps cannot panic here either; Results don't have errors in them
// expect() may panic, but this function is only ever called in main
pub fn get_auth_paths() -> (String, String) {

    // load in certs from environment filepaths
    let cert_file = env::var_os("CRT_FILE")
        .expect("Certificates filepath variable not found in environment.")
        .into_string()
        .unwrap();
    let private_key_file = env::var_os("PK_FILE")
        .expect("Private keys filepath variable not found in environment.")
        .into_string()
        .unwrap();

    (cert_file, private_key_file)
}


fn print_help() {
    println!("\nUsage:  archie-server [OPTION]\n");
    println!("The executable that runs the server.\n");
    println!("Options:");
    println!("    --no-tls     Run without TLS; serve over HTTP");
    println!("    --help, -h   Print this help message and quit\n");
}

//arg1: Option<String>
pub fn process_cli_args() -> Result<RunMode, Error> {

    let arg1 = std::env::args().nth(1);
    match arg1 {
        Some(arg) => {
            match arg.as_str() {
                "--no-tls"      => Ok(RunMode::NoTls),
                "--help" | "-h" => {
                    print_help();
                    return Ok(RunMode::PrintHelp);
                }
                other => {
                    print_help();
                    return Err(
                        Error::new(
                            ErrorKind::InvalidInput,
                            format!("Option \"{other}\" is not recognized.")
                        )
                    );
                }
            }
        },
        None => Ok(RunMode::Tls)
    }
}



#[cfg(test)]
mod tests {

    // these tests are limited in scope, because the functions they test
    // only have the values that I provide. Users cannot provide data to
    // these functions, directly or indirectly. They're closer to sanity 
    // checks than tests
    use super::*;
    use std::{io::Error, fs::read_to_string};
    use tracing::{info, debug, warn, error};

    #[test]
    fn get_existing_env_var() {
        let value = String::from("a value/here");
        assert_eq!(get_env_var("EX_VAR"), Ok(value));
    }

    #[test]
    fn get_nonexist_env_var() {
        assert_eq!(get_env_var("NONEX_VAR"), Err(VarError::NotPresent));
    }

    #[test]
    // #[ignore]    // because we're logging to stdout, and it clashes with logging_to_stdout test
    fn logging_to_file() -> Result<(), Error> {
        let test_log_path = String::from("./test.log");
        build_logger(test_log_path.clone(), false)?.init();
        println!("[ PRE-LOG ]: Logger initialized!");

        info!("some information");
        debug!("some debugging info");
        warn!("a warning");
        error!("an error");

        let log_string = read_to_string(test_log_path)?;

        assert!(log_string.contains("some information"));
        assert!(log_string.contains("some debugging info"));
        assert!(log_string.contains("a warning"));
        assert!(log_string.contains("an error"));

        Ok(())
    }

    // since using #[test] or #[tokio::test] results in an error,
    // I use #[tracing_test::traced_test] instead. 
    // But this causes the function to look like dead code to
    // Cargo, so I'm ignoring the compiler warning here for this reason.
    #[tracing_test::traced_test]
    #[allow(dead_code)]
    async fn logging_to_stdout() {

        std::env::set_var("SERVER_LOG", "");
        build_stdout_logger(true).init();

        info!("some information");
        debug!("some debugging info");
        warn!("a warning");
        error!("an error");

        assert!(logs_contain("some information"));
        assert!(logs_contain("some debugging info"));
        assert!(logs_contain("a warning"));
        assert!(logs_contain("an error"));
    }
}