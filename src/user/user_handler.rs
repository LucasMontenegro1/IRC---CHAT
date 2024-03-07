use std::{
    io::Write,
    net::TcpStream,
    str::FromStr,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    channel::Channel,
    command::{
        away::AwayMsg,
        dcc_accept::DccAcceptMessage,
        dcc_chat::DccChatMessage,
        dcc_pause::DccPauseMessage,
        dcc_resume::DccResumeMessage,
        dcc_send::DccSendMessage,
        invite::InviteMsg,
        join::JoinMsg,
        kick::KickMsg,
        kill::Kill,
        list::ListMsg,
        mode::Mode,
        names::NamesMsg,
        nick_command::NickCommand,
        notice_msg::NoticeMsg,
        oper_msg::OperMsg,
        part::PartMsg,
        pass_command::PassCommand,
        private_msg::PrivMsg,
        quit::QuitMsg,
        squit::SquitMsg,
        topic::TopicMsg,
        traits::{RegistrationCommand, Runnable},
        user_command::*,
        who::Who,
        whois::Whois,
        Command,
    },
    dcc::{
        command::{
            accept::DccAccept, chat::DccChat, pause::DccPause, resume::DccResume, send::DccSend,
        },
        dcc_handler::is_dcc_chat,
    },
    error::error_server::ErrorServer,
    parser::{add_prefix, dcc_message::DccMessage, message::Message},
    reply::{code::Code, Reply},
    repository::{
        connection::Connection, repository_channel::client_channel::ClientChannel,
        traits::operations::Operations,
    },
    server_comunication::{
        info_sender::{inform_all_server_an_user_command, inform_all_servers},
        server::Server,
        spanning_tree::SpanningTree,
    },
    utils::{read_message_from, write_message_to},
};

use super::{builder::UserBuilder, User};

///
/// struct that implements the user handler,
/// this is an entity that is in charge of
/// everything that has to do with adding
/// new users and that contains their thread
///
#[derive(Clone)]
pub struct UserHandler {
    nick_sender: ClientChannel<String, Connection>,
    channel_sender: ClientChannel<String, Channel>,
    user: Option<User>,
    spanning_tree: Arc<Mutex<SpanningTree>>,
}

impl UserHandler {
    ///
    /// function that creates a new
    /// user handler
    ///
    pub fn new(
        nick_sender: ClientChannel<String, Connection>,
        channel_sender: ClientChannel<String, Channel>,
        spanning_tree: Arc<Mutex<SpanningTree>>,
    ) -> Self {
        UserHandler {
            nick_sender,
            channel_sender,
            user: None,
            spanning_tree,
        }
    }

    /// Function in charge of the registration
    /// process of a user in the IRC network,
    /// creating the UserBuilder entity
    ///
    /// # Arguments
    /// * `msg` - Entity that represents a recevied message from a client of the
    /// IRC network
    /// * `servername` - Name of the server on which the user is registered
    /// * `socket` - Stream through which the user communicates to complete the
    /// registration process
    ///
    /// # Returns
    /// If the user succesfully completes the registration process, it returns a
    /// UserHandler entity with all the required information.
    pub fn user_registration(
        &mut self,
        msg: Message,
        servername: &str,
        socket: &mut TcpStream,
    ) -> Result<Box<dyn Runnable>, ErrorServer> {
        //println!("Registrando un nuevo cliente");
        let builder = match Self::create_user_builder(msg, &UserBuilder::new()) {
            Some(b) => b,
            None => UserBuilder::new(),
        };
        let mut new_user = self.register_user(servername, socket, builder)?;
        loop {
            new_user = match self.add_user(new_user, socket)? {
                None => self.register_user(servername, socket, UserBuilder::new())?,
                Some(u) => return Ok(u),
            }
        }
    }

    fn create_user_builder(msg: Message, new_user: &UserBuilder) -> Option<UserBuilder> {
        println!("UserHandler: {:?}", msg);
        // Por ahora chequeamos manualmente que tipo de comando tiene el msg.
        let builder = match Self::process_registration_msg(msg) {
            Some(b) => match b {
                Ok(command) => command.register_user(new_user),
                Err(r) => {
                    let _ = Self::handle_register_reply(r);
                    return None;
                }
            },
            None => return None,
        };
        Some(builder)
    }

    ///
    /// function that is in charge of
    /// registering new users on the
    /// server
    ///
    fn register_user(
        &self,
        servername: &str,
        socket: &TcpStream,
        mut new_user: UserBuilder,
    ) -> Result<User, ErrorServer> {
        let mut client = socket.try_clone()?;
        // Loopea hasta que se completa el UserBuilder.
        loop {
            let msg = read_message_from(&mut client)?;
            match Message::from_str(&msg) {
                Ok(msg) => {
                    let mut builder = match Self::create_user_builder(msg, &new_user) {
                        Some(b) => b,
                        None => continue,
                    };
                    //Agrego el servername con el nombre del server. Habría que hacer algo parecido con el hostname.
                    builder.servername(servername);
                    // Si es capaz de construir, es porque tiene los atributos completos.
                    match builder.build() {
                        Ok(u) => return Ok(u),
                        Err(_) => new_user = builder,
                    }
                }
                Err(e) => write_message_to(&format!("{e:}"), &mut client)?,
            }
        }
    }

    fn handle_register_reply(r: Reply) -> Result<Reply, ErrorServer> {
        match r.code() {
            Code::RplyNone => Err(ErrorServer::UnacceptedClient),
            _ => Ok(r),
        }
    }

    fn process_registration_msg(
        msg: Message,
    ) -> Option<Result<Box<dyn RegistrationCommand>, Reply>> {
        let builder = match msg.command() {
            Command::User => UserCommand::new_registration_command(msg),
            Command::Nick => NickCommand::new_registration_command(msg),
            Command::Pass => PassCommand::new_registration_command(msg),
            Command::Quit => Err(Reply::rpl_none()),
            _ => return None,
        };
        Some(builder)
    }

    fn add_user(
        &mut self,
        user: User,
        socket: &mut TcpStream,
    ) -> Result<Option<Box<dyn Runnable>>, ErrorServer> {
        let connection = Connection::new(socket.try_clone()?, user.clone());

        if !self.nick_sender.add(user.nickname.clone(), connection)? {
            let reply = Reply::err_nickname_in_use(None, vec![user.nickname]);
            write_message_to(&reply, socket)?;
            return Ok(None);
        } else {
            match self.spanning_tree.lock() {
                Ok(st) => {
                    let servers = st.get_servers();
                    for server in servers {
                        if let Some(mut connection) = server.get_connection() {
                            let msg = user.build_user_msg();
                            connection.write_all(msg.as_bytes())?;
                        }
                    }
                }
                Err(_) => return Err(ErrorServer::LockedResource),
            }
            println!("User: Successfully connected to: {}", socket.local_addr()?);
        }
        self.set_user(user);
        Ok(Some(Box::new(self.clone())))
    }

    fn set_user(&mut self, user: User) {
        self.user = Some(user);
    }

    ///
    /// function that clones the nick sender
    ///
    // fn nick_sender(&self) -> ClientChannel<String, Connection> {
    //     self.nick_sender.clone()
    // }

    pub fn execute_irc_command_from(
        msg: Message,
        user: &mut User,
        spanning_tree: &Arc<Mutex<SpanningTree>>,
        nick_sender: &ClientChannel<String, Connection>,
        channel_sender: &ClientChannel<String, Channel>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        match msg.command() {
            Command::Kill => {
                let oper = match nick_sender.search(user.clone().nickname)? {
                    Some(oper) => oper.is_op_connection(),
                    None => return Err(ErrorServer::UnreachableClient),
                };
                let msg = Kill::new(msg, oper)?;
                msg.response(nick_sender.to_owned(), spanning_tree.clone())
            }

            Command::Quit => {
                let quit = QuitMsg::new(&msg)?;
                inform_all_server_an_user_command(
                    spanning_tree,
                    nick_sender,
                    &msg.prefix().unwrap(),
                    msg,
                )?;
                quit.response(nick_sender, channel_sender)?;
                Err(ErrorServer::UnreachableClient)
            }
            Command::List => {
                let msg = ListMsg::new(msg)?;
                msg.response(channel_sender.to_owned())
            }
            Command::Names => {
                let msg = NamesMsg::new(msg)?;
                msg.response(channel_sender.to_owned())
            }
            Command::Mode => {
                let mut mode = Mode::new(&msg)?;
                let replies = mode.response(nick_sender.to_owned(), channel_sender.to_owned());
                inform_all_servers(spanning_tree, nick_sender, &msg.prefix().unwrap(), msg)?;
                replies
            }
            Command::Invite => {
                inform_all_server_an_user_command(spanning_tree, nick_sender, &msg.prefix().unwrap(), msg.clone())?;
                let mut msg = InviteMsg::new(msg)?;
                msg.response(nick_sender.to_owned(), channel_sender.to_owned())
            }
            Command::Whois => {
                println!("WHOIS");
                let msg = Whois::new(msg)?;
                msg.response(nick_sender.to_owned(), channel_sender.to_owned())
            }
            Command::Who => {
                println!("WHO");
                let msg = Who::new(msg)?;
                msg.response(nick_sender.to_owned(), channel_sender.to_owned())
            }
            Command::Join => {
                let mut r = vec![];
                let join = JoinMsg::new(&msg)?;
                r.append(&mut join.response(channel_sender.clone())?);
                inform_all_servers(
                    spanning_tree,
                    nick_sender,
                    &msg.prefix().unwrap(),
                    msg.clone(),
                )?;
                //RPL_NAMREPLY

                let names = NamesMsg::new(msg)?;
                r.append(&mut names.response(channel_sender.to_owned())?);
                Ok(r)
            }
            Command::Part => {
                let part = PartMsg::new(&msg)?;
                inform_all_servers(spanning_tree, nick_sender, &msg.prefix().unwrap(), msg)?;
                part.response(channel_sender)
            }
            Command::Privmsg => {
                let privmsgs = PrivMsg::new(msg)?;
                let mut replies = vec![];
                let st = match spanning_tree.lock() {
                    Ok(st) => st,
                    Err(_) => return Err(ErrorServer::LockedResource),
                };
                for msg in privmsgs {
                    let servers = msg.get_interested_servers(nick_sender, channel_sender)?;
                    let mut servers: Vec<Server> = servers
                        .into_iter()
                        .filter_map(|s| st.look_for_nearest_connection(s))
                        .collect();

                    let informer =
                        st.look_for_nearest_connection(msg.get_origin_server(nick_sender)?);

                    servers.sort_by(|a, b| a.servername.cmp(&b.servername));
                    servers.dedup();

                    //Filtrar el informador
                    if let Some(server) = informer {
                        servers.retain(|s| {
                            //println!("{} vs INFORMADOR {}", s.servername, server.servername);
                            server.servername != s.servername
                                && s.servername != st.get_root().server.servername
                        });
                    }

                    servers.into_iter().for_each(|mut s| {
                        println!("Le envio al servidor: {:?}", s);
                        if write_message_to(&msg, &mut s).is_err() {
                            println!("error - send msg server -> sever")
                        };
                    });

                    replies.append(&mut msg.response(nick_sender, channel_sender)?);
                }

                Ok(replies)
            }
            Command::Nick => {
                let mut replies = vec![Reply::rpl_none()];
                match NickCommand::new(msg) {
                    Ok(cmd) => {
                        let replies = cmd.response(
                            nick_sender.to_owned(),
                            channel_sender.to_owned(),
                            spanning_tree.clone(),
                        )?;
                        if let Some(r) = replies.first() {
                            if Reply::rpl_none().eq(r) {
                                user.set_nickname(cmd.get_new_nickname());
                            }
                        }
                    }
                    Err(r) => {
                        replies = vec![r];
                    }
                };
                Ok(replies)
            }
            Command::User => Ok(vec![Reply::err_already_registered(None)]),
            Command::Pass => Ok(vec![Reply::err_already_registered(None)]),
            Command::Oper => {
                let oper = OperMsg::new(&msg)?;
                inform_all_servers(spanning_tree, nick_sender, &msg.prefix().unwrap(), msg)?;
                oper.response(nick_sender.to_owned())
            }
            Command::Notice => {
                let msg = NoticeMsg::new(msg, spanning_tree.clone())?;
                msg.response(nick_sender.to_owned(), channel_sender.to_owned())
            }
            Command::Away => {
                let mut away = AwayMsg::new(&msg)?;
                inform_all_servers(spanning_tree, nick_sender, &msg.prefix().unwrap(), msg)?;
                away.response(nick_sender)
            }
            Command::Kick => {
                let mut kick = KickMsg::new(&msg)?;
                let result = kick.response(channel_sender.to_owned());
                inform_all_servers(spanning_tree, nick_sender, &msg.prefix().unwrap(), msg)?;
                result
            }
            Command::Topic => {
                let mut topic = TopicMsg::new(&msg)?;
                let replies = topic.response(channel_sender.to_owned())?;
                inform_all_servers(spanning_tree, nick_sender, &msg.prefix().unwrap(), msg)?;
                Ok(replies)
            }
            Command::Squit => {
                let squit = SquitMsg::new(&msg)?;
                let result = if !squit.get_operator(nick_sender)?.is_op_connection() {
                    Ok(vec![Reply::err_no_privileges(None)])
                } else {
                    squit.response(nick_sender, channel_sender, spanning_tree)
                };
                println!(
                    "POST-SQUIT | SPANNING TREE: {:?}",
                    spanning_tree.lock().unwrap()
                );
                inform_all_servers(spanning_tree, nick_sender, &msg.prefix().unwrap(), msg)?;
                result
            }
            _ => {
                println!("{} - IRC Command is not implemented.", msg.command());
                Ok(vec![Reply::rpl_none()])
            }
        }
    }

    fn handle_user_replies(replies: Vec<Reply>, client: &mut dyn Write) -> Result<(), ErrorServer> {
        for r in replies {
            thread::sleep(Duration::from_millis(150));
            match r.code() {
                Code::RplyNone => continue,
                _ => write_message_to(&r, client)?,
            }
        }
        Ok(())
    }

    pub fn execute_dcc_command_from(
        msg: DccMessage,
        user: &mut User,
        spanning_tree: &Arc<Mutex<SpanningTree>>,
        nick_sender: &ClientChannel<String, Connection>,
        channel_sender: &ClientChannel<String, Channel>,
    ) -> Result<Vec<Reply>, ErrorServer> {
        let _ = user;
        let mut replies = vec![Reply::rpl_none()];
        match msg.command() {
            crate::dcc::command::DccCommand::Chat => {
                let dcc_chat = DccChatMessage::new(&msg)?;
                println!("{:?}", dcc_chat);
                let st = match spanning_tree.lock() {
                    Ok(st) => st,
                    Err(_) => return Err(ErrorServer::LockedResource),
                };
                let servers = dcc_chat.get_interested_servers(nick_sender, channel_sender)?;
                let mut servers: Vec<Server> = servers
                    .into_iter()
                    .filter_map(|s| st.look_for_nearest_connection(s))
                    .collect();

                let informer =
                    st.look_for_nearest_connection(dcc_chat.get_origin_server(nick_sender)?);

                servers.sort_by(|a, b| a.servername.cmp(&b.servername));
                servers.dedup();
                if let Some(server) = informer {
                    servers.retain(|s| {
                        //println!("{} vs INFORMADOR {}", s.servername, server.servername);
                        server.servername != s.servername
                            && s.servername != st.get_root().server.servername
                    });
                }

                servers.into_iter().for_each(|mut s| {
                    println!("Le envio al servidor: {:?}", s);
                    let format = dcc_chat.format_message(DccChat::new(msg.clone()).unwrap());
                    if write_message_to(&format, &mut s).is_err() {
                        println!("error - send msg server -> sever")
                    };
                });

                replies = dcc_chat.response(nick_sender, channel_sender)?;
            }
            crate::dcc::command::DccCommand::Send => {
                let dcc_chat = DccSendMessage::new(&msg)?;
                println!("{:?}", dcc_chat);
                let st = match spanning_tree.lock() {
                    Ok(st) => st,
                    Err(_) => return Err(ErrorServer::LockedResource),
                };
                let servers = dcc_chat.get_interested_servers(nick_sender, channel_sender)?;
                let mut servers: Vec<Server> = servers
                    .into_iter()
                    .filter_map(|s| st.look_for_nearest_connection(s))
                    .collect();

                let informer =
                    st.look_for_nearest_connection(dcc_chat.get_origin_server(nick_sender)?);

                servers.sort_by(|a, b| a.servername.cmp(&b.servername));
                servers.dedup();
                if let Some(server) = informer {
                    servers.retain(|s| {
                        //println!("{} vs INFORMADOR {}", s.servername, server.servername);
                        server.servername != s.servername
                            && s.servername != st.get_root().server.servername
                    });
                }

                servers.into_iter().for_each(|mut s| {
                    println!("Le envio al servidor: {:?}", s);
                    let format = dcc_chat.format_message(DccSend::new(msg.clone()).unwrap());
                    if write_message_to(&format, &mut s).is_err() {
                        println!("error - send msg server -> sever")
                    };
                });

                let _ = &mut dcc_chat.response(nick_sender, channel_sender)?;
            }
            crate::dcc::command::DccCommand::Pause => {
                let dcc_chat = DccPauseMessage::new(&msg)?;
                println!("{:?}", dcc_chat);
                let st = match spanning_tree.lock() {
                    Ok(st) => st,
                    Err(_) => return Err(ErrorServer::LockedResource),
                };
                let servers = dcc_chat.get_interested_servers(nick_sender, channel_sender)?;
                let mut servers: Vec<Server> = servers
                    .into_iter()
                    .filter_map(|s| st.look_for_nearest_connection(s))
                    .collect();

                let informer =
                    st.look_for_nearest_connection(dcc_chat.get_origin_server(nick_sender)?);

                servers.sort_by(|a, b| a.servername.cmp(&b.servername));
                servers.dedup();
                if let Some(server) = informer {
                    servers.retain(|s| {
                        //println!("{} vs INFORMADOR {}", s.servername, server.servername);
                        server.servername != s.servername
                            && s.servername != st.get_root().server.servername
                    });
                }

                servers.into_iter().for_each(|mut s| {
                    println!("Le envio al servidor: {:?}", s);
                    let format = dcc_chat.format_message(DccPause::new(msg.clone()).unwrap());
                    if write_message_to(&format, &mut s).is_err() {
                        println!("error - send msg server -> sever")
                    };
                });

                let _ = &mut dcc_chat.response(nick_sender, channel_sender)?;
            }
            crate::dcc::command::DccCommand::Resume => {
                let dcc_chat = DccResumeMessage::new(&msg)?;
                println!("{:?}", dcc_chat);
                let st = match spanning_tree.lock() {
                    Ok(st) => st,
                    Err(_) => return Err(ErrorServer::LockedResource),
                };
                let servers = dcc_chat.get_interested_servers(nick_sender, channel_sender)?;
                let mut servers: Vec<Server> = servers
                    .into_iter()
                    .filter_map(|s| st.look_for_nearest_connection(s))
                    .collect();

                let informer =
                    st.look_for_nearest_connection(dcc_chat.get_origin_server(nick_sender)?);

                servers.sort_by(|a, b| a.servername.cmp(&b.servername));
                servers.dedup();
                if let Some(server) = informer {
                    servers.retain(|s| {
                        //println!("{} vs INFORMADOR {}", s.servername, server.servername);
                        server.servername != s.servername
                            && s.servername != st.get_root().server.servername
                    });
                }

                servers.into_iter().for_each(|mut s| {
                    println!("Le envio al servidor: {:?}", s);
                    let format = dcc_chat.format_message(DccResume::new(msg.clone()).unwrap());
                    if write_message_to(&format, &mut s).is_err() {
                        println!("error - send msg server -> sever")
                    };
                });

                let _ = &mut dcc_chat.response(nick_sender, channel_sender)?;
            }
            crate::dcc::command::DccCommand::Accept => {
                let dcc_chat = DccAcceptMessage::new(&msg)?;
                println!("{:?}", dcc_chat);
                let st = match spanning_tree.lock() {
                    Ok(st) => st,
                    Err(_) => return Err(ErrorServer::LockedResource),
                };
                let servers = dcc_chat.get_interested_servers(nick_sender, channel_sender)?;
                let mut servers: Vec<Server> = servers
                    .into_iter()
                    .filter_map(|s| st.look_for_nearest_connection(s))
                    .collect();

                let informer =
                    st.look_for_nearest_connection(dcc_chat.get_origin_server(nick_sender)?);

                servers.sort_by(|a, b| a.servername.cmp(&b.servername));
                servers.dedup();
                if let Some(server) = informer {
                    servers.retain(|s| {
                        //println!("{} vs INFORMADOR {}", s.servername, server.servername);
                        server.servername != s.servername
                            && s.servername != st.get_root().server.servername
                    });
                }

                servers.into_iter().for_each(|mut s| {
                    println!("Le envio al servidor: {:?}", s);
                    let format = dcc_chat.format_message(DccAccept::new(msg.clone()).unwrap());
                    if write_message_to(&format, &mut s).is_err() {
                        println!("error - send msg server -> sever")
                    };
                });

                let _ = &mut dcc_chat.response(nick_sender, channel_sender)?;
            }

            _ => println!("Unknown dcc Command"),
        }

        Ok(replies)
    }

    // Method in charge of reading receiving messages from a socket
    // and handle it to a function to be treated as a Command or DccCommand.
    fn listen_to_user(&mut self, socket: &mut TcpStream) -> Result<(), ErrorServer> {
        loop {
            match read_message_from(socket) {
                Err(err) => return self.handle_dropped_connection(&Message::from(err).to_string()),
                Ok(msg) => match self.handle_received_message(&msg) {
                    Ok(replies) => Self::handle_user_replies(replies, socket)?,
                    Err(_e) => println!("An error ocurred at execute message: \n - {msg}"),
                },
            }
        }
    }

    fn handle_dropped_connection(&self, msg: &str) -> Result<(), ErrorServer> {
        let msg = match &self.user {
            Some(u) => add_prefix(u.nickname(), msg),
            None => msg.to_string(),
        };
        let msg = Message::from_str(&msg)?;
        let quit = QuitMsg::new(&msg)?;
        inform_all_servers(
            &self.spanning_tree,
            &self.nick_sender,
            &msg.prefix().unwrap(),
            msg,
        )?;
        quit.response(&self.nick_sender, &self.channel_sender)?;
        Ok(())
    }

    // Function that receives a message on string format.
    // Validates if it's a DccComand or a IRC command, and execute it.
    fn handle_received_message(&mut self, msg: &str) -> Result<Vec<Reply>, ErrorServer> {
        let msg = match &self.user {
            Some(u) => add_prefix(u.nickname(), msg),
            None => msg.to_string(),
        };
        match is_dcc_chat(&msg) {
            true => self.execute_dcc_commands(&msg),
            false => self.execute_irc_commands(&msg),
        }
    }

    // Function that receives an DCC message formatted as a
    // string and decides how to handle it according to the
    // command that travels in it.
    fn execute_dcc_commands(&mut self, msg: &str) -> Result<Vec<Reply>, ErrorServer> {
        println!("DCC MESSAGE {:?}", msg);
        let command = DccMessage::from_str(msg)?;
        self.execute_dcc_command(command)?;
        Ok(vec![Reply::rpl_none()])
    }

    fn execute_dcc_command(&mut self, command: DccMessage) -> Result<Vec<Reply>, ErrorServer> {
        let user = match &mut self.user {
            Some(u) => u,
            None => return Err(ErrorServer::UnreachableClient),
        };
        Self::execute_dcc_command_from(
            command,
            user,
            &self.spanning_tree,
            &self.nick_sender,
            &self.channel_sender,
        )
    }

    // Function that receives an IRC message formatted as a
    // string and decides how to handle it according to the
    // command that travels in it.
    fn execute_irc_commands(&mut self, msg: &str) -> Result<Vec<Reply>, ErrorServer> {
        println!("IRC COMMAND {:?}", msg);
        match Message::from_str(msg) {
            Ok(command) => self.execute_irc_command(command),
            Err(e) => Err(e.into()),
        }
    }

    // Method that delegates the execution of an IRC command to the UserHandler.
    fn execute_irc_command(&mut self, command: Message) -> Result<Vec<Reply>, ErrorServer> {
        let user = match &mut self.user {
            Some(u) => u,
            None => return Err(ErrorServer::UnreachableClient),
        };
        Self::execute_irc_command_from(
            command,
            user,
            &self.spanning_tree,
            &self.nick_sender,
            &self.channel_sender,
        )
    }
}

impl Runnable for UserHandler {
    ///
    /// function that executes the main loop of each of
    /// the users, receiving the commands that
    /// it delivers
    ///
    ///
    fn run(&mut self, socket: &mut TcpStream) -> Result<(), ErrorServer> {
        self.listen_to_user(socket)
    }
}
