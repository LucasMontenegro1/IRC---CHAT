use super::server_connection::ConnectionServer;
use crate::command::Command;
use crate::parser::{convert_into_command_prefix, message::Message};
use std::io::Write;

const MIN_HOPCOUNT: usize = 1;
///
/// struct that models a
/// server within the irc
/// network
///
#[derive(Clone, Debug)]
pub struct Server {
    pub servername: String,
    connection: Option<ConnectionServer>,
}

impl Server {
    ///
    ///function that creates a
    /// new server with your
    /// connection (optional)
    ///
    pub fn new(servername: String, connection: Option<ConnectionServer>) -> Self {
        Server {
            servername,
            connection,
        }
    }

    ///
    /// returns the connection to a
    /// server in case it exists
    ///
    pub fn get_connection(&self) -> Option<ConnectionServer> {
        self.connection.clone()
    }
    ///
    /// st the connection to a
    /// server
    ///
    pub fn set_connection(&mut self, connection: ConnectionServer) {
        self.connection = Some(connection)
    }

    pub fn build_server_message_as_neighbour(&self, destination: &Server) -> String {
        self.build_server_message_as_source(destination, MIN_HOPCOUNT)
    }

    pub fn build_server_message_as_source(&self, destination: &Server, hopcount: usize) -> String {
        Self::build_server_message(self, destination, hopcount)
    }

    pub fn build_server_message_as_destination(&self, source: &Server, hopcount: usize) -> String {
        Self::build_server_message(source, self, hopcount)
    }

    fn build_server_message(source: &Server, destination: &Server, hopcount: usize) -> String {
        Self::build_message(&source.servername, &destination.servername, hopcount)
    }

    fn build_message(source: &str, destination: &str, hopcount: usize) -> String {
        let parameters = vec![
            destination.to_string(),
            hopcount.to_string(),
            ":Nueva conexion".to_string(),
        ];
        let msg = Message::new(
            Some(convert_into_command_prefix(source)),
            Command::Server,
            Some(parameters),
        );
        //println!("Builded message {}", msg.to_string());
        msg.to_string()
    }
}

///
/// implementation of PartialEq
/// for the server struct
///
impl PartialEq for Server {
    fn eq(&self, other: &Self) -> bool {
        self.servername == other.servername
    }
}

impl Write for Server {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.connection {
            Some(s) => s.write(buf),
            None => Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.connection {
            Some(s) => s.flush(),
            None => Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match &mut self.connection {
            Some(s) => s.write_all(buf),
            None => Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::server_comunication::server::Server;

    #[test]
    fn equals() {
        let server1 = Server::new("server1".to_string(), None);
        let server2 = Server::new("server1".to_string(), None);
        let result = server2 == server1;
        assert!(result);
    }

    #[test]
    fn contains() {
        let server1 = Server::new("server1".to_string(), None);
        let server = server1.clone();
        let vec = vec![server1];
        let c = vec.contains(&server);
        assert!(c);
    }
}
