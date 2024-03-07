use crate::{
    channel::Channel,
    command::topic::TopicMsg,
    error::{error_channel::ErrorChannel, error_server::ErrorServer},
    parser::message::Message,
    reply::Reply,
    repository::{
        repository_channel::client_channel::ClientChannel, traits::operations::Operations,
    },
};

pub const COMMA_U8: u8 = b',';

///
/// struct that implements a join message
/// according to what is established by the
/// IRC protocol
///
///
pub struct JoinMsg {
    user: String,
    channels: Vec<String>,
    keys: Option<Vec<String>>,
}

impl JoinMsg {
    ///
    /// function that creates a
    /// new join message
    ///
    pub fn new(msg: &Message) -> Result<Self, ErrorServer> {
        let channels = Self::get_channels_from_msg(msg);
        if channels.is_none() {
            return Err(ErrorServer::BadQuery);
        }
        let user = match msg.prefix() {
            Some(u) => u,
            None => return Err(ErrorServer::UnknownCommand),
        };
        Ok(JoinMsg {
            user,
            channels: channels.unwrap(),
            keys: Self::get_key_from_msg(msg),
        })
    }

    ///
    /// function that responds to the private
    /// message sent by the user and
    /// acts accordingly
    ///
    pub fn response(
        &self,
        channel_sender: ClientChannel<String, Channel>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        if self.channels.is_empty() {
            return Ok(vec![Reply::err_need_more_params(
                None,
                vec!["JOIN".to_string()],
            )]);
        }
        let mut replies = vec![];
        let iter_keys = match &self.keys {
            Some(keys) => keys
                .iter()
                .map(|s| {
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.to_owned())
                    }
                })
                .collect::<Vec<Option<String>>>(),
            None => vec![None; self.channels.len()],
        };
        let iter = self.channels.iter().zip(iter_keys.iter());
        for (ch, key) in iter {
            //Busco para saber si existe
            match channel_sender.search(ch.to_owned())? {
                Some(mut c) => match c.add_member(&self.user, key) {
                    Ok(_) => channel_sender.update(c.name.clone(), c.clone()),
                    Err(ErrorChannel::BadKey) => {
                        replies.push(Reply::err_bad_chan_key(vec![ch.clone()]));
                        continue;
                    }
                    Err(ErrorChannel::BannedClient) => {
                        replies.push(Reply::err_banned_from_ch(vec![ch.clone()]));
                        continue;
                    }
                    Err(ErrorChannel::FullChannel) => {
                        replies.push(Reply::err_channel_is_full(ch.clone()));
                        continue;
                    }
                    Err(ErrorChannel::ClientNotInvited) => {
                        replies.push(Reply::err_invite_only_chan(ch.clone()));
                        continue;
                    }
                },
                None => channel_sender.add(ch.clone(), Channel::new(ch.clone(), self.user.clone())),
            }?;

            //RPL_TOPIC
            let mut msg = TopicMsg::new_join(self.user.clone(), ch.clone())?;
            replies.append(&mut msg.response(channel_sender.clone())?);
        }
        Ok(replies)
    }

    ///
    /// function that parses a message
    /// and returns the channel from it
    ///
    fn get_channels_from_msg(msg: &Message) -> Option<Vec<String>> {
        //Tiene que parsear los canales, separados por coma.
        let channels = msg.get_param_from_msg(0)?;

        let split = channels.trim().split(COMMA_U8 as char);
        let mut list_channels = vec![];

        for value in split {
            if !value.is_empty() && (value.contains('&') || value.contains('#')) {
                list_channels.push(value.to_string());
            }
        }

        Some(list_channels)
    }

    //TODO: devuelve bien las keys ?
    fn get_key_from_msg(msg: &Message) -> Option<Vec<String>> {
        let keys: String = msg.get_param_from_msg(1)?;

        let split = keys.trim().split(COMMA_U8 as char);
        let mut list_keys = vec![];

        for value in split {
            list_keys.push(value.to_string());
        }

        Some(list_keys)
    }
}

#[cfg(test)]
mod test {
    use crate::command::join::JoinMsg;
    use crate::command::Command;
    use crate::parser::message::Message;

    //TODO: Testear fuerte -> esta bien parseado? que se espera ?
    #[test]
    fn get_key_from_msg() {
        let parameters = vec!["hola".to_string()];
        JoinMsg::get_key_from_msg(&Message::new(None, Command::Join, Some(parameters)));
    }
}
