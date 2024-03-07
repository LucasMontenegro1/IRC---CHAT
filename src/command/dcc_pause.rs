use crate::{
    channel::Channel,
    dcc::command::pause::DccPause,
    error::error_server::ErrorServer,
    parser::dcc_message::DccMessage,
    reply::Reply,
    repository::{
        connection::Connection, repository_channel::client_channel::ClientChannel,
        traits::operations::Operations,
    },
    utils::write_message_to,
};

///
/// struct that implements a dcc chat message
/// according to what is established by the
/// IRC protocol
///
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DccPauseMessage {
    msg: DccMessage,
    from: String,
    to: String,
    ip: String,
    port: String,
    filename: String,
}

impl DccPauseMessage {
    pub fn new(msg: &DccMessage) -> Result<Self, ErrorServer> {
        let from = match msg.prefix() {
            Some(u) => u,
            None => return Err(ErrorServer::UnknownCommand),
        };

        let to = match msg.target_user() {
            Some(u) => u,
            None => return Err(ErrorServer::UnknownCommand),
        };

        let ip = Self::get_ip(msg.clone())?;
        let port = Self::get_port(msg.clone())?;
        let filename = Self::get_filename(msg.clone())?;

        Ok(DccPauseMessage {
            msg: msg.clone(),
            from,
            to,
            filename,
            port,
            ip,
        })
    }

    fn get_ip(msg: DccMessage) -> Result<String, ErrorServer> {
        let params = msg.parameters();
        match params {
            Some(parameters) => match parameters.get(1) {
                None => Ok(String::new()),
                Some(str) => Ok(str.to_owned()),
            },
            None => Ok(String::new()),
        }
    }

    fn get_port(msg: DccMessage) -> Result<String, ErrorServer> {
        let params = msg.parameters();
        match params {
            Some(parameters) => match parameters.get(2) {
                None => Ok(String::new()),
                Some(str) => Ok(str.to_owned()),
            },
            None => Ok(String::new()),
        }
    }

    fn get_filename(msg: DccMessage) -> Result<String, ErrorServer> {
        let params = msg.parameters();
        match params {
            Some(parameters) => match parameters.first() {
                None => Ok(String::new()),
                Some(str) => Ok(str.to_owned()),
            },
            None => Ok(String::new()),
        }
    }

    pub fn response(
        self,
        nick_sender: &ClientChannel<String, Connection>,
        channel_sender: &ClientChannel<String, Channel>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        let _ = channel_sender;
        // let oper = match nick_sender.search(self.from.clone())? {
        //     Some(oper) => oper.is_op_connection(),
        //     None => return Err(ErrorServer::UnreachableClient),
        // };
        self.process_receiver(nick_sender, self.to.to_string(), None)?;
        Ok(vec![Reply::rpl_none()])
    }

    fn process_receiver(
        &self,
        nick_sender: &ClientChannel<String, Connection>,
        receiver: String,
        channel: Option<String>,
    ) -> Result<Reply, ErrorServer> {
        if self.from == receiver {
            return Ok(Reply::rpl_none());
        }
        let reply = match nick_sender.search(receiver.clone())? {
            Some(mut c) => match channel {
                None => {
                    let away = c.get_away_msg();
                    if c.see_if_clonable().is_some() {
                        self.send_message_to(&mut c, channel, away)?
                    } else {
                        Reply::rpl_none()
                    }
                }
                Some(_) => self.send_message_to(&mut c, channel, None)?,
            },
            None => Reply::err_no_such_nickname(None, vec![receiver]),
        };

        Ok(reply)
    }

    fn send_message_to(
        &self,
        client: &mut Connection,
        channel: Option<String>,
        away_msg: Option<String>,
    ) -> Result<Reply, ErrorServer> {
        let _ = channel;
        let chat = DccPause::new(self.msg.clone())?;
        let message = self.format_message(chat);

        println!("{:?}", message);

        if let Err(_e) = write_message_to(&message, client) {
            //Si es error puede que sea porque somos el receptor o porque no tenemos conexion directa con el usuario.
        }

        //Away Response
        if let Some(msg) = away_msg {
            return Ok(Reply::rpl_away(client.get_nickname(), msg));
        };

        Ok(Reply::rpl_none())
    }

    pub fn get_interested_servers(
        &self,
        nick_sender: &ClientChannel<String, Connection>,
        channel_sender: &ClientChannel<String, Channel>,
    ) -> Result<Vec<String>, ErrorServer> {
        let mut servers: Vec<String> = vec![];

        if self.to.starts_with('#') || self.to.starts_with('&') {
            let channel = match channel_sender.search(self.to.to_owned())? {
                Some(c) => c,
                None => return Ok(vec![]),
            };
            let ss = channel
                .return_members()
                .into_iter()
                .filter_map(|u| nick_sender.search(u).ok())
                .flatten()
                .collect::<Vec<Connection>>();

            for s in ss {
                servers.push(match s.get_servername() {
                    Some(s) => s.to_string(),
                    None => continue,
                })
            }

            servers.sort();
            servers.dedup();
        } else {
            let user = match nick_sender.search(self.to.to_owned())? {
                Some(u) => u,
                None => return Ok(vec![]),
            };
            servers.push(user.get_servername().unwrap().to_string())
        }
        Ok(servers)
    }

    pub fn get_origin_server(
        &self,
        nick_sender: &ClientChannel<String, Connection>,
    ) -> Result<String, ErrorServer> {
        if let Some(c) = nick_sender.search(self.from.to_owned())? {
            if let Some(name) = c.get_servername() {
                Ok(name.to_owned())
            } else {
                Err(ErrorServer::UnreachableClient)
            }
        } else {
            Err(ErrorServer::UnreachableClient)
        }
    }

    pub fn format_message(&self, msg: DccPause) -> String {
        let _ = msg;
        format!(
            ":{} PRIVMSG {} dcc pause {} {} {}",
            self.from, self.to, self.filename, self.ip, self.port
        )
    }
}
