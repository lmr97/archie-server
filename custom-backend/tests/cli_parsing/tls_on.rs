use custom_backend::utils::init_utils::process_cli_args;

fn main() {

    let procd = process_cli_args().unwrap();
    assert_eq!(procd, Some(false));
}