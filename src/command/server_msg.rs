use std::sync::{Arc, Mutex};

use crate::{
    command::Command,
    error::error_server::ErrorServer,
    parser::message::Message,
    reply::Reply,
    server_comunication::{
        info_sender::send_to_all_servers,
        server::Server,
        spanning_tree::{edge::Edge, node::Node, SpanningTree},
    },
};

pub struct ServerMsg {
    from: String,
    pub servername: String,
    hopcount: usize,
}

impl ServerMsg {
    pub fn new(msg: Message) -> Result<Self, ErrorServer> {
        let from = match msg.prefix() {
            Some(u) => u,
            None => return Err(ErrorServer::UnknownCommand),
        };
        Ok(ServerMsg {
            from,
            servername: get_servername_from_msg(&msg)?,
            hopcount: get_hopcount_from_msg(&msg)?,
        })
    }

    pub fn response(
        self,
        spanning_tree: Arc<Mutex<SpanningTree>>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        let mut replies = vec![];
        match spanning_tree.lock() {
            Ok(mut st) => {
                let server_to_connect = st.search(self.from.clone());
                match server_to_connect {
                    Some(actual_server) => {
                        let new_server = Server::new(self.servername.clone(), None);
                        if st.search(self.servername.clone()).is_none() {
                            //Si no lo tiene, lo agrego sin conexión
                            let node = Node::new(new_server.clone());
                            st.add_edge(Edge::new(Node::new(actual_server), node, self.hopcount));
                            println!("NEW SERVER KNOWN: {:?}", new_server.servername);
                        } else {
                            replies.push(Reply::err_already_registered(None));
                        }
                        let mut prefix = String::from(":");
                        prefix.push_str(&self.from);
                        let message = Message::new(
                            Some(prefix),
                            Command::Server,
                            Some(vec![self.servername, (self.hopcount + 1).to_string()]),
                        );
                        //Le aviso a mis conocidos (de los que tengo conexión) que hay un nuevo nodo.
                        send_to_all_servers(&st, message, &self.from)?;
                    }
                    None => return Ok(vec![Reply::rpl_none()]),
                }
            }
            Err(_) => return Err(ErrorServer::LockedResource),
        }

        Ok(replies)
    }
}

fn get_servername_from_msg(msg: &Message) -> Result<String, ErrorServer> {
    let mut server = String::new();
    if let Some(parameters) = msg.parameters() {
        if let Some(str) = parameters.first() {
            if !str.starts_with(':') {
                server.push_str(str.as_str());
            }
        }
    }
    Ok(server)
}

fn get_hopcount_from_msg(msg: &Message) -> Result<usize, ErrorServer> {
    let hopcount;
    if let Some(parameters) = msg.parameters() {
        if let Some(str) = parameters.get(1) {
            hopcount = str.clone();
            let result = hopcount.parse::<usize>().unwrap();
            return Ok(result);
        }
    }
    println!("Error en hopcount");
    Err(ErrorServer::UnknownCommand)
}
