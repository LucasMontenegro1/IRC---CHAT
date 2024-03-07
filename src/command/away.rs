use crate::{
    error::error_server::ErrorServer,
    parser::message::Message,
    reply::Reply,
    repository::{connection::Connection, traits::operations::Operations},
};
pub struct AwayMsg {
    user: String,
    automatic_response: Option<String>,
}

impl AwayMsg {
    ///
    /// function that creates a
    /// new names message
    ///
    pub fn new(msg: &Message) -> Result<Self, ErrorServer> {
        let user = match msg.prefix() {
            Some(u) => u,
            None => return Err(ErrorServer::UnknownCommand),
        };
        Ok(AwayMsg {
            user,
            automatic_response: msg.get_param_from_msg(0),
        })
    }

    ///
    /// function that responds to the names
    /// message sent by the user and
    /// acts accordingly
    ///
    pub fn response(
        &mut self,
        nick_sender: &dyn Operations<String, Connection>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        //Buscarse así mismo
        let reply = if let Some(mut connection) = nick_sender.search(self.user.to_owned())? {
            //Setear el mensaje
            connection.set_away_msg(self.automatic_response.clone());
            //Actualizar la base de datos.
            nick_sender.update(self.user.to_owned(), connection)?;

            match &self.automatic_response {
                None => Reply::rpl_unaway(),
                Some(_msg) => Reply::rpl_nowaway(),
            }
        } else {
            return Err(ErrorServer::UnreachableClient);
        };
        Ok(vec![reply])
    }
}

#[cfg(test)]
mod test {
    use crate::{
        command::{away::AwayMsg, Command},
        error::error_server::ErrorServer,
        parser::message::Message,
        repository::traits::operations::Operations,
    };

    struct MockNickChannel {}
    impl<String, Connection> Operations<String, Connection> for MockNickChannel {
        fn add(&self, key: String, value: Connection) -> Result<bool, ErrorServer> {
            let _ = key;
            let _ = value;
            Ok(true)
        }

        fn delete(&self, key: String) -> Result<bool, ErrorServer> {
            let _ = key;
            Ok(true)
        }

        fn find_all(&self) -> Result<Vec<Connection>, ErrorServer> {
            Ok(vec![])
        }

        fn search(&self, key: String) -> Result<Option<Connection>, ErrorServer> {
            let _ = key;
            Ok(None)
        }

        fn update(&self, key: String, value: Connection) -> Result<bool, ErrorServer> {
            let _ = key;
            let _ = value;
            Ok(true)
        }
    }
    #[test]
    fn test_new_without_automatic_response() {
        let prefix = ":pepe".to_string();
        let res_awaymsg = AwayMsg::new(&Message::new(Some(prefix), Command::Away, None));
        assert!(res_awaymsg.is_ok());
        let away_msg = res_awaymsg.unwrap();
        assert_eq!(away_msg.user, "pepe");
        assert!(away_msg.automatic_response.is_none())
    }

    #[test]
    fn test_new_has_automatic_response() {
        let parameters = vec!["par".to_string()];
        let prefix = ":pepe".to_string();
        let res_awaymsg =
            AwayMsg::new(&Message::new(Some(prefix), Command::Away, Some(parameters)));

        assert!(res_awaymsg.is_ok());
        let away_msg = res_awaymsg.unwrap();
        assert_eq!(away_msg.user, "pepe");
        assert_eq!(away_msg.automatic_response, Some("par".to_string()));
    }

    #[test]
    fn test_response_is_ok() {}
}
