use irc_project::error::error_server::ErrorServer;
use irc_project::server::init_server;
use std::env;

fn main() -> Result<(), ErrorServer> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        return Err(ErrorServer::TcpFail);
    }
    //println!("main() server initiation");
    init_server(args[1].to_string(), args[2].to_string())?;
    Ok(())
}
