use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use crate::{
    channel::Channel,
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

///
/// struct that implements a notice message
/// according to what is established by the
/// IRC protocol
///
///
pub struct NoticeMsg {
    from: String,
    msg: String,
    to: Vec<String>,
    st: Arc<Mutex<SpanningTree>>,
}

impl NoticeMsg {
    ///
    /// function that creates a
    /// new notice message
    ///
    pub fn new(msg: Message, st: Arc<Mutex<SpanningTree>>) -> Result<Self, ErrorServer> {
        if let Some(msg_send) = msg.get_param_from_msg(1) {
            let from = match msg.prefix() {
                Some(u) => u,
                None => return Err(ErrorServer::UnknownCommand),
            };
            if let Some(u) = Self::get_receivers_from_notice(&msg) {
                return Ok(NoticeMsg {
                    from,
                    to: u,
                    msg: msg_send,
                    st,
                });
            } else {
                return Ok(NoticeMsg {
                    from,
                    to: Vec::new(),
                    msg: msg_send,
                    st,
                });
            }
        }
        Err(ErrorServer::BadQuery)
    }

    ///
    /// function that responds to the private
    /// message sent by the user and
    /// acts accordingly
    ///
    pub fn response(
        self,
        nick_sender: ClientChannel<String, Connection>,
        channel_sender: ClientChannel<String, Channel>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        for string in &self.to {
            if self.to.is_empty() {
                return Ok(vec![Reply::rpl_none()]);
            }

            if self.msg.is_empty() {
                return Ok(vec![Reply::rpl_none()]);
            }
            let oper = match nick_sender.search(self.from.clone())? {
                Some(oper) => oper.is_op_connection(),
                None => return Err(ErrorServer::UnreachableClient),
            };
            if oper {
                for x in self.to.clone() {
                    if x.starts_with('$') {
                        self.send_multiple_msgs(&nick_sender, x)?;
                        return Ok(vec![Reply::rpl_none()]);
                    }
                }
            }
            if &self.from == string {
                continue;
            }
            if string.starts_with('&') || string.starts_with('#') {
                match channel_sender.search(string.clone())? {
                    Some(c) => {
                        for member in c.return_members() {
                            self.process_receiver(&nick_sender, member, Some(string.clone()))?;
                        }
                    }
                    None => return Ok(vec![Reply::rpl_none()]),
                }
            } else {
                self.process_receiver(&nick_sender, string.to_string(), None)?;
            }
        }
        Ok(vec![Reply::rpl_none()])
    }

    ///
    /// function that is responsible for
    /// processing  who receives the given
    /// notice message
    ///  
    fn process_receiver(
        &self,
        nick_sender: &ClientChannel<String, Connection>,
        receiver: String,
        channel: Option<String>,
    ) -> Result<(), ErrorServer> {
        if self.from == receiver {
            return Ok(());
        }
        match nick_sender.search(receiver)? {
            Some(mut c) => self.send_message_to(&mut c, channel)?,
            None => return Ok(()),
        }

        Ok(())
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
        mut user: String,
    ) -> Result<(), ErrorServer> {
        user.remove(0);
        user.remove(0);
        let result = nick_sender.find_all()?;
        for k in result.iter() {
            if k.get_nickname().contains(user.as_str()) {
                self.send_message_to(&mut k.clone(), None)?;
            }
        }
        Ok(())
    }

    ///
    /// function that sends a message to
    /// the given client
    ///
    fn send_message_to(
        &self,
        client: &mut Connection,
        channel: Option<String>,
    ) -> Result<(), ErrorServer> {
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

        if let Ok(st) = self.st.lock() {
            if let Some(servername) = client.get_servername() {
                if *servername != st.get_root().server.servername {
                    let mut from = ":".to_string();
                    from.push_str(self.from.clone().as_str());
                    let msg = Message::new(
                        Some(from),
                        crate::command::Command::Notice,
                        Some(vec![client.get_nickname(), self.msg.clone()]),
                    );
                    if let Some(server) = st.look_for_nearest_connection(servername.to_string()) {
                        if let Some(mut connection) = server.get_connection() {
                            if let Err(_e) = connection.write_all(msg.to_string().as_bytes()) {
                                return Err(ErrorServer::UnreachableClient);
                            }
                        }
                    }
                } else if let Err(_e) = write_message_to(&message, client) {
                    return Err(ErrorServer::UnreachableClient);
                }
            }
        } else {
            return Err(ErrorServer::LockedResource);
        }
        Ok(())
    }

    ///
    /// Function that
    /// returns the message to
    /// send by the notice message
    /// as a String
    ///
    pub fn get_msg_from_notice(msg: &Message) -> Option<String> {
        msg.get_param_from_msg(1)
    }

    ///
    /// Function that returns
    /// the different receivers
    /// of the notice message
    ///
    fn get_receivers_from_notice(msg: &Message) -> Option<Vec<String>> {
        let mut receivers = vec![];
        if let Some(str) = msg.get_param_from_msg(0) {
            let split = str.trim().split(',');
            for spl in split {
                receivers.push(spl.to_string());
            }
            return Some(receivers);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::command::notice_msg::NoticeMsg;
    use crate::command::Command;
    use crate::parser::message::Message;

    #[test]
    fn test_get_msg_from_notice() {
        let prefix = ":papa".to_string();
        let parameters = vec!["par1".to_string(), "par2".to_string()];
        let opt_res = NoticeMsg::get_msg_from_notice(&Message::new(
            Some(prefix),
            Command::Notice,
            Some(parameters),
        ));

        assert_eq!(opt_res, Some("par2".to_string()));
    }

    #[test]
    fn test_get_msg_from_notice_none_parameters_get_none() {
        let prefix = ":papa".to_string();
        let opt_res =
            NoticeMsg::get_msg_from_notice(&Message::new(Some(prefix), Command::Notice, None));

        assert_eq!(opt_res, None);
    }

    #[test]
    fn test_get_receivers_from_notice() {
        let prefix = ":papa".to_string();
        let parameters = vec!["us1,us2,us3".to_string(), "par2".to_string()];
        let opt_res = NoticeMsg::get_receivers_from_notice(&Message::new(
            Some(prefix),
            Command::Notice,
            Some(parameters),
        ));

        assert!(opt_res.is_some());
        let res = opt_res.unwrap();
        assert_eq!(res.len(), 3);
        assert!(res.contains(&"us1".to_string()));
        assert!(res.contains(&"us2".to_string()));
        assert!(res.contains(&"us3".to_string()));
    }

    #[test]
    fn test_get_receivers_from_notice_none_parameters_get_none() {
        let prefix = ":papa".to_string();
        let opt_res = NoticeMsg::get_receivers_from_notice(&Message::new(
            Some(prefix),
            Command::Notice,
            None,
        ));

        assert!(opt_res.is_none());
    }
}
