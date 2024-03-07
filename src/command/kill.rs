use std::{
    io::Write,
    net::Shutdown,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    error::error_server::ErrorServer,
    parser::message::Message,
    reply::Reply,
    repository::{
        connection::Connection, repository_channel::client_channel::ClientChannel,
        traits::operations::Operations,
    },
    server_comunication::spanning_tree::SpanningTree,
    utils::write_message_to,
};

pub struct Kill {
    user: String,
    oper: bool,
}

impl Kill {
    pub fn new(msg: Message, oper: bool) -> Result<Self, ErrorServer> {
        Ok(Kill {
            user: Self::get_user_from_msg(msg)?,
            oper,
        })
    }

    pub fn response(
        self,
        nick_sender: ClientChannel<String, Connection>,
        spanning_tree: Arc<Mutex<SpanningTree>>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        println!("paso");
        if !self.oper {
            return Ok(vec![Reply::err_no_privileges(None)]);
        }
        if let Some(conn) = nick_sender.search(self.user.clone())? {
            nick_sender.delete(self.user.clone())?;
            match spanning_tree.lock() {
                Ok(st) => {
                    if let Some(servername) = conn.get_servername() {
                        if *servername == st.get_root().server.servername {
                            let user_reply =
                                Reply::err_nick_collision(None, vec![(self.user.clone())]);

                            if let Some(mut tcp) = conn.see_if_clonable() {
                                tcp.write_all(user_reply.to_string().as_bytes())?;
                                thread::sleep(Duration::from_millis(200));
                                tcp.shutdown(Shutdown::Both)?;
                            }
                        }
                        // mando un kill a todos los servers de los que tengo conn
                        let msg = Message::new(None, super::Command::Kill, Some(vec![self.user]));
                        for server in st.get_servers() {
                            if let Some(mut s_conn) = server.get_connection() {
                                write_message_to(&msg.to_string(), &mut s_conn)?;
                            }
                        }
                    }
                }
                Err(_) => return Err(ErrorServer::LockedResource),
            }
            let reply = Reply::rpl_none();
            Ok(vec![reply])
        } else {
            let reply = Reply::err_no_such_nickname(None, vec![self.user]);
            Ok(vec![reply])
        }
    }

    fn get_user_from_msg(msg: Message) -> Result<String, ErrorServer> {
        match msg.parameters() {
            Some(c) => match c.first() {
                Some(d) => Ok(d.clone()),
                None => Err(ErrorServer::UnknownCommand),
            },
            None => Err(ErrorServer::UnknownCommand),
        }
    }
}

// nick_sender: ClientChannel<String, Connection>,
// spanning_tree: Arc<Mutex<SpanningTree>>,
