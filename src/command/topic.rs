use crate::{
    channel::Channel,
    error::error_server::ErrorServer,
    parser::message::Message,
    reply::Reply,
    repository::{
        repository_channel::client_channel::ClientChannel, traits::operations::Operations,
    },
};

pub struct TopicMsg {
    user: String,
    channel: Option<String>,
    topic: Option<String>,
}

impl TopicMsg {
    ///
    /// function that creates a
    /// new names message
    ///
    pub fn new(msg: &Message) -> Result<Self, ErrorServer> {
        let user = match msg.prefix() {
            Some(u) => u,
            None => return Err(ErrorServer::UnknownCommand),
        };
        Ok(TopicMsg {
            user,
            channel: msg.get_param_from_msg(0),
            topic: msg.get_param_from_msg(1),
        })
    }

    pub fn new_join(user: String, channel: String) -> Result<Self, ErrorServer> {
        Ok(TopicMsg {
            user,
            channel: Some(channel),
            topic: None,
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
        let mut channel = match &self.channel {
            None => {
                return Ok(vec![Reply::err_need_more_params(
                    None,
                    vec!["TOPIC".to_string()],
                )]);
            }
            Some(name) => match channel_sender.search(name.to_owned())? {
                Some(ch) => ch,
                None => {
                    return Ok(vec![Reply::err_no_such_channel(
                        None,
                        vec![name.to_string()],
                    )]);
                }
            },
        };
        if !channel.has_member(&self.user) {
            return Ok(vec![Reply::err_not_on_channel(
                None,
                vec![self.user.clone()],
            )]);
        }

        let reply = match &self.topic {
            None => match channel.get_topic() {
                Some(t) => Reply::rpl_topic(channel.name.clone(), t),
                None => Reply::rpl_no_topic(channel.name.clone()),
            },
            Some(t) => {
                if !channel.user_can_set_topic(&self.user) {
                    return Ok(vec![Reply::err_chan_o_privs_needed(
                        None,
                        vec![self.user.clone()],
                    )]);
                }

                channel.set_topic(t.to_string());
                channel_sender.update(channel.name.clone(), channel)?;
                Reply::rpl_none()
            }
        };

        Ok(vec![reply])
    }
}
