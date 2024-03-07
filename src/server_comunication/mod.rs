pub mod info_sender;
pub mod server;
pub mod server_connection;
pub mod spanning_tree;

use crate::{
    channel::Channel,
    command::{
        kill::Kill, nick_command::NickCommand, notice_msg::NoticeMsg, server_msg::ServerMsg,
        squit::SquitMsg, traits::Runnable, user_msg::UserMsg, Command,
    },
    dcc::dcc_handler::is_dcc_chat,
    error::{error_msg::ErrorMsg, error_server::ErrorServer},
    parser::{add_prefix, dcc_message::DccMessage, message::Message},
    reply::{code::Code, Reply},
    repository::{
        connection::Connection, repository_channel::client_channel::ClientChannel,
        traits::operations::Operations,
    },
    user::{user_handler::UserHandler, User},
    utils::{read_message_from, write_message_to, write_messages_to},
};

use self::{
    info_sender::{inform_all_servers, send_to_all_servers},
    server::Server,
    server_connection::ConnectionServer,
    spanning_tree::SpanningTree,
};

use std::{
    io::{stdin, BufRead, BufReader, Write},
    net::{Shutdown, TcpStream},
    str::FromStr,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

const SERVER_IP_PORT_POSITION: usize = 0;

///
///  Struct that is in charge to handle the communication
/// of the [`crate::server::MainServer`] between other servers.
/// Also has direct communication between the [`crate::database::RepositoryHandler`]
/// entities initialized on the main server, because each message received in the
/// server may need information from there or has to manipulate it.
///
#[derive(Clone)]
pub struct ServerComunicationHandler {
    servername: String,
    spanning_tree: Arc<Mutex<SpanningTree>>,
    nick_sender: ClientChannel<String, Connection>,
    channel_sender: ClientChannel<String, Channel>,
}

impl ServerComunicationHandler {
    ///
    /// Basic constructor.
    ///
    pub fn new(
        servername: String,
        servers: Arc<Mutex<SpanningTree>>,
        nick_sender: ClientChannel<String, Connection>,
        channel_sender: ClientChannel<String, Channel>,
    ) -> Self {
        ServerComunicationHandler {
            servername,
            spanning_tree: servers,
            nick_sender,
            channel_sender,
        }
    }

    fn get_servername(&self) -> String {
        self.servername.clone()
    }

    fn get_servers(&self) -> Arc<Mutex<SpanningTree>> {
        self.spanning_tree.clone()
    }

    /// Method in charge of registering a new server to the network.
    /// It sends the handshake to the server that tries to connect,
    /// and establishes the connection by proccessing the received message.
    ///
    /// # Arguments
    /// * `msg`    - Entity that represent the registration message
    /// * `socket` - Stream that belongs to the incoming connection form the server
    /// that tries to establish a new connection
    ///
    /// # Returns
    /// Returns an entity which implements the trait [`Runnable`], in charge
    /// handle the communication between the new server.
    pub fn register_server(
        &mut self,
        msg: Message,
        socket: &mut TcpStream,
    ) -> Result<Box<dyn Runnable>, ErrorServer> {
        self.send_handshake(socket)?;
        self.handle_handshake(msg, socket)
    }

    ///
    /// Function in charge of running the thread where the server
    /// can use the command-line in order to enable the communication
    /// between the server and the computed that created it
    ///
    /// # Returns
    /// The [`JoinHandle`] that enables to manage the resulting error
    /// when the thread it's done.
    pub fn run_cli(&self) -> JoinHandle<Result<(), ErrorServer>> {
        let sc = self.clone();
        //Abre hilo para leer de stdin, sin bloquear al hilo que ejecutó el run_cli()
        thread::spawn(move || -> Result<(), ErrorServer> {
            let mut stdin = BufReader::new(stdin());
            Self::handle_cli_communication(&mut stdin, sc)
        })
    }

    // Execute a loop that reads from stdin in a thread
    // so as not to block the thread that executes this function
    fn handle_cli_communication(
        reader: &mut BufReader<std::io::Stdin>,
        server_comunication: ServerComunicationHandler,
    ) -> Result<(), ErrorServer> {
        while let Some(Ok(line)) = reader.lines().next() {
            match build_msg_from_handshake(&line) {
                Err(e) => println!("handle_cli_communication(): {e}"),
                Ok(msg) => match msg.command() {
                    Command::Server => {
                        if let Err(e) = server_comunication.handle_server_message(&line) {
                            println!("{:?}", e);
                        }
                    }
                    _ => println!("SERVER message was expected, and received: {:?}", msg),
                },
            }
        }
        Err(ErrorServer::UnreachableClient)
    }

    // Handles a received message as a str, containing the 'ip' you want to connect to
    // and the [`Message`] you want to send to that same ip.
    //
    // 1. The readed line it's expected to be a [`Command::Server`] message trying to connect to a new server, so the receipent "ip" is obtained from the handshake.
    // 2. Once the connection with the received ip it's established, construct the [`Message`] from the readed line. If it's not, destroy the stream.
    // 3. The created Message is sent to the destination connection, which is the one that traveled on the command line.
    // 4. Waits for handshake's response.
    fn handle_server_message(&self, line: &str) -> Result<(), ErrorServer> {
        match TcpStream::connect(get_destination_host_from_handshake(line)) {
            Err(e) => Err(ErrorServer::TcpStreamError(e.kind())),
            Ok(mut destination) => match build_msg_from_handshake(line) {
                Ok(_message) => {
                    let _ = self.send_handshake(&mut destination);
                    self.wait_for_handshake_response(destination)
                }
                Err(e) => {
                    let _ = destination.shutdown(Shutdown::Both);
                    Err(e.into())
                }
            },
        }
    }

    // Creates a thread waiting for the handshake response. When the message it's received, validates the handshake.
    //
    // 1. A thread is created, listening for the connection that was created from the command line.
    // 2. We receive the handshake, so we validate that it is a SERVER message.
    fn wait_for_handshake_response(&self, stream: TcpStream) -> Result<(), ErrorServer> {
        let sc = self.clone();
        thread::spawn(move || -> Result<(), ErrorServer> {
            let mut socket = stream.try_clone()?;
            let new_communication = match read_message_from(&mut socket) {
                Ok(msg) => match Message::from_str(&msg) {
                    Ok(message) => sc.handle_handshake(message, &mut socket),
                    Err(_) => Err(ErrorServer::UnknownCommand),
                },
                Err(_e) => {
                    stream.shutdown(Shutdown::Both)?;
                    Err(ErrorServer::TcpFail)
                }
            };
            match new_communication {
                Ok(mut server) => server.run(&mut socket),
                Err(e) => Err(e),
            }
        });
        Ok(())
    }

    // Method in charge of sending the registration message of a new neighbor server to the servers in the network.
    fn inform_neighbours_new_server(&self, msg: Message) -> Result<(), ErrorServer> {
        if !msg.is_command(Command::Server) {
            return Err(ErrorServer::UnexpectedCommand);
        }

        let msg = ServerMsg::new(msg)?;
        let informer = Server::new(self.get_servername(), None);
        let new_server = Server::new(msg.servername, None);
        //HANDSHAKE
        match self.get_servers().lock() {
            Ok(st) => Ok(send_to_all_servers(
                &st,
                Message::from_str(&informer.build_server_message_as_source(&new_server, 2))?,
                &self.get_servername(),
            )?),
            Err(_e) => Err(ErrorServer::LockedResource),
        }
    }

    // Validates that the message it's a handshake. If it is, send all information to the new server connection, adds the new
    // server into the server's network and runs the listening of the messages from the newcome server.
    //
    // 1. If it is indeed a server message, we proceed to send the current information of the server.
    // 2. Announce the new server to the neighbours
    // 3. From the SERVER message, we have information of the server we wanted to connect, so we add it to the network (Spanning Tree).
    // 4. It executes a loop where listens the server's messages from the new connection.
    fn handle_handshake(
        &self,
        msg: Message,
        new_server_stream: &mut TcpStream,
    ) -> Result<Box<dyn Runnable>, ErrorServer> {
        if !msg.is_command(Command::Server) {
            new_server_stream.shutdown(Shutdown::Both)?;
            return Err(ErrorServer::UnexpectedCommand);
        }

        self.inform_neighbours_new_server(msg.clone())?;
        self.send_server_information(new_server_stream)?;
        self.add_server_into_network(&msg.to_string(), new_server_stream)?;
        Ok(Box::new(self.clone()))
    }

    // Prepares info and runs a new thread to send all the information to the new connection.
    fn send_server_information(&self, destination: &mut TcpStream) -> Result<(), ErrorServer> {
        let info = self.prepare_info()?;
        let mut client = destination.try_clone()?;
        thread::spawn(move || {
            if let Err(_e) = write_messages_to(&mut info.iter(), &mut client) {
                println!("ERROR SENDING SERVER INFORMATION.")
            }
        });
        Ok(())
    }

    fn add_server_into_network(
        &self,
        msg: &str,
        new_connection: &mut TcpStream,
    ) -> Result<(), ErrorServer> {
        // Agrego de prefijo el nombre del servidor
        //ya que los hijos deben saber de quien conocen el nuevo servidor.
        let c = Message::from_str(&add_prefix(Some(&self.get_servername()), msg))?;
        let cmd = ServerMsg::new(c.clone())?;
        //Agrega sin conexion si no lo tiene.
        //Manda el mensaje para sus hijos que poseen conexión.
        let replies = cmd.response(self.get_servers())?;
        if let Some(r) = replies.first() {
            //Agrego el server con CONEXION al arbol
            if Code::ErrAlreadyregistred == r.code() {
                new_connection
                    .shutdown(Shutdown::Read)
                    .expect("shutdown call failed");
                return Err(ErrorServer::ServerClosed);
            }
        }
        self.add_server_connection(c, new_connection)
    }

    fn add_server_connection(
        &self,
        msg: Message,
        socket: &mut TcpStream,
    ) -> Result<(), ErrorServer> {
        //Creo la conexion del server que se dió conocer.
        let connection = ConnectionServer::new(socket.try_clone()?);
        match self.spanning_tree.lock() {
            Ok(mut st) => {
                let params = msg.parameters().unwrap();
                let servername = params.first().unwrap();
                //Creo el nuevo nodo del arbol con la info del servidor
                st.set_connection(servername.clone(), connection)?;
            }
            Err(_) => return Err(ErrorServer::LockedResource),
        }
        Ok(())
    }

    // Send the Server's handshake to a neighbour, defined as direct connection server.
    fn send_handshake(&self, new_connection: &mut TcpStream) -> Result<(), ErrorServer> {
        let server = Server::new(self.servername.clone(), None);
        //HANDSHAKE
        write_message_to(
            &server.build_server_message_as_neighbour(&server),
            new_connection,
        )
    }

    //Prepares data from de server communication handler to be sended
    //via IRC commands as Strings.
    fn prepare_info(&self) -> Result<Vec<String>, ErrorServer> {
        let mut messages = vec![];
        messages.append(&mut self.get_server_creation_msg_from_repository()?);
        messages.append(&mut self.get_user_creation_msg_from_repository()?);
        messages.append(&mut self.get_channels_creation_msg_from_repository()?);
        Ok(messages)
    }

    // Create a collection of String with the commands that adds users of the current nickname's repository
    fn get_user_creation_msg_from_repository(&self) -> Result<Vec<String>, ErrorServer> {
        let connections = self.nick_sender.find_all()?;
        Ok(connections
            .iter()
            .map(|c| c.get_user().build_user_msg())
            .collect())
    }

    // Create a collection of String with the commands that generates channels from the current channel's repository
    fn get_channels_creation_msg_from_repository(&self) -> Result<Vec<String>, ErrorServer> {
        //Chequear colision de nombres, especialmente cuando alguno tiene una restricción al respecto.
        let channels = self.channel_sender.find_all()?;
        Ok(channels
            .iter()
            .flat_map(|c| c.build_channel_msg())
            .collect())
    }

    // Create a collection of String with the commands that generates all server that exists from the current network,
    // known by the original server.
    fn get_server_creation_msg_from_repository(&self) -> Result<Vec<String>, ErrorServer> {
        let messages = match self.spanning_tree.lock() {
            Ok(st) => st
                .get_edges()
                .iter()
                .map(|e| {
                    e.source
                        .server
                        .build_server_message_as_source(&e.destination.server, e.cost + 1)
                })
                .collect(),
            Err(_) => return Err(ErrorServer::LockedResource),
        };
        Ok(messages)
    }

    // Method in charge of reading receiving messages from a socket
    // and handle it to a functionto be treated as a Command or DccCommand.
    fn listen_to_server(&self, socket: &mut TcpStream) -> Result<(), ErrorServer> {
        loop {
            match read_message_from(socket) {
                Err(err) => return self.handle_dropped_connection(&Message::from(err).to_string()),
                Ok(msg) => match self.handle_received_message(&msg) {
                    Ok(replies) => Self::handle_user_replies(replies, socket)?,
                    Err(e) => println!("An error ocurred at execute message: \n - {msg} : \n Handled error at handle_received_message() {:?}", e)
                }
            }
        }
    }

    fn handle_dropped_connection(&self, msg: &str) -> Result<(), ErrorServer> {
        // Eliminar server SQUIT
        println!("listen_to_server(): {msg}");
        Err(ErrorServer::ServerClosed)
    }

    fn handle_user_replies(
        _replies: Vec<Reply>,
        client: &mut dyn Write,
    ) -> Result<(), ErrorServer> {
        let _ = client;
        //println!("RECEIVED REPLIES FROM SERVER {:?}", replies);
        Ok(())
    }

    // Function that receives a message on string format.
    // Validates if it's a DccComand or a IRC command, and execute it.
    // Then handle the Result, if it's a Reply ignores it, and if it's an error
    // logs it.
    fn handle_received_message(&self, msg: &str) -> Result<Vec<Reply>, ErrorServer> {
        match is_dcc_chat(msg) {
            true => self.execute_dcc_commands(msg),
            false => self.execute_irc_commands(msg),
        }
    }

    // Function that receives an DCC message formatted as a
    // string and decides how to handle it according to the
    // command that travels in it.
    fn execute_dcc_commands(&self, msg: &str) -> Result<Vec<Reply>, ErrorServer> {
        println!("DCC MESSAGE {:?}", msg);
        let command = DccMessage::from_str(msg)?;
        self.execute_dcc_command(command)
    }

    // Function that receives an IRC message formatted as a
    // string and decides how to handle it according to the
    // command that travels in it.
    fn execute_irc_commands(&self, msg: &str) -> Result<Vec<Reply>, ErrorServer> {
        println!("IRC COMMAND {:?}", msg);
        match Message::from_str(msg) {
            Err(e) => Err(e.into()),
            Ok(command) => match Self::command_need_special_treatment(&command) {
                true => self.execute_irc_command_as_server(command),
                false => self.execute_irc_command(command),
            },
        }
    }

    // Messages that needs to be handled by the Server, and it's different
    // from the usual treatment that occurs in the UserHandler.
    fn command_need_special_treatment(c: &Message) -> bool {
        matches!(
            c.command(),
            Command::Squit
                | Command::Server
                | Command::User
                | Command::Notice
                | Command::Nick
                | Command::Kill
        )
    }

    // Messages that needs to be handled by the Server
    fn execute_irc_command_as_server(&self, c: Message) -> Result<Vec<Reply>, ErrorServer> {
        match c.command() {
            Command::Squit => {
                let squit = SquitMsg::new(&c)?;
                let result =
                    squit.response(&self.nick_sender, &self.channel_sender, &self.spanning_tree)?;
                print!(
                    "POST-SQUIT | SPANNING TREE: {:?}",
                    self.spanning_tree.lock().unwrap()
                );
                if let Some(prefix) = c.prefix() {
                    inform_all_servers(&self.spanning_tree, &self.nick_sender, &prefix, c)?;
                }
                Ok(result)
            }
            Command::Server => {
                let msg = ServerMsg::new(c)?;
                msg.response(self.get_servers())?;
                thread::sleep(Duration::from_millis(150));
                print!(
                    "POST-SERVER | SPANNING TREE: {:?}",
                    self.spanning_tree.lock().unwrap()
                );
                Ok(vec![Reply::rpl_none()])
            }
            Command::User => {
                let msg = UserMsg::new(c);
                msg.response(self.nick_sender.clone(), self.get_servers());
                Ok(vec![Reply::rpl_none()])
            }

            Command::Notice => {
                let msg = NoticeMsg::new(c, self.get_servers())?;
                msg.response(self.nick_sender.clone(), self.channel_sender.clone())?;
                Ok(vec![Reply::rpl_none()])
            }

            Command::Nick => {
                match NickCommand::new(c) {
                    Ok(cmd) => {
                        cmd.response(
                            self.nick_sender.clone(),
                            self.channel_sender.clone(),
                            self.get_servers(),
                        )?;
                    }
                    Err(_r) => {}
                };
                Ok(vec![Reply::rpl_none()])
            }
            Command::Kill => {
                let msg = Kill::new(c, true)?;
                msg.response(self.nick_sender.clone(), self.get_servers())?;
                Ok(vec![Reply::rpl_none()])
            }
            _ => Err(ErrorServer::UnexpectedCommand),
        }
    }

    // Method that delegates the execution of an IRC command to the UserHandler.
    fn execute_irc_command(&self, command: Message) -> Result<Vec<Reply>, ErrorServer> {
        UserHandler::execute_irc_command_from(
            command,
            &mut User::new_empty(),
            &self.get_servers(),
            &self.nick_sender,
            &self.channel_sender,
        )
    }

    // Method that delegates the execution of an DCC command to the UserHandler.
    fn execute_dcc_command(&self, c: DccMessage) -> Result<Vec<Reply>, ErrorServer> {
        UserHandler::execute_dcc_command_from(
            c,
            &mut User::new_empty(),
            &self.get_servers(),
            &self.nick_sender,
            &self.channel_sender,
        )
    }
}

impl Runnable for ServerComunicationHandler {
    fn run(&mut self, socket: &mut TcpStream) -> Result<(), ErrorServer> {
        self.listen_to_server(socket)
    }
}

// It's expected that server's ip:port travels in the SERVER_IP_PORT_POSITION place.
fn get_destination_host_from_handshake(server_msg: &str) -> String {
    let lines: Vec<String> = server_msg.split_whitespace().map(str::to_string).collect();
    lines[SERVER_IP_PORT_POSITION].to_string()
}

// The server_msg is expected to be a message with an ip prefixed, so to build de Message, we ignore the first element.
fn build_msg_from_handshake(server_msg: &str) -> Result<Message, ErrorMsg> {
    let mut lines: Vec<String> = server_msg.split_whitespace().map(str::to_string).collect();
    lines = if "SQUIT".eq(server_msg.to_uppercase().trim()) {
        lines
    } else {
        lines.remove(SERVER_IP_PORT_POSITION);
        lines
    };
    Message::from_str(&lines.join(" "))
}
