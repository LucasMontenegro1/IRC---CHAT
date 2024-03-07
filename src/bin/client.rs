use irc_project::client::run_client_console;
use irc_project::error::error_client::ErrorClient;
use std::env;
use std::io::{stdin, stdout};
use std::net::TcpStream;
use std::sync::{mpsc, Arc};

fn main() -> Result<(), ErrorClient> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(ErrorClient::UnacceptedClient);
    }

    let read_stream = Arc::new(TcpStream::connect(args[1].as_str())?);
    println!(
        "Connection established with address: {}",
        read_stream.local_addr()?
    );

    let (tx, rx) = mpsc::channel::<String>();
    run_client_console(read_stream, Box::new(stdin()), Box::new(stdout()), tx, rx)
}
