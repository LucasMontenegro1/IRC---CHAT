use crate::{
    error::error_server::ErrorServer,
    parser::message::Message,
    reply::Reply,
    repository::{
        connection::Connection, repository_channel::client_channel::ClientChannel,
        traits::operations::Operations,
    },
};

pub struct OperMsg {
    from: String,
    user: String,
    password: String,
}
impl OperMsg {
    pub fn new(msg: &Message) -> Result<Self, ErrorServer> {
        let from = match msg.prefix() {
            Some(u) => u,
            None => return Err(ErrorServer::UnknownCommand),
        };
        let (user, password) = get_user_and_pass_from_msg(msg);
        Ok(OperMsg {
            from,
            user,
            password,
        })
    }

    ///
    /// Function that checks if
    /// the user that is sending
    /// the message is now a
    /// server operator and
    /// returns a bool with the
    /// result
    ///
    pub fn response(
        &self,
        nick_sender: ClientChannel<String, Connection>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        if self.from.is_empty() || self.password.is_empty() {
            return Ok(vec![Reply::err_need_more_params(
                None,
                vec!["OPER".to_string()],
            )]);
        };

        let user_connection = nick_sender.search(self.from.clone())?;

        match user_connection {
            Some(mut connection) => {
                //ByPass porque sino en multiserver no funciona.
                //Debo fijarme si tengo conexion directa, sino, no valido que tenga pass.
                let user = connection.get_user();
                let password;
                if let Some(_c) = connection.see_if_clonable() {
                    if user.password().is_none() {
                        return Ok(vec![Reply::err_no_oper_host(None)]);
                    }
                    password = self.password.clone();
                } else {
                    password = String::from("squit");
                }
                if user.oper_validation(self.user.clone(), password) {
                    connection.get_op_privileges();
                    nick_sender.update(self.from.clone(), connection)?;
                    Ok(vec![Reply::rpl_you_are_oper(None)])
                } else {
                    Ok(vec![Reply::err_password_missmatch(None)])
                }
            }
            None => Err(ErrorServer::UnreachableClient),
        }
    }
}

///
/// function that searches the message
/// and returns the username
///  and password
///
fn get_user_and_pass_from_msg(msg: &Message) -> (String, String) {
    match msg.parameters() {
        Some(parameters) => match parameters.first() {
            None => (String::new(), String::new()),
            Some(str) => match parameters.get(1) {
                Some(p) => (str.to_owned(), p.to_owned()),
                None => (str.to_owned(), String::new()),
            },
        },
        None => (String::new(), String::new()),
    }
}
