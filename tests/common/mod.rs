// use irc_project::error::error_client::ErrorClient;
// use std::net::TcpStream;
// use std::process::Command;
// use std::sync::mpsc::Sender;
// use std::sync::Arc;

// pub fn init_client(tcpAddress: String) -> Result<Arc<TcpStream>, ErrorClient> {
//     Command::new("sleep").arg("10").spawn().expect("cant wait");
//     let tcp = Arc::new(TcpStream::connect(tcpAddress)?);
//     return Ok(tcp);
// }

// pub fn send_message(tx: Sender<String>, msg: String) {
//     Command::new("sleep").arg("50").spawn().expect("cant wait");
//     tx.send(msg).expect("cant send msg on tx");
// }
