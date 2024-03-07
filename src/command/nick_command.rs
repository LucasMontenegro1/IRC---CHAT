use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use crate::{
    channel::Channel,
    command::traits::RegistrationCommand,
    error::error_server::ErrorServer,
    parser::message::Message,
    reply::Reply,
    repository::{
        connection::Connection, repository_channel::client_channel::ClientChannel,
        traits::operations::Operations,
    },
    server_comunication::spanning_tree::SpanningTree,
    user::builder::UserBuilder,
};

pub struct NickCommand {
    from: String,
    new_nickname: String,
}

impl RegistrationCommand for NickCommand {
    fn register_user(&self, builder: &UserBuilder) -> UserBuilder {
        let mut user = UserBuilder::new();
        user.load(builder);
        user.nickname(&self.new_nickname);
        user
    }
}

impl NickCommand {
    pub fn new_registration_command(msg: Message) -> Result<Box<dyn RegistrationCommand>, Reply> {
        Ok(Box::new(NickCommand {
            from: String::from(""),
            new_nickname: Self::get_new_nickname_from_msg(msg)?,
        }))
    }
    pub fn new(msg: Message) -> Result<Self, Reply> {
        let from = match msg.prefix() {
            Some(u) => u,
            None => return Err(Reply::err_no_nickname_given(None)),
        };
        Ok(NickCommand {
            from,
            new_nickname: Self::get_new_nickname_from_msg(msg)?,
        })
    }

    pub fn get_new_nickname(&self) -> &str {
        &self.new_nickname
    }

    //La llama cuando se llama la función NICK luego del registro.
    //Falta:
    //- Replies
    // - Más allá de los replies, manejar bien los errores. Sacar unwraps, mejorar match's, etc.
    // - Hay que cambiar el NICK en los channels también.
    pub fn response(
        &self,
        nick_sender: ClientChannel<String, Connection>,
        channel_sender: ClientChannel<String, Channel>,
        spanning_tree: Arc<Mutex<SpanningTree>>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        let c = match nick_sender.search(self.from.clone())? {
            Some(c) => c,
            None => {
                return Ok(vec![Reply::err_erroneus_nickname(
                    None,
                    vec![self.from.clone()],
                )])
            }
        };

        if Self::already_registered(&self.new_nickname, &nick_sender)? {
            //ERR_NICKNAMEINUSE: No se si se refiere al new_nick o a quien lo mandó
            return Ok(vec![Reply::err_nickname_in_use(
                None,
                vec![self.new_nickname.clone()],
            )]);
        }

        if !Self::update_nickname(
            self.from.clone(),
            self.new_nickname.clone(),
            c.clone(),
            &nick_sender,
        )? {
            return Ok(vec![Reply::err_erroneus_nickname(
                None,
                vec![self.new_nickname.clone()],
            )]);
        }

        // Puede haber una gran inconsistencia si ocurre un error en el medio de esta operacion.
        Self::update_channels(&self.from, &self.new_nickname.clone(), &channel_sender)?;

        if let Some(connection) = nick_sender.search(self.new_nickname.clone())? {
            match spanning_tree.lock() {
                Ok(st) => {
                    if let Some(excluded) = st.look_for_nearest_connection(
                        connection.get_servername().unwrap().to_string(),
                    ) {
                        let servers = st.get_servers();
                        for server in servers {
                            if server != excluded {
                                if let Some(mut connection) = server.get_connection() {
                                    let mut prefix = ":".to_string();
                                    prefix.push_str(self.from.clone().as_str());
                                    let msg = Message::new(
                                        Some(prefix),
                                        super::Command::Nick,
                                        Some(vec![self.new_nickname.clone()]),
                                    );
                                    connection.write_all(msg.to_string().as_bytes())?;
                                }
                            }
                        }
                    }
                }
                Err(_) => return Err(ErrorServer::LockedResource),
            }
        }

        Ok(vec![Reply::rpl_none()])
    }

    fn get_new_nickname_from_msg(msg: Message) -> Result<String, Reply> {
        if let Some(params) = &msg.parameters() {
            let mut params = params.iter();
            if let Some(user) = params.next() {
                return Ok(String::from(user));
            }
        }
        Err(Reply::err_no_nickname_given(None))
    }

    fn update_channels(
        old_nickname: &str,
        new_nickname: &str,
        channel_sender: &ClientChannel<String, Channel>,
    ) -> Result<(), ErrorServer> {
        for mut ch in channel_sender.find_all()? {
            if ch.update_member(old_nickname, new_nickname).is_ok() {
                channel_sender.update(ch.name.clone(), ch)?;
            }
        }
        Ok(())
    }

    fn update_nickname(
        old_nickname: String,
        new_nickname: String,
        mut connection: Connection,
        nick_sender: &ClientChannel<String, Connection>,
    ) -> Result<bool, ErrorServer> {
        connection.set_nickname(&new_nickname);
        nick_sender.add(new_nickname, connection)?;
        nick_sender.delete(old_nickname)?;
        Ok(true)
    }

    fn already_registered(
        nickname: &str,
        nick_sender: &ClientChannel<String, Connection>,
    ) -> Result<bool, ErrorServer> {
        let result = nick_sender.search(nickname.to_owned())?;
        Ok(result.is_some())
    }
}
