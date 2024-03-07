use crate::error::error_server::ErrorServer;
use std::io::Write;
use std::net::TcpStream;

///
/// struct that implements a connection
/// a connection between servers. it has
/// a tcp stream and makes it clonable so it
/// can be used in the repositories and spanning trees.
///
#[derive(Debug)]
pub struct ConnectionServer {
    stream: Result<TcpStream, ErrorServer>,
}

impl ConnectionServer {
    ///
    /// function that creates a new connection
    ///
    pub fn new(stream: TcpStream) -> Self {
        Self { stream: Ok(stream) }
    }
    ///
    /// function that tries to clone
    /// the tcp stream into the connection
    ///
    fn see_if_clonable(&self) -> Result<TcpStream, ErrorServer> {
        match &self.stream {
            Ok(c) => match c.try_clone() {
                Ok(c) => Ok(c),
                Err(_) => Err(ErrorServer::TcpFail),
            },
            Err(_) => Err(ErrorServer::TcpFail),
        }
    }
}

impl Clone for ConnectionServer {
    ///
    /// clones a connection
    ///
    fn clone(&self) -> Self {
        Self {
            stream: self.see_if_clonable(),
        }
    }
}

///
/// implementation of the trait
/// write for connection so it
/// can be used to send messages
/// through the tcp stream
///
///
impl Write for ConnectionServer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.stream {
            Ok(s) => s.write(buf),
            Err(_) => Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.stream {
            Ok(s) => s.flush(),
            Err(_) => Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
        }
    }
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match &mut self.stream {
            Ok(s) => s.write_all(buf),
            Err(_) => Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
        }
    }
}
