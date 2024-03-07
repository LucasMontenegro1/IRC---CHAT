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
/// struct that implements a list message
/// according to what is established by the
/// IRC protocol
///
///
pub struct ListMsg {
    user: String,
    channels: Option<Vec<String>>,
}

impl ListMsg {
    ///
    /// function that creates a
    /// new list message
    ///
    pub fn new(msg: Message) -> Result<Self, ErrorServer> {
        let user = match msg.prefix() {
            Some(u) => u,
            None => return Err(ErrorServer::UnknownCommand),
        };
        Ok(ListMsg {
            user,
            channels: Self::get_channels_from_msg(&msg),
        })
    }

    ///
    /// function that responds to the list
    /// message sent by the user and
    /// acts accordingly
    ///
    pub fn response(
        &self,
        channel_sender: ClientChannel<String, Channel>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        //Itera por los canales y crea un vector de canales
        let (mut list_channels, is_list_all) = match &self.channels {
            None => (channel_sender.find_all()?, true),
            Some(channels) => (Self::find_channels(channels, &channel_sender)?, false),
        };

        list_channels = self.filter_channels(list_channels, &self.user);

        //Creo el response
        let response = self.create_response(list_channels, is_list_all);
        // Se esta buscando así mismo.
        let mut replies = vec![Reply::rply_list_start(None)];

        if !response.is_empty() {
            for msg in response {
                replies.push(Reply::rply_list(None, vec![msg]));
            }
        }

        replies.push(Reply::rply_list_end(None));

        Ok(replies)
    }

    fn filter_channels(&self, channels: Vec<Channel>, user: &str) -> Vec<Channel> {
        let mut vec = Vec::new();
        for ch in channels {
            if !ch.is_secret() || ch.has_member(user) {
                vec.push(ch);
            }
        }
        vec
    }

    fn create_response(&self, channels: Vec<Channel>, list_all: bool) -> Vec<String> {
        let response = match list_all {
            true => channels
                .iter()
                .map(|c| self.info_all_channels(c))
                .collect::<Vec<String>>(),
            false => channels
                .iter()
                .map(|c| self.info_channel(c))
                .collect::<Vec<String>>(),
        };

        response
    }

    fn find_channels(
        channels: &Vec<String>,
        channel_sender: &ClientChannel<String, Channel>,
    ) -> Result<Vec<Channel>, ErrorServer> {
        let mut list_channels: Vec<Channel> = vec![];
        for ch in channels {
            match channel_sender.search(ch.clone())? {
                Some(c) => list_channels.push(c.clone()),
                None => continue,
            }
        }
        Ok(list_channels)
    }

    ///
    /// function that is responsible
    /// for parsing the channels contained
    /// in the list message
    ///
    fn get_channels_from_msg(msg: &Message) -> Option<Vec<String>> {
        //Parsear los canales
        let channels = msg.get_param_from_msg(0)?;

        let split = channels.trim().split(COMMA_U8 as char);
        let mut list_channels = vec![];

        for value in split {
            //Validar si comienzan con & y #.
            if !value.is_empty() && (value.contains('&') || value.contains('#')) {
                list_channels.push(value.to_string());
            }
        }

        Some(list_channels)
    }

    fn info_all_channels(&self, channel: &Channel) -> String {
        let mut response = String::new();
        //Armar una rta con información del canal
        response.push_str(&channel.get_status_name(&self.user));
        response.push(' ');
        if !channel.is_priv() || channel.has_member(&self.user) {
            if let Some(topic) = &channel.get_topic() {
                response.push_str(topic);
                response.push('~');
            }
        }
        response
    }
    fn info_channel(&self, channel: &Channel) -> String {
        let mut response = String::new();
        //Armar una rta con información del canal
        response.push_str(&channel.get_name());
        response.push(' ');
        response.push_str(" status: ");
        response.push_str(channel.status());
        response.push(' ');
        response
    }
}

#[cfg(test)]
mod test {
    use crate::channel::Channel;
    use crate::command::list::ListMsg;
    use crate::command::Command;
    use crate::parser::message::Message;

    #[test]
    fn get_channel_from_msg() {
        let parameters = vec!["pepe".to_string()];
        let prefix = "papa".to_string();
        let listmsg = ListMsg::new(Message::new(Some(prefix), Command::List, Some(parameters)));
        let result = listmsg
            .unwrap()
            .info_channel(&Channel::new("channel1".to_string(), "pepe".to_string()));

        assert_eq!(result, "channel1  status: public ");
    }
}
