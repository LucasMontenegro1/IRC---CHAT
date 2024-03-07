use crate::server_comunication::server::Server;
use crate::server_comunication::server_connection::ConnectionServer;

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    pub server: Server,
}

impl Node {
    pub fn new(server: Server) -> Self {
        Node { server }
    }

    pub fn get_element(&self) -> Server {
        self.server.clone()
    }

    pub fn set_connection(&mut self, connection: ConnectionServer) {
        self.server = Server::new(self.server.servername.clone(), Some(connection));
    }
}
