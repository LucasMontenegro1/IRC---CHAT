use std::collections::HashSet;

use crate::{
    channel::Channel,
    error::error_server::ErrorServer,
    parser::message::Message,
    reply::Reply,
    repository::connection::Connection,
    repository::{
        repository_channel::client_channel::ClientChannel, traits::operations::Operations,
    },
};

pub const COMMA_U8: u8 = b',';

///
/// struct that implements a names message
/// according to what is established by the
/// IRC protocol
///
///
pub struct Mode {
    user: String,
    channel: Option<String>,
    nick: Option<String>,
    flag: Option<String>,
    param: Option<String>,
}

impl Mode {
    ///
    /// function that creates a
    /// new names message
    ///
    pub fn new(msg: &Message) -> Result<Self, ErrorServer> {
        let user = match msg.prefix() {
            Some(u) => u,
            None => return Err(ErrorServer::UnknownCommand),
        };
        Ok(Mode {
            user,
            channel: Self::get_channel_from_msg(msg),
            nick: Self::get_nick_from_msg(msg),
            flag: msg.get_param_from_msg(1),
            param: msg.get_param_from_msg(2),
        })
    }

    ///
    /// function that responds to the names
    /// message sent by the user and
    /// acts accordingly
    ///
    pub fn response(
        &mut self,
        nick_sender: ClientChannel<String, Connection>,
        channel_sender: ClientChannel<String, Channel>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        //Busca el prefijo
        //Obtiene el nickname
        let response: Option<HashSet<String>>;
        let mut replies = vec![];

        if let (None, None) = (&self.channel, &self.nick) {
            return Ok(vec![Reply::err_need_more_params(
                None,
                vec![String::from("MODE")],
            )]);
        }

        if let Some(ch) = &self.channel {
            match channel_sender.search(ch.to_string())? {
                Some(mut c) => {
                    if c.is_channel_operator(&self.user) {
                        match &self.flag {
                            Some(f) => {
                                response = c.modify_channel_flag(f, self.param.clone());
                            }
                            None => {
                                let vec = vec![ch.to_string(), c.return_channel_flags_str()];
                                return Ok(vec![Reply::rply_modes(vec)]);
                            }
                        }
                        channel_sender.update(c.name.clone(), c.clone())?;
                    } else {
                        return Ok(vec![Reply::err_chan_o_privs_needed(
                            None,
                            vec![c.name.clone()],
                        )]);
                    }
                }
                None => return Ok(vec![Reply::err_no_such_channel(None, vec![ch.to_owned()])]),
            }

            if let Some(r) = response {
                for i in r {
                    replies.push(Reply::rpl_ban_list(vec![ch.to_string(), i]));
                }
                replies.push(Reply::rpl_end_ban_list(vec![ch.to_string()]));
            }
        }

        if let Some(s) = &self.nick {
            if *s != self.user {
                return Ok(vec![Reply::err_users_dont_match()]);
            }

            let mut client = match nick_sender.search(self.user.clone())? {
                Some(c) => c,
                None => return Err(ErrorServer::UnreachableClient),
            };

            match &self.flag {
                Some(f) => {
                    client.modify_connecion_flag(f);
                    nick_sender.update(self.user.clone(), client.clone())?;
                }
                None => {
                    let vec = vec![s.to_string(), client.return_connection_flags_str()];
                    return Ok(vec![Reply::rply_modes(vec)]);
                }
            }
        }

        Ok(replies)
    }

    fn get_channel_from_msg(msg: &Message) -> Option<String> {
        if let Some(ch) = msg.get_param_from_msg(0) {
            if ch.starts_with('#') || ch.starts_with('&') {
                return Some(ch);
            }
        }

        None
    }

    fn get_nick_from_msg(msg: &Message) -> Option<String> {
        if let Some(s) = msg.get_param_from_msg(0) {
            if !s.starts_with('#') && !s.starts_with('&') {
                return Some(s);
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use crate::command::mode::Mode;
    use crate::command::Command;
    use crate::parser::message::Message;

    #[test]
    fn get_nick_from_msg() {
        let parameters = vec!["pepe".to_string()];
        let result = Mode::get_nick_from_msg(&Message::new(None, Command::Mode, Some(parameters)));

        assert_eq!(result, Some("pepe".to_string()));
    }

    #[test]
    fn get_miss_nick_from_msg_get_none() {
        let parameters = vec![];
        let result = Mode::get_nick_from_msg(&Message::new(None, Command::Mode, Some(parameters)));

        assert!(result.is_none());
    }

    #[test]
    fn get_miss_parameters_from_msg_get_none() {
        let result = Mode::get_nick_from_msg(&Message::new(None, Command::Mode, None));
        assert!(result.is_none());
    }

    #[test]
    fn get_none_from_msg_because_nick_parameter_has_hastash() {
        let parameters = vec!["#pepe".to_string()];
        let result = Mode::get_nick_from_msg(&Message::new(None, Command::Mode, Some(parameters)));

        assert!(result.is_none());
    }

    #[test]
    fn get_none_from_msg_because_nick_parameter_has_andpersand() {
        let parameters = vec!["&pepe".to_string()];
        let result = Mode::get_nick_from_msg(&Message::new(None, Command::Mode, Some(parameters)));

        assert!(result.is_none());
    }

    /// get channel tests
    #[test]
    fn get_channel_from_msg() {
        let parameters = vec!["#ch1".to_string()];
        let result =
            Mode::get_channel_from_msg(&Message::new(None, Command::Mode, Some(parameters)));

        assert_eq!(result, Some("#ch1".to_string()));
    }

    #[test]
    fn get_channel_from_msg_miss_channel_get_none() {
        let parameters = vec!["ch1".to_string()];
        let result =
            Mode::get_channel_from_msg(&Message::new(None, Command::Mode, Some(parameters)));

        assert!(result.is_none());
    }

    #[test]
    fn get_channel_from_msg_miss_parameter_get_none() {
        let result = Mode::get_channel_from_msg(&Message::new(None, Command::Mode, None));

        assert!(result.is_none());
    }

    #[test]
    fn get_channel_from_msg_with_andpersand() {
        let parameters = vec!["&ch1".to_string()];
        let result =
            Mode::get_channel_from_msg(&Message::new(None, Command::Mode, Some(parameters)));

        assert_eq!(result, Some("&ch1".to_string()));
    }
}
