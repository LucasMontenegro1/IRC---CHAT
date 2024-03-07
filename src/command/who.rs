use std::{io::Write, thread, time::Duration};

use crate::{
    channel::Channel,
    error::error_server::ErrorServer,
    parser::message::Message,
    reply::Reply,
    repository::{
        connection::Connection, repository_channel::client_channel::ClientChannel,
        traits::operations::Operations,
    },
};

pub const COMMA_U8: u8 = b',';

///
/// struct that implements a names message Who
/// according to what is established by the
/// IRC protocol
///
///
pub struct Who {
    _user: String,
    who_user: Option<String>,
    _is_operator: bool,
}

impl Who {
    ///
    /// function that creates a
    /// new names message
    ///
    pub fn new(msg: Message) -> Result<Self, ErrorServer> {
        let _user = match msg.prefix() {
            Some(u) => u,
            None => return Err(ErrorServer::UnknownCommand),
        };
        Ok(Who {
            _user,
            who_user: msg.get_param_from_msg(0),
            _is_operator: is_operator_from_msg(&msg),
        })
    }

    ///
    /// function that responds to the names
    /// message sent by the user and
    /// acts accordingly
    ///
    pub fn response(
        &self,
        nick_sender: ClientChannel<String, Connection>,
        channel_sender: ClientChannel<String, Channel>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        // Primero genero la response de parte
        // del usuario, osea el rplwhoisuser
        let mut msg: Vec<String> = vec![];
        match &self.who_user {
            None => {
                for c in nick_sender.find_all()? {
                    if let Some(response) = Self::make_response(c) {
                        msg.push(response);
                    }
                }
            }
            Some(name) => {
                let mut matches_channel = false;

                //Decido que hacer si lo encuentro o no
                for ch in channel_sender.find_all()? {
                    if ch.name.contains(name) {
                        matches_channel = true;
                        let str = ch.name.clone();
                        msg.push(str);
                    }
                }

                if !matches_channel {
                    for c in nick_sender.find_all()? {
                        if let Some(response) = Self::make_response(c) {
                            if response.contains(name) {
                                msg.push(response);
                            }
                        }
                    }
                }
            }
        };
        let mut replies = vec![Reply::rpl_none()];
        for s in msg {
            replies.push(Reply::rpl_who(vec![s]));
            replies.push(Reply::rpl_endwho());
        }
        Ok(replies)
    }

    fn make_response(c: Connection) -> Option<String> {
        if c.is_invisible_connection() {
            return None;
        }

        let mut str = String::new();
        if let Some(s) = c.get_username() {
            str.push_str(s);
            str.push(' ');
        }
        if let Some(h) = c.get_hostname() {
            str.push_str(h);
            str.push(' ');
        }

        let mut str = c.get_nickname();
        str.push_str(": ");

        if let Some(f) = c.get_realname() {
            str.push_str(f);
        }
        Some(str)
    }
}

///
/// sends the corresponding
/// reply to the command who
/// according to the established
/// in the irc protocol
///
///
pub fn send_rpl_whorply(msg: String, client: &mut dyn Write) -> Result<(), ErrorServer> {
    // let msg = msg.to_owned()
    let reply = Reply::rpl_who(vec![msg]).to_string();

    thread::sleep(Duration::from_millis(150));
    if let Err(_e) = client.write(reply.as_bytes()) {
        return Err(ErrorServer::UnreachableClient);
    }
    Ok(())
}

///
/// sends the corresponding end of
/// reply to the command who
/// according to the established
/// in the irc protocol
///
///
pub fn send_rpl_endwho(client: &mut dyn Write) -> Result<(), ErrorServer> {
    // let msg = msg.to_owned()
    let reply = Reply::rpl_endwho().to_string();

    thread::sleep(Duration::from_millis(150));
    if let Err(_e) = client.write(reply.as_bytes()) {
        return Err(ErrorServer::UnreachableClient);
    }
    Ok(())
}

pub fn is_operator_from_msg(msg: &Message) -> bool {
    if let Some(u) = msg.get_param_from_msg(1) {
        return u == "o";
    }
    false
}

#[cfg(test)]
mod test {
    use crate::command::Command;
    use crate::parser::message::Message;

    use super::*;

    #[test]
    fn send_reply_endwho_is_ok() {
        //GIVEN
        let mut client = Vec::<u8>::new();
        //WHEN
        let result = send_rpl_endwho(&mut client);
        //THEN
        assert!(result.is_ok());
        assert!(!client.is_empty());
    }

    #[test]
    fn send_reply_whoreply_is_ok() {
        //GIVEN
        let msg = "hola".to_owned();
        let mut client = Vec::<u8>::new();
        assert!(client.is_empty());
        //WHEN
        let result = send_rpl_whorply(msg, &mut client);
        //THEN
        assert!(result.is_ok());
        assert!(!client.is_empty());
    }

    #[test]
    fn get_answer_ok_for_is_operator() {
        let parameters = vec!["pepe".to_owned(), "o".to_owned()];
        let is_op = is_operator_from_msg(&Message::new(None, Command::Away, Some(parameters)));

        assert!(is_op);
    }

    #[test]
    fn get_answer_false_for_is_operator_without_parameter() {
        let parameters = vec!["pepe".to_owned()];
        let is_op = is_operator_from_msg(&Message::new(None, Command::Away, Some(parameters)));

        assert!(!is_op);
    }
}
