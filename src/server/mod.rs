use crate::channel::Channel;
use crate::command::traits::Runnable;
use crate::command::Command;
use crate::database::RepositoryHandler;
use crate::error::error_server::ErrorServer;
use crate::parser::message::Message;
use crate::repository::connection::Connection;
use crate::repository::repository_channel::client_channel::ClientChannel;
use crate::server_comunication::server::Server;
use crate::server_comunication::spanning_tree::node::Node;
use crate::server_comunication::spanning_tree::SpanningTree;
use crate::server_comunication::ServerComunicationHandler;
use crate::user::user_handler::UserHandler;
use crate::utils::{read_message_from, write_message_to};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// Struct that encapsulates the information of a server needs to handle, in order
/// to be part of a irc connections. Is in charge to stores information about
/// [`Connection`] from users, servers with [`SpanningTree`] and [`Channel`] that
/// exists across the network.
pub struct MainServer {
    nicknames: RepositoryHandler<String, Connection>,
    channels: RepositoryHandler<String, Channel>,
    spanning_tree: Arc<Mutex<SpanningTree>>,
}

impl MainServer {
    /// Constructor
    ///
    /// # Arguments
    /// * `server_name` - Name of the server
    ///
    /// # Returns
    ///  If the [`RepositoryHandler`] entities, in charge of the communication with
    /// persistion entities, are created succesfully, returns the MainServer entity
    /// with a new [`SpanningTree`] with his own as a root.
    pub fn new(server_name: String) -> Result<Self, ErrorServer> {
        // create spanning tree with the server
        let root = Node::new(Server::new(server_name, None));
        let st = SpanningTree::new(root, vec![]);
        let spanning_tree = Arc::new(Mutex::new(st));
        // Create & Run a repositorys.
        Ok(MainServer {
            nicknames: RepositoryHandler::new()?,
            channels: RepositoryHandler::new()?,
            spanning_tree,
        })
    }

    //Initialize a ServerComunicationHandler
    fn build_server_comunication(&self) -> Result<ServerComunicationHandler, ErrorServer> {
        Ok(ServerComunicationHandler::new(
            self.get_servername()?,
            self.get_servers(),
            self.get_nick_repository_channels(),
            self.get_channels_repository_channels(),
        ))
    }

    //Initialize a UserHandler
    fn build_user_handler(&self) -> UserHandler {
        UserHandler::new(
            self.get_nick_repository_channels(),
            self.get_channels_repository_channels(),
            self.get_servers(),
        )
    }

    // Getter of the spanning tree
    fn get_servers(&self) -> Arc<Mutex<SpanningTree>> {
        self.spanning_tree.clone()
    }

    // Getter of the spanning tree's root name, is the same of the MainServer.
    fn get_servername(&self) -> Result<String, ErrorServer> {
        if let Ok(sp) = self.spanning_tree.lock() {
            return Ok(sp.get_root_name());
        }
        Err(ErrorServer::LockedResource)
    }

    // Getter of the channel which encapsulates the communication with the persistance of <nickname, connection>
    fn get_nick_repository_channels(&self) -> ClientChannel<String, Connection> {
        self.nicknames.get_channels()
    }

    // Getter of the channel which encapsulates the communication with the persistance of <channel's name, Channel>
    fn get_channels_repository_channels(&self) -> ClientChannel<String, Channel> {
        self.channels.get_channels()
    }

    //Attempts to register a client or a server, depending on the message received by the main server.
    fn handle_incoming_connection(
        &self,
        mut stream: TcpStream,
        addr: SocketAddr,
    ) -> Result<JoinHandle<Result<(), ErrorServer>>, ErrorServer> {
        let servername = self.get_servername()?;
        let user_handler = self.build_user_handler();
        let server_communication = self.build_server_comunication()?;
        //REGISTRATION
        let th = thread::Builder::new().name(addr.to_string()).spawn(
            move || -> Result<(), ErrorServer> {
                let mut element = Self::registration(
                    user_handler,
                    server_communication,
                    &servername,
                    &mut stream,
                )?;
                // init
                element.run(&mut stream.try_clone()?)
            },
        )?;
        Ok(th)
    }

    //Attempts to register a client or a server, depending on the message received by the main server.
    fn registration(
        mut user_handler: UserHandler,
        mut server_comunication: ServerComunicationHandler,
        servername: &str,
        socket: &mut TcpStream,
    ) -> Result<Box<dyn Runnable>, ErrorServer> {
        let mut client = socket.try_clone()?;
        loop {
            let msg = read_message_from(&mut client)?;
            match Message::from_str(&msg) {
                Ok(message) => {
                    return match message.command() {
                        Command::User | Command::Nick | Command::Pass => {
                            //println!("Registro de un Usuario: user_registration( msg, servername, socket )");
                            user_handler.user_registration(message, servername, socket)
                        }
                        Command::Server => {
                            //println!("Registro de un Servidor: register_server( msg, socket )");
                            server_comunication.register_server(message, socket)
                        }
                        _ => continue,
                    };
                }
                Err(e) => write_message_to(&format!("{e:}"), &mut client)?,
            }
        }
    }
}

/// Initiates [`MainServer`]  entity with all information neccesary to run a server of
/// the irc network and opens for new connections to handle.
/// Also allows CLI communication in orden to commands server-to-server connections.
pub fn init_server(ip: String, name: String) -> Result<(), ErrorServer> {
    // Create the connection of the server, ready to accept connections.
    let server = MainServer::new(name.clone())?;

    //Starts command line communication
    let cli_thread = server.build_server_comunication()?.run_cli();

    let listener = TcpListener::bind(ip.as_str())?;
    let _msg_thread = thread::spawn(move || -> Result<(), ErrorServer> {
        loop {
            if let Ok((socket, addr)) = listener.accept() {
                server.handle_incoming_connection(socket, addr)?;
            }
        }
    });
    match cli_thread.join() {
        Ok(r) => r,
        Err(_) => Err(ErrorServer::PoisonedThread),
    }
}
