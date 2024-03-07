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
/// struct that implements a names message Whois
/// according to what is established by the
/// IRC protocol
///
///
pub struct Whois {
    _user: String,
    who_users: Option<Vec<String>>,
}

impl Whois {
    ///
    /// function that creates a
    /// new names message
    ///
    pub fn new(msg: Message) -> Result<Self, ErrorServer> {
        let _user = match msg.prefix() {
            Some(u) => u,
            None => return Err(ErrorServer::UnknownCommand),
        };
        Ok(Whois {
            _user,
            who_users: get_users_from_msg(&msg),
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
        let responses = self.create_final_response(&nick_sender)?;
        let channels = channel_sender.find_all()?;
        let replies = {
            let mut r = self.send_all_whoisuser_replys(responses)?;
            r.append(&mut self.send_all_whoischannel_replys(channels)?);
            r.push(Reply::rpl_endwhois());
            r
        };
        Ok(replies)
    }

    fn send_all_whoischannel_replys(
        &self,
        channels: Vec<Channel>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        let mut replies = vec![];
        if let Some(users) = &self.who_users {
            for user in users {
                for ch in &channels {
                    if ch.has_member(user) {
                        let mut response = vec![user.clone()];
                        response.push(Self::display_user(user, ch));
                        replies.push(Reply::rpl_whoischannel(response));
                    }
                }
            }
        }
        Ok(replies)
    }

    fn send_all_whoisuser_replys(&self, responses: Vec<String>) -> Result<Vec<Reply>, ErrorServer> {
        let mut replies = vec![];
        for message in responses {
            replies.push(Reply::rpl_whoischannel(vec![message]))
        }
        Ok(replies)
    }

    fn display_user(user: &str, ch: &Channel) -> String {
        let mut st = String::new();
        if ch.is_channel_operator(user) {
            st.push_str(":{@");
            st.push_str(&ch.name);
            st.push('}');
        } else {
            st.push_str(":{+");
            st.push_str(&ch.name);
            st.push('}');
        }
        st
    }

    fn create_response(connection: Connection) -> String {
        let mut str = connection.get_nickname();
        str.push(' ');

        if let Some(s) = connection.get_username() {
            str.push_str(s);
            str.push(' ');
        }

        if let Some(h) = connection.get_hostname() {
            str.push_str(h);
            str.push(' ');
        }

        if let Some(r) = connection.get_realname() {
            str.push_str(r);
        }
        str
    }

    fn create_final_response(
        &self,
        nick_sender: &ClientChannel<String, Connection>,
    ) -> Result<Vec<String>, ErrorServer> {
        let mut msg: Vec<String> = vec![];
        if let Some(users) = &self.who_users {
            for user in users {
                msg.push(match nick_sender.search(user.to_owned())? {
                    Some(u) => Self::create_response(u),
                    None => continue,
                })
            }
        };
        Ok(msg)
    }
}
///
/// sends the corresponding
/// reply to the command whois (Channel)
/// according to the established
/// in the irc protocol
///
///
pub fn send_rpl_whoischannel(msg: Vec<String>, client: &mut dyn Write) -> Result<(), ErrorServer> {
    // let msg = msg.to_owned()
    let reply = Reply::rpl_whoischannel(msg).to_string();

    thread::sleep(Duration::from_millis(150));
    if let Err(_e) = client.write(reply.as_bytes()) {
        return Err(ErrorServer::UnreachableClient);
    }
    Ok(())
}

///
/// sends the corresponding
/// reply to the command whois (user)
/// according to the established
/// in the irc protocol
///
///
pub fn send_rpl_whoisuser(msg: Vec<String>, client: &mut dyn Write) -> Result<(), ErrorServer> {
    // let msg = msg.to_owned()
    let reply = Reply::rpl_whoisuser(msg).to_string();

    thread::sleep(Duration::from_millis(150));
    if let Err(_e) = client.write(reply.as_bytes()) {
        return Err(ErrorServer::UnreachableClient);
    }
    Ok(())
}

pub fn send_rpl_endwhois(client: &mut dyn Write) -> Result<(), ErrorServer> {
    // let msg = msg.to_owned()
    let reply = Reply::rpl_endwhois().to_string();

    thread::sleep(Duration::from_millis(150));
    if let Err(_e) = client.write(reply.as_bytes()) {
        return Err(ErrorServer::UnreachableClient);
    }
    Ok(())
}

///
/// Function that returns
/// the users contained on the
/// whois message
///
fn get_users_from_msg(msg: &Message) -> Option<Vec<String>> {
    if let Some(users) = msg.get_param_from_msg(0) {
        let mut list_users = vec![];

        for value in users.trim().split(COMMA_U8 as char) {
            if !value.is_empty() && (!value.contains('&') && !value.contains('#')) {
                list_users.push(value.to_string());
            }
        }
        return Some(list_users);
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::command::whois::get_users_from_msg;
    use crate::command::Command;
    use crate::parser::message::Message;

    #[test]
    fn test_get_users_from_msg() {
        let prefix = ":papa".to_string();
        let parameters = vec!["usr1,usr2".to_string(), "par2".to_string()];

        let opt_users = get_users_from_msg(&Message::new(
            Some(prefix),
            Command::Whois,
            Some(parameters),
        ));

        assert!(opt_users.is_some());
        let users = opt_users.expect("");
        assert_eq!(users.len(), 2);
    }

    #[test]
    fn test_get_users_from_msg_none_parameters_get_none() {
        let prefix = ":papa".to_string();

        let opt_users = get_users_from_msg(&Message::new(Some(prefix), Command::Whois, None));
        assert!(opt_users.is_none());
    }

    #[test]
    fn test_get_channels_from_msg() {
        let prefix = ":papa".to_string();
        let parameters = vec!["usr1,usr2".to_string(), "par2".to_string()];

        let opt_users = get_users_from_msg(&Message::new(
            Some(prefix),
            Command::Whois,
            Some(parameters),
        ));

        assert!(opt_users.is_some());
        let users = opt_users.expect("");
        assert_eq!(users.len(), 2);
    }
}
