use crate::{
    channel::Channel,
    command::part::PartMsg,
    error::error_server::ErrorServer,
    parser::message::Message,
    reply::Reply,
    repository::{
        connection::Connection, repository_channel::client_channel::ClientChannel,
        traits::operations::Operations,
    },
};

use std::str::FromStr;

///
/// struct that implements the
/// quit message as stipulated
/// by the irc protocol
///
pub struct QuitMsg {
    pub user: String,
    //Lo pide el RFC pero no tiene ninguna utilidad ahora.
    _msg: String,
}

impl QuitMsg {
    ///
    /// create a new message
    ///  of type quit
    ///
    pub fn new(msg: &Message) -> Result<Self, ErrorServer> {
        let user = match msg.prefix() {
            Some(u) => u,
            None => return Err(ErrorServer::UnknownCommand),
        };
        Ok(QuitMsg {
            _msg: Self::get_msg_from_quit(user.clone(), msg)?,
            user,
        })
    }

    ///
    /// function that responds to the quit
    /// message sent by the user and
    /// acts accordingly
    ///
    pub fn response(
        &self,
        nick_sender: &ClientChannel<String, Connection>,
        channel_sender: &ClientChannel<String, Channel>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        nick_sender.delete(self.user.clone())?;

        self.get_part_messages(channel_sender)?
            .iter()
            .filter_map(|msg| PartMsg::new(msg).ok())
            .for_each(|part| {
                if part.response(channel_sender).is_err() {
                    println!("error - when obtain part response")
                }
            });

        Ok(vec![Reply::rpl_none()])
    }

    ///
    /// function that is responsible for
    /// extracting the farewell message
    /// from the quit message
    ///
    fn get_msg_from_quit(nickname: String, msg: &Message) -> Result<String, ErrorServer> {
        if let Some(parameters) = msg.parameters() {
            if let Some(message) = parameters.first() {
                let mut msg = message.to_string();
                msg.remove(0);
                return Ok(msg);
            };
        }
        Ok(nickname)
    }

    fn create_part_message(&self, channel: Channel) -> Result<Message, ErrorServer> {
        let mut msg = String::from(":");
        msg.push_str(&self.user);
        msg.push_str(" PART ");
        msg.push_str(&channel.name);

        Ok(Message::from_str(&msg)?)
    }

    fn get_part_messages(
        &self,
        channel_sender: &ClientChannel<String, Channel>,
    ) -> Result<Vec<Message>, ErrorServer> {
        let mut part_msgs = vec![];
        for channel in channel_sender.find_all()? {
            part_msgs.push(self.create_part_message(channel)?);
        }
        Ok(part_msgs)
    }
}

#[cfg(test)]
mod test {
    use crate::command::quit::QuitMsg;
    use crate::command::Command;
    use crate::parser::message::Message;

    #[test]
    fn test_new_is_ok() {
        let parameters = vec![":msg_quit".to_string()];
        let prefix = ":user".to_string();
        let res_quit_msg =
            QuitMsg::new(&Message::new(Some(prefix), Command::Quit, Some(parameters)));

        assert!(res_quit_msg.is_ok());
        let quit_msg = res_quit_msg.unwrap();
        assert_eq!(quit_msg._msg, "msg_quit");
        assert_eq!(quit_msg.user, "user");
    }
}
