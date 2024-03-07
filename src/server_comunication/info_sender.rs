use std::sync::MutexGuard;

use super::spanning_tree::SpanningTree;
use crate::repository::traits::operations::Operations;
use crate::server_comunication::ClientChannel;
use crate::server_comunication::Connection;
use crate::{error::error_server::ErrorServer, parser::message::Message, utils::write_message_to};
use std::sync::Arc;
use std::sync::Mutex;

pub fn send_to_all_servers(
    st: &MutexGuard<SpanningTree>,
    message: Message,
    origin_server: &str,
) -> Result<(), ErrorServer> {
    let mut servers = st.get_servers();
    //println!("----------------------------------------------");
    //println!("SERVER ORIGEN {:?}", origin_server);
    let informer = match st.look_for_nearest_connection(origin_server.to_owned()) {
        Some(s) => s,
        None => return Ok(()), //porque no conozco el origen.
    };
    //println!("SERVER PASAMSG {:?}", informer.servername);
    for server in &mut servers {
        if &informer != server {
            println!(
                "{:?} --> {:?} : {:?}",
                informer.servername,
                server.servername,
                message.to_string()
            );
            if write_message_to(&message, server).is_err() {
                // No puede enviar informción porque no es vecino o el destinatario es él mismo
                //println!(
                  //" (!) - Server doesn't have a direct connection with server: {}",
                    //server.servername
                //)
            };
        }
    }
    //println!("----------------------------------------------");
    Ok(())
}

// GENERA LOOPS SI EL SERVIDOR NO SE CAE.
pub fn inform_all_servers(
    spanning_tree: &Arc<Mutex<SpanningTree>>,
    nick_sender: &ClientChannel<String, Connection>,
    nickname: &str,
    msg: Message,
) -> Result<(), ErrorServer> {
    match spanning_tree.lock() {
        Ok(st) => {
            if let Some(connection) = nick_sender.search(nickname.to_string())? {
                send_to_all_servers(&st, msg, connection.get_servername().unwrap())
            } else {
                send_to_all_servers(&st, msg, &st.get_root().server.servername)
            }
        }
        Err(_) => Err(ErrorServer::LockedResource),
    }
}

pub fn inform_all_server_an_user_command(
    spanning_tree: &Arc<Mutex<SpanningTree>>,
    nick_sender: &ClientChannel<String, Connection>,
    nickname: &str,
    msg: Message,
) -> Result<(), ErrorServer> {
    match spanning_tree.lock() {
        Ok(st) => {
            if let Some(connection) = nick_sender.search(nickname.to_string())? {
                send_to_all_servers(&st, msg, connection.get_servername().unwrap())
            } else {
                Err(ErrorServer::UnreachableClient)
            }
        }
        Err(_) => Err(ErrorServer::LockedResource),
    }
}
