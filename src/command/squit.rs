use crate::{
    channel::Channel,
    command::quit::QuitMsg,
    error::error_server::ErrorServer,
    parser::message::Message,
    reply::Reply,
    repository::{
        connection::Connection, repository_channel::client_channel::ClientChannel,
        traits::operations::Operations,
    },
    server_comunication::spanning_tree::{edge::Edge, SpanningTree},
    utils::write_message_to,
};

use std::{
    str::FromStr,
    sync::{Arc, Mutex, MutexGuard},
};

///
/// struct that implements the
/// quit message as stipulated
/// by the irc protocol
///
pub struct SquitMsg {
    _msg: String,
    oper: String,
    server_to_delete: String,
}

impl SquitMsg {
    ///
    /// create a new message
    ///  of type quit
    ///
    pub fn new(msg: &Message) -> Result<Self, ErrorServer> {
        let oper = match msg.prefix() {
            Some(u) => u,
            None => return Err(ErrorServer::UnknownCommand),
        };
        Ok(SquitMsg {
            _msg: Self::get_msg_from_squit(oper.clone(), msg)?,
            oper,
            server_to_delete: Self::get_server_from_squit(msg)?,
        })
    }

    ///
    /// function that responds to the quit
    /// message sent by the user and
    /// acts accordingly
    ///
    pub fn response(
        &self,
        nick_sender: &ClientChannel<String, Connection>,
        channel_sender: &ClientChannel<String, Channel>,
        spanning_tree: &Arc<Mutex<SpanningTree>>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        let operator = match nick_sender.search(self.oper.to_string())? {
            Some(u) => u,
            None => return Err(ErrorServer::UnreachableClient),
        };

        let actual_server = operator.get_servername().unwrap().to_string();

        let lost_servers = self.get_lost_servers(spanning_tree);

        println!("SERVERS PERDIDOS {:?}", lost_servers);

        match spanning_tree.lock() {
            Ok(mut st) => {
                let mut msg = String::from(":");
                msg.push_str(&self.oper);
                msg.push_str(" SQUIT ");
                match st.look_for_nearest_connection(self.server_to_delete.clone()) {
                    Some(mut s) => {
                        if s.servername == self.server_to_delete {
                            msg.push_str(&st.get_root().server.servername);
                            msg.push(' ');
                            msg.push_str(&self._msg);
                            if let Some(origin) = st.look_for_nearest_connection(actual_server) {
                                if origin.servername != self.server_to_delete
                                    && write_message_to(&msg, &mut s).is_err()
                                {
                                    println!("error - writing msg from server to other server");
                                }
                            }
                        }
                    }
                    None => {
                        return Ok(vec![Reply::err_no_such_server(
                            None,
                            vec![self.server_to_delete.to_string()],
                        )])
                    }
                };

                for edge in self.get_lost_edges(&st) {
                    println!(
                        "ELIMINANDO {}->{}",
                        edge.source.get_element().servername,
                        edge.destination.get_element().servername
                    );
                    st.delete_edge(
                        edge.source.get_element().servername,
                        edge.destination.get_element().servername,
                    );
                }
            }
            Err(_) => return Err(ErrorServer::LockedResource),
        }

        self.get_quit_messages(lost_servers, nick_sender)?
            .iter()
            .filter_map(|msg| QuitMsg::new(msg).ok())
            .for_each(|quit| {
                println!("Elimino usuario {:?}", quit.user);
                if quit.response(nick_sender, channel_sender).is_err() {
                    println!("error squit - when try to quit user from channel");
                };
            });

        Ok(vec![Reply::rpl_none()])
    }

    ///
    /// function that is responsible for
    /// extracting the farewell message
    /// from the quit message
    ///
    fn get_msg_from_squit(servername: String, msg: &Message) -> Result<String, ErrorServer> {
        if let Some(parameters) = msg.parameters() {
            if let Some(message) = parameters.get(1) {
                let msg = message.to_string();
                return Ok(msg);
            };
        }
        Ok(servername)
    }

    ///
    /// function that is responsible for
    /// extracting the farewell message
    /// from the quit message
    ///
    fn get_server_from_squit(msg: &Message) -> Result<String, ErrorServer> {
        if let Some(parameters) = msg.parameters() {
            if let Some(message) = parameters.first() {
                let msg = message.to_string();
                return Ok(msg);
            };
        }
        Err(ErrorServer::ServerClosed)
    }

    fn create_quit_message(&self, nickname: &str) -> Result<Message, ErrorServer> {
        let mut msg = String::from(":");
        msg.push_str(nickname);
        msg.push_str(" QUIT :Server quit");

        Ok(Message::from_str(&msg)?)
    }

    fn get_quit_messages(
        &self,
        lost_server: Vec<String>,
        nick_sender: &ClientChannel<String, Connection>,
    ) -> Result<Vec<Message>, ErrorServer> {
        let mut quit_msgs = vec![];
        for connection in nick_sender.find_all()? {
            if lost_server.contains(&connection.get_servername().unwrap().to_string()) {
                quit_msgs.push(self.create_quit_message(&connection.get_nickname())?);
            }
        }
        Ok(quit_msgs)
    }

    fn get_lost_servers(&self, spanning_tree: &Arc<Mutex<SpanningTree>>) -> Vec<String> {
        let mut servernames = vec![self.server_to_delete.clone()];
        let edges = match spanning_tree.lock() {
            Ok(st) => self.get_lost_edges(&st),
            Err(_) => vec![],
        };
        let minimum_cost = edges
            .iter()
            .filter(|e| {
                e.source.get_element().servername == self.server_to_delete
                    || e.destination.get_element().servername == self.server_to_delete
            })
            .min_by(|e1, e2| e1.cost.cmp(&e2.cost))
            .unwrap()
            .cost;

        for e in edges {
            if e.cost != minimum_cost {
                if !servernames.contains(&e.source.get_element().servername) {
                    servernames.push(e.source.get_element().servername.clone())
                }
                if !servernames.contains(&e.destination.get_element().servername) {
                    servernames.push(e.destination.get_element().servername.clone())
                }
            }
        }
        //servernames.sort();
        //servernames.dedup();

        servernames
    }

    fn get_lost_edges(&self, st: &MutexGuard<SpanningTree>) -> Vec<Edge> {
        let minimum_cost = st
            .get_edges()
            .into_iter()
            .filter(|e| {
                e.source.get_element().servername == self.server_to_delete
                    || e.destination.get_element().servername == self.server_to_delete
            })
            .min_by(|e1, e2| e1.cost.cmp(&e2.cost))
            .unwrap()
            .cost;

        println!("COSTO MINIMO {}", minimum_cost);

        let edges: Vec<Edge> = st
            .get_edges()
            .into_iter()
            .filter(|e| {
                //Nos fijamos los que forman conexiones directas con el server
                //a eliminar y los que cumplan con el costo minimo para llegar hacia el.
                e.cost > minimum_cost
                    || e.source.get_element().servername == self.server_to_delete
                    || e.destination.get_element().servername == self.server_to_delete
            })
            .collect();
        edges
    }

    pub fn get_operator(
        &self,
        nick_sender: &ClientChannel<String, Connection>,
    ) -> Result<Connection, ErrorServer> {
        match nick_sender.search(self.oper.clone())? {
            Some(c) => Ok(c),
            None => Err(ErrorServer::UnreachableClient),
        }
    }
}
