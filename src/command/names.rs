use std::{io::Write, thread, time::Duration};

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
/// struct that implements a names message
/// according to what is established by the
/// IRC protocol
///
///
pub struct NamesMsg {
    user: String,
    channels: Option<Vec<String>>,
}

impl NamesMsg {
    ///
    /// function that creates a
    /// new names message
    ///
    pub fn new(msg: Message) -> Result<Self, ErrorServer> {
        let user = match msg.prefix() {
            Some(u) => u,
            None => return Err(ErrorServer::UnknownCommand),
        };
        Ok(NamesMsg {
            user,
            channels: Self::get_channels_from_msg(&msg),
        })
    }

    ///
    /// function that responds to the names
    /// message sent by the user and
    /// acts accordingly
    ///
    pub fn response(
        &self,
        channel_sender: ClientChannel<String, Channel>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        //Busca el prefijo
        //Obtiene el nickname
        let list_channels = match &self.channels {
            None => channel_sender.find_all()?,
            Some(channels) => Self::find_channels(channels, &channel_sender)?,
        };

        let replies = if !list_channels.is_empty() {
            let mut r = vec![];
            for ch in &list_channels {
                if ch.has_member(&self.user) || (!ch.is_priv() && !ch.is_secret()) {
                    let response = Self::create_response(ch);
                    r.push(Reply::rpl_nam_rply(None, vec![response]));
                    r.push(Reply::rpl_end_of_names(None, vec![ch.name.to_string()]));
                }
            }
            r
        } else {
            vec![Reply::rpl_end_of_names(None, vec!["".to_string()])]
        };
        Ok(replies)
    }

    fn create_response(channel: &Channel) -> String {
        String::from(&channel.list_channel())
    }

    fn find_channels(
        channels: &Vec<String>,
        channel_sender: &ClientChannel<String, Channel>,
    ) -> Result<Vec<Channel>, ErrorServer> {
        let mut list_channels: Vec<Channel> = vec![];
        for ch in channels {
            match channel_sender.search(ch.to_owned())? {
                Some(c) => list_channels.push(c.clone()),
                None => continue,
            }
        }
        Ok(list_channels)
    }

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
}

///
/// function that sends a message to
/// the given client
///
pub fn send_message_to(
    msg: Vec<String>,
    client: &mut dyn Write,
    ch: String,
) -> Result<(), ErrorServer> {
    send_nam_reply(msg, client)?;
    send_end_of_names(ch, client)
}

pub fn send_end_of_names(ch: String, client: &mut dyn Write) -> Result<(), ErrorServer> {
    let end = Reply::rpl_end_of_names(None, vec![ch]).to_string();

    thread::sleep(Duration::from_millis(150));
    if let Err(_e) = client.write(end.as_bytes()) {
        return Err(ErrorServer::UnreachableClient);
    }
    Ok(())
}

pub fn send_nam_reply(msg: Vec<String>, client: &mut dyn Write) -> Result<(), ErrorServer> {
    let reply = Reply::rpl_nam_rply(None, msg).to_string();

    thread::sleep(Duration::from_millis(150));
    if let Err(_e) = client.write(reply.as_bytes()) {
        return Err(ErrorServer::UnreachableClient);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::io::Write;

    struct MockClient {
        msg: String,
    }

    // impl MockClient {
    //     pub fn new() -> MockClient {
    //         MockClient { msg: "".to_owned() }
    //     }

    //     pub fn msg(&self) -> String {
    //         self.msg.clone()
    //     }
    // }

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
    fn test_send_nam_reply() {
        // let mut client = MockClient::new();
        // let msgs = vec!["hola".to_string()];
        //
        // let result = send_nam_reply(msgs, &mut client);
        //
        // assert_eq!(client.msg,"hola".to_string());
    }
}
