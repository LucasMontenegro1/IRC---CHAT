use crate::{
    channel::Channel,
    error::error_server::ErrorServer,
    parser::message::Message,
    reply::Reply,
    repository::{
        repository_channel::client_channel::ClientChannel, traits::operations::Operations,
    },
};

pub struct KickMsg {
    user: String,
    channel: Option<String>,
    kicked_user: Option<String>,
}

impl KickMsg {
    ///
    /// function that creates a
    /// new names message
    ///
    pub fn new(msg: &Message) -> Result<Self, ErrorServer> {
        let user = match msg.prefix() {
            Some(u) => u,
            None => return Err(ErrorServer::UnknownCommand),
        };
        Ok(KickMsg {
            user,
            channel: msg.get_param_from_msg(0),
            kicked_user: msg.get_param_from_msg(1),
        })
    }

    ///
    /// function that responds to the names
    /// message sent by the user and
    /// acts accordingly
    ///
    pub fn response(
        &mut self,
        channel_sender: ClientChannel<String, Channel>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        let (channel_name, user_to_kick) = match (&self.channel, &self.kicked_user) {
            (Some(ch), Some(u)) => (ch, u),
            _ => {
                return Ok(vec![Reply::err_need_more_params(
                    None,
                    vec!["KICK".to_string()],
                )]);
            }
        };

        let mut channel = match channel_sender.search(channel_name.to_owned())? {
            Some(ch) => ch,
            None => {
                return Ok(vec![Reply::err_no_such_channel(
                    None,
                    vec![channel_name.to_owned()],
                )]);
            }
        };

        if !channel.has_member(&self.user) {
            return Ok(vec![Reply::err_not_on_channel(
                None,
                vec![self.user.to_owned()],
            )]);
        }

        if !channel.user_can_kick_others(&self.user) {
            return Ok(vec![Reply::err_chan_o_privs_needed(
                None,
                vec![self.user.clone()],
            )]);
        }

        if !channel.remove_member(user_to_kick) {
            return Ok(vec![Reply::err_not_on_channel(
                None,
                vec![user_to_kick.to_owned()],
            )]);
        }

        channel_sender.update(channel_name.to_owned(), channel)?;
        Ok(vec![Reply::rpl_none()])
    }
}

#[cfg(test)]
mod test {
    use crate::command::kick::KickMsg;
    use crate::command::Command;
    use crate::parser::message::Message;

    #[test]
    fn test_new_is_ok() {
        let parameters = vec!["channel".to_string(), "kicked_user".to_string()];
        let prefix = ":user".to_string();
        let kick_msg_res =
            KickMsg::new(&Message::new(Some(prefix), Command::Kick, Some(parameters)));

        assert!(kick_msg_res.is_ok());
        let kick_msg = kick_msg_res.unwrap();
        assert!(kick_msg.channel.is_some());
        assert!(kick_msg.kicked_user.is_some());

        assert_eq!(kick_msg.user, "user");
        assert_eq!(kick_msg.channel.unwrap(), "channel");
        assert_eq!(kick_msg.kicked_user.unwrap(), "kicked_user");
    }
}
