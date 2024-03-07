use crate::{
    channel::Channel,
    error::error_server::ErrorServer,
    parser::message::Message,
    reply::Reply,
    repository::{
        repository_channel::client_channel::ClientChannel, traits::operations::Operations,
    },
};

pub const COMMA_U8: u8 = b',';

///
/// struct that implements a private message
/// according to what is established by the
/// IRC protocol
///
///
pub struct PartMsg {
    user: String,
    channels: Vec<String>,
}

impl PartMsg {
    ///
    /// function that creates a
    /// new private message
    ///
    pub fn new(msg: &Message) -> Result<Self, ErrorServer> {
        if let Some(channels) = get_channels_from_msg(msg) {
            let user = match msg.prefix() {
                Some(u) => u,
                None => return Err(ErrorServer::UnknownCommand),
            };
            return Ok(PartMsg { user, channels });
        }
        Err(ErrorServer::UnknownCommand)
    }

    ///
    /// function that responds to the part
    /// message sent by the user and
    /// acts accordingly
    ///
    pub fn response(
        &self,
        channel_sender: &ClientChannel<String, Channel>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        let mut replies = vec![];
        if self.channels.is_empty() {
            return Ok(vec![Reply::err_need_more_params(
                None,
                vec!["PART".to_string()],
            )]);
        }
        for ch in &self.channels {
            let mut channel = match channel_sender.search(ch.clone())? {
                Some(c) => c,
                None => {
                    //ERR_NOSUCHCHANNEL
                    replies.push(Reply::err_no_such_channel(None, vec![ch.clone()]));
                    continue;
                }
            };

            if !channel.remove_member(&self.user) {
                //ERR_NOTONCHANNEL
                replies.push(Reply::err_not_on_channel(None, vec![ch.clone()]));
                continue;
            }

            if channel.is_empty() {
                println!("Removing channel");
                channel_sender.delete(channel.name)?;
            } else {
                println!("Removing user");
                channel_sender.update(channel.name.clone(), channel)?;
            }
        }
        Ok(replies)
    }
}

///
/// function that parses a message
/// and returns the channel from it
///
fn get_channels_from_msg(msg: &Message) -> Option<Vec<String>> {
    //Parsear los canales
    if let Some(channels) = msg.get_param_from_msg(0) {
        let mut list_channels = vec![];

        for value in channels.trim().split(COMMA_U8 as char) {
            //Validar si comienzan con & y #.
            if !value.is_empty() && (value.contains('&') || value.contains('#')) {
                list_channels.push(value.to_string());
            }
        }
        return Some(list_channels);
    };
    None
}

#[cfg(test)]
mod tests {
    use crate::command::part::{get_channels_from_msg, PartMsg};
    use crate::command::Command;
    use crate::parser::message::Message;

    #[test]
    fn test_get_channels_from_msg() {
        let prefix = ":papa".to_string();
        let parameters = vec!["par1,&ch1,#ch2".to_string(), "par2".to_string()];

        let opt_channels =
            get_channels_from_msg(&Message::new(Some(prefix), Command::Part, Some(parameters)));

        assert!(opt_channels.is_some());
        let channels = opt_channels.unwrap();
        assert_eq!(channels.len(), 2);
        assert!(channels.contains(&"&ch1".to_string()));
        assert!(channels.contains(&"#ch2".to_string()));
    }

    #[test]
    fn test_get_channels_from_msg_none_parameters_get_none() {
        let prefix = ":papa".to_string();

        let opt_channels = get_channels_from_msg(&Message::new(Some(prefix), Command::Part, None));

        assert!(opt_channels.is_none());
    }

    #[test]
    fn test_new() {
        let prefix = ":papa".to_string();
        let parameters = vec!["par1,&ch1,#ch2".to_string(), "par2".to_string()];

        let partmsg_res =
            PartMsg::new(&Message::new(Some(prefix), Command::Part, Some(parameters)));

        assert!(partmsg_res.is_ok());
        let partmsg = partmsg_res.unwrap();
        assert_eq!(partmsg.user, "papa".to_string());
        assert_eq!(partmsg.channels.len(), 2);
    }

    #[test]
    fn test_new_none_prefix_get_error() {
        let parameters = vec!["par1,&ch1,#ch2".to_string(), "par2".to_string()];

        let partmsg_res = PartMsg::new(&Message::new(None, Command::Part, Some(parameters)));

        assert!(partmsg_res.is_err());
    }

    #[test]
    fn test_new_none_parameters_get_error() {
        let prefix = ":papa".to_string();

        let partmsg_res = PartMsg::new(&Message::new(Some(prefix), Command::Part, None));

        assert!(partmsg_res.is_err());
    }
}
