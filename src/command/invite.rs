use std::io::Write;

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

///
/// struct that implements the invite
/// msg according to the irc protocol
///
pub struct InviteMsg {
    user: String,
    channel: String,
    to: String,
}

impl InviteMsg {
    ///
    ///function that creates a
    /// new message of the type
    /// invite
    ///
    pub fn new(msg: Message) -> Result<Self, ErrorServer> {
        let channel = msg.get_param_from_msg(1);
        if let Some(chan) = channel {
            let user = match msg.prefix() {
                Some(u) => u,
                None => return Err(ErrorServer::UnknownCommand),
            };
            let ch = chan;
            let to = msg.get_param_from_msg(0);

            if let Some(t) = to {
                return Ok(InviteMsg {
                    user,
                    channel: ch,
                    to: t,
                });
            }
        }
        Err(ErrorServer::BadQuery)
    }

    ///
    /// function that is responsible for
    /// responding to the invitation message
    /// given by a user who wants to invite
    /// another to a channel within the server
    ///
    ///
    pub fn response(
        &mut self,
        nick_sender: ClientChannel<String, Connection>,
        channel_sender: ClientChannel<String, Channel>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        let mut replies = vec![];
        if let (true, true) = (self.channel.is_empty(), self.user.is_empty()) {
            return Ok(vec![Reply::err_need_more_params(
                None,
                vec![String::from("INVITE")],
            )]);
        }
        //Decido que hacer si lo encuentro o no
        let mut channel = match channel_sender.search(self.channel.clone())? {
            Some(c) => c,
            None => {
                return Ok(vec![Reply::err_no_such_channel(
                    None,
                    vec![self.channel.clone()],
                )])
            }
        };
        match nick_sender.search(self.to.clone())? {
            Some(c) => {
                if !channel.has_member(&self.user) {
                    return Ok(vec![Reply::err_not_on_channel(
                        None,
                        vec![self.channel.clone()],
                    )]);
                } //Reply ERR_NOTONCHANNEL

                if channel.has_member(&self.to) {
                    return Ok(vec![Reply::err_user_on_channel(
                        None,
                        vec![self.to.clone(), self.channel.clone()],
                    )]);
                } // Reply ERR_USERONCHANNEL

                if !channel.is_channel_operator(&self.user) {
                    return Ok(vec![Reply::err_chan_o_privs_needed(
                        None,
                        vec![self.channel.clone()],
                    )]);
                } // Reply ERR_CHANOPRIVSNEEDED

                channel.invite_member(self.to.clone());
                channel_sender.update(channel.name.clone(), channel)?;

                //RPL_AWAY
                if let Some(msg) = c.get_away_msg() {
                    replies.push(Reply::rpl_away(c.get_nickname(), msg));
                }
                //RPL_INVITING
                replies.push(Reply::rpl_inviting(self.channel.clone(), self.to.clone()));
            }
            None => {
                replies.push(Reply::err_no_such_nickname(None, vec![self.to.clone()]));
            }
        }
        Ok(replies)
    }
}

///
/// function that sends a message to
/// the given client
///
pub fn send_message_to(msg: &str, client: &mut dyn Write) -> Result<(), ErrorServer> {
    let msg = msg.to_owned();
    if let Err(_e) = client.write(msg.as_bytes()) {
        return Err(ErrorServer::UnreachableClient);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::command::invite::{send_message_to, InviteMsg};
    use crate::command::Command;
    use crate::parser::message::Message;
    use std::io::Write;

    struct MockClient {
        msg: String,
    }

    impl MockClient {
        // pub fn new() -> MockClient {
        //     MockClient { msg: "".to_owned() }
        // }

        pub fn msg(&self) -> String {
            self.msg.clone()
        }
    }

    impl Write for MockClient {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let x = String::from_utf8_lossy(buf);
            let y = x.as_ref().to_owned();
            if x.as_ref() == "error" {
                return Err(std::io::Error::from(std::io::ErrorKind::WriteZero));
            }
            Ok(y.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.msg = "".to_owned();
            Ok(())
        }
    }

    #[test]
    fn new_invitemsg_is_ok() {
        let parameters = vec!["pepe".to_string(), "channel1".to_string()];
        let prefix = ":user".to_string();
        let result = InviteMsg::new(Message::new(
            Some(prefix),
            Command::Invite,
            Some(parameters),
        ));

        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.user, "user".to_string());
        assert_eq!(msg.channel, "channel1".to_string());
        assert_eq!(msg.to, "pepe".to_string());
    }

    #[test]
    fn send_message_to_client() {
        let mut client = MockClient {
            msg: "pepe".to_owned(),
        };

        let result = send_message_to("hola", &mut client);

        assert!(result.is_ok());
        assert_eq!(client.msg(), "pepe");
    }

    #[test]
    fn send_message_to_client_get_error() {
        let mut client = MockClient {
            msg: "pepe".to_owned(),
        };

        let result = send_message_to("error", &mut client);

        assert!(result.is_err());
    }
}
