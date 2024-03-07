use std::{
    io::Write,
    net::Shutdown,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    parser::message::Message,
    reply::Reply,
    repository::{
        connection::Connection, repository_channel::client_channel::ClientChannel,
        traits::operations::Operations,
    },
    server_comunication::spanning_tree::SpanningTree,
    user::User,
    utils::write_message_to,
};

pub struct UserMsg {
    username: String,
    hostname: String,
    servername: String,
    realname: String,
    nickname: String,
    msg: Message,
}

impl UserMsg {
    pub fn new(msg: Message) -> Self {
        UserMsg {
            username: get_username(msg.clone()),
            hostname: get_hostname(msg.clone()),
            servername: get_servername(msg.clone()),
            realname: get_realname(msg.clone()),
            nickname: get_nickname(msg.clone()),
            msg,
        }
    }
    pub fn response(
        self,
        nick_sender: ClientChannel<String, Connection>,
        spanning_tree: Arc<Mutex<SpanningTree>>,
    ) {
        //falta hacer los checkeos de colisiones
        let user = User::new(
            self.nickname.as_str(),
            self.username.as_str(),
            self.hostname.as_str(),
            self.servername.as_str(),
            self.realname.as_str(),
            "",
        );

        let connection = Connection::connection_away_server(user);

        let added = nick_sender.add(self.nickname.clone(), connection).unwrap();
        // spanning tree se lo pasa al resto del arbol
        if added {
            if let Ok(st) = spanning_tree.lock() {
                if let Some(excluded) = st.look_for_nearest_connection(self.servername) {
                    let servers = st.get_servers();
                    for server in servers {
                        if let Some(mut conn) = server.get_connection() {
                            if server != excluded {
                                write_message_to(&self.msg, &mut conn).unwrap();
                            }
                        }
                    }
                }
            }
        } else {
            //manda el kill a todos excepto el subarbol de donde viene el otro
            if let Some(conn) = nick_sender.search(self.nickname.clone()).unwrap() {
                nick_sender.delete(self.nickname.clone()).unwrap();
                if let Ok(st) = spanning_tree.lock() {
                    if let Some(servername) = conn.get_servername() {
                        if *servername == st.get_root().server.servername {
                            let user_reply =
                                Reply::err_erroneus_nickname(None, vec![(self.nickname.clone())]);
                            if let Some(mut tcp) = conn.see_if_clonable() {
                                tcp.write_all(user_reply.to_string().as_bytes()).unwrap();
                                thread::sleep(Duration::from_millis(200));
                                tcp.shutdown(Shutdown::Both).unwrap();
                            }
                        }
                    };
                    if let Some(excluded) = st.look_for_nearest_connection(self.servername) {
                        let servers = st.get_servers();
                        for server in servers {
                            if let Some(mut conn) = server.get_connection() {
                                if server != excluded {
                                    let msg = Message::new(
                                        None,
                                        super::Command::Kill,
                                        Some(vec![self.nickname.clone()]),
                                    );
                                    write_message_to(&msg.to_string(), &mut conn).unwrap();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn get_nickname(msg: Message) -> String {
    if let Some(nick) = msg.prefix() {
        return nick;
    }
    " ".to_string()
}

pub fn get_username(msg: Message) -> String {
    if let Some(parameter) = msg.parameters() {
        if let Some(username) = parameter.first() {
            return username.clone();
        }
    }
    " ".to_string()
}

pub fn get_realname(msg: Message) -> String {
    if let Some(parameter) = msg.parameters() {
        if let Some(realname) = parameter.get(3) {
            return realname.clone();
        }
    }
    " ".to_string()
}

pub fn get_hostname(msg: Message) -> String {
    if let Some(parameter) = msg.parameters() {
        if let Some(hostname) = parameter.get(1) {
            return hostname.clone();
        }
    }
    " ".to_string()
}
pub fn get_servername(msg: Message) -> String {
    if let Some(parameter) = msg.parameters() {
        if let Some(servername) = parameter.get(2) {
            return servername.clone();
        }
    }
    " ".to_string()
}
