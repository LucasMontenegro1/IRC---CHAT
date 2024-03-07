use crate::{
    channel::Channel,
    error::error_server::ErrorServer,
    parser::message::Message,
    reply::Reply,
    repository::{
        connection::Connection, repository_channel::client_channel::ClientChannel,
        traits::operations::Operations,
    },
    utils::write_message_to,
};

///
/// struct that implements a private message
/// according to what is established by the
/// IRC protocol
///
///
pub struct PrivMsg {
    from: String,
    msg: String,
    to: String,
}

impl PrivMsg {
    ///
    /// function that creates a
    /// new private message
    ///
    pub fn new(msg: Message) -> Result<Vec<Self>, ErrorServer> {
        let from = match msg.prefix() {
            Some(u) => u,
            None => return Err(ErrorServer::UnknownCommand),
        };
        let receiver = Self::get_receivers_from_privmsg(&msg)?;
        let mut msgs = vec![];
        for to in receiver {
            msgs.push(PrivMsg {
                from: from.clone(),
                to: to.clone(),
                msg: Self::get_msg_from_privmsg(&msg)?,
            })
        }
        Ok(msgs)
    }

    ///
    /// function that responds to the private
    /// message sent by the user and
    /// acts accordingly
    ///
    pub fn response(
        self,
        nick_sender: &ClientChannel<String, Connection>,
        channel_sender: &ClientChannel<String, Channel>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        //ERR_NONICKNAMEGIVEN
        if self.to.is_empty() {
            return Ok(vec![Reply::err_no_nickname_given(None)]);
        }

        //ERR_NOTEXTTOSEND
        if self.msg.is_empty() {
            return Ok(vec![Reply::err_no_text_to_send(None)]);
        }

        let oper = match nick_sender.search(self.from.clone())? {
            Some(oper) => oper.is_op_connection(),
            None => return Err(ErrorServer::UnreachableClient),
        };

        let msg_receiver = self.to.clone();
        let mut replies = vec![];
        if self.from == msg_receiver {
            return Ok(vec![Reply::rpl_none()]);
        }
        if oper {
            if self.to.starts_with('$') {
                replies.append(&mut self.send_multiple_msgs(nick_sender, &self.to)?);
            } else if self.to.starts_with('#') {
                replies.push(self.send_to_hostname(nick_sender, self.to.to_string())?);
            } else {
                replies.push(self.process_receiver(nick_sender, msg_receiver, None)?);
            }
            return Ok(replies);
        }

        if msg_receiver.starts_with('&') || msg_receiver.starts_with('#') {
            let mut r = match channel_sender.search(msg_receiver.clone())? {
                Some(c) => self.send_to_channel(c, nick_sender)?,
                None => vec![Reply::err_no_such_nickname(None, vec![msg_receiver])],
            };
            replies.append(&mut r);
        } else {
            replies.push(self.process_receiver(nick_sender, msg_receiver, None)?);
        }
        Ok(replies)
    }

    fn send_to_channel(
        &self,
        c: Channel,
        nick_sender: &ClientChannel<String, Connection>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        let mut replies = vec![Reply::rpl_none()];
        if c.user_can_speak(&self.from) {
            for member in c.return_members() {
                replies.push(self.process_receiver(nick_sender, member, Some(c.name.clone()))?);
            }
        } else {
            //ERR_CANTSENDTOCHANNEL
            return Ok(vec![Reply::err_cant_send_to_channel(vec![c.get_name()])]);
        }
        Ok(replies)
    }

    ///
    /// function that is responsible for
    /// processing  who receives the given
    /// private message
    ///
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
                    if let Some(_connection) = c.see_if_clonable() {
                        self.send_message_to(&mut c, channel)?;
                    }
                    //Away Response
                    match c.get_away_msg() {
                        Some(away_msg) => Reply::rpl_away(c.get_nickname(), away_msg),
                        None => Reply::rpl_none(),
                    }
                }
                Some(_) => self.send_message_to(&mut c, channel)?,
            },
            None => Reply::err_no_such_nickname(None, vec![receiver]),
        };

        Ok(reply)
    }

    ///
    /// function that is used in case
    /// an operator of the server
    /// wants to send a message to multiple
    /// registered user using the special
    /// keys
    ///
    fn send_multiple_msgs(
        &self,
        nick_sender: &ClientChannel<String, Connection>,
        user: &String,
    ) -> Result<Vec<Reply>, ErrorServer> {
        let mut user = user.to_owned();
        user.remove(0);
        user.remove(0);
        let mut replies = vec![Reply::rpl_none()];
        for k in nick_sender.find_all()? {
            if k.get_nickname().contains(user.as_str()) {
                replies.push(self.send_message_to(&mut k.clone(), None)?);
            }
        }
        Ok(replies)
    }

    fn send_to_hostname(
        &self,
        nick_sender: &ClientChannel<String, Connection>,
        mut hostname: String,
    ) -> Result<Reply, ErrorServer> {
        hostname.remove(0);
        if hostname.pop() == Some('.') {
            for k in nick_sender.find_all()? {
                if let Some(s) = k.get_hostname() {
                    if s.contains(hostname.as_str()) {
                        return self.send_message_to(&mut k.clone(), None);
                    }
                };
            }
        }
        Ok(Reply::rpl_none())
    }

    ///
    /// function that sends a message to
    ///
    fn send_message_to(
        &self,
        client: &mut Connection,
        channel: Option<String>,
    ) -> Result<Reply, ErrorServer> {
        let message = self.create_message(channel);
        //println!("Cree el mensaje {message}");

        if let Err(_e) = write_message_to(&message, client) {
            //Si es error puede que sea porque somos el receptor o porque no tenemos conexion directa con el usuario.
        }

        Ok(Reply::rpl_none())
    }

    fn create_message(&self, channel: Option<String>) -> String {
        let mut message = String::new();
        if let Some(c) = channel {
            message.push_str(&c);
            message.push_str(": ");
        }

        message.push_str(&self.from);
        message.push_str(": ");

        let mut msg = self.msg.clone();
        msg.remove(0);
        message.push_str(&msg);
        message
    }
    ///
    /// Function that
    /// returns the message to
    /// send by the private message
    /// as a String
    ///
    pub fn get_msg_from_privmsg(msg: &Message) -> Result<String, ErrorServer> {
        let params = msg.parameters();
        match params {
            Some(parameters) => match parameters.get(1) {
                None => Ok(String::new()),
                Some(str) => Ok(str.to_owned()),
            },
            None => Ok(String::new()),
        }
    }

    ///
    /// Function that returns
    /// the different receivers
    /// of the private message
    ///
    pub fn get_receivers_from_privmsg(msg: &Message) -> Result<Vec<String>, ErrorServer> {
        let mut receivers = vec![];
        if let Some(parameters) = msg.parameters() {
            if let Some(str) = parameters.first() {
                if !str.starts_with(':') {
                    for spl in str.trim().split(',') {
                        receivers.push(spl.to_string());
                    }
                }
            }
        }
        Ok(receivers)
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
}

impl ToString for PrivMsg {
    fn to_string(&self) -> String {
        let mut msg = String::from(":");
        msg.push_str(&self.from.clone());
        msg.push_str(" PRIVMSG ");
        msg.push_str(&self.to);
        msg.push(' ');
        msg.push_str(&self.msg);
        msg
    }
}
