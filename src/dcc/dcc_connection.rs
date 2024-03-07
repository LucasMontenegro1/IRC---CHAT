use std::{
    io::{Read, Write},
    str::FromStr,
    sync::mpsc::Sender,
};

use super::command::text_msg::DccTextMsg;
use crate::{error::error_server::ErrorServer, parser::dcc_message::DccMessage};

/// A trait representing a stream that can be cloned.
///
/// This trait combines `Write`, `Read`, and `Send` traits and adds a method `try_clone`.
pub trait Stream: Write + Read + Send {
    fn try_clone(&self) -> std::io::Result<Self>
    where
        Self: Sized;
}

// Implementation of the Stream trait for TcpStream.
impl Stream for std::net::TcpStream {
    fn try_clone(&self) -> std::io::Result<Self> {
        self.try_clone()
    }
}

/// Struct representing a direct message.
#[derive(Debug, PartialEq, Eq)]
pub struct DirectMessage(pub String, pub String, pub String);

impl DirectMessage {
    /// Gets the TCP address from the message.
    pub fn get_tcp_address(&self) -> String {
        self.0.clone()
    }

    /// Gets the user from the message.
    pub fn get_user(&self) -> String {
        self.1.clone()
    }

    /// Gets the message content.
    pub fn get_msg(&self) -> String {
        self.2.clone()
    }
}

/// Represents a connection for Direct Client-to-Client (DCC) communication.
#[derive(Debug)]
pub struct DccConnection<S>
where
    S: Stream + 'static, // Add Stream as a trait bound for the generic type S
{
    /// The user associated with the connection.
    user: String,

    /// The unique identifier for the connection.
    id: String,

    /// The stream associated with the connection, wrapped in a Result to handle potential errors.
    stream: Result<S, ErrorServer>,

    /// A sender channel for sending direct messages.
    sender: Sender<DirectMessage>,
}

impl<S: Stream + 'static> DccConnection<S> {
    /// Creates a new DccConnection instance.
    ///
    /// # Arguments
    ///
    /// * `user` - The user associated with the connection.
    /// * `stream` - The stream associated with the connection.
    /// * `id` - The identifier for the connection.
    /// * `sender` - The sender for direct messages.
    pub fn new(user: String, stream: S, id: String, sender: Sender<DirectMessage>) -> Self {
        Self {
            user,
            stream: Ok(stream),
            id,
            sender,
        }
    }

    /// Receives messages on the connection.
    pub fn receive_msgs(&mut self) -> Result<(), ErrorServer> {
        loop {
            let mut buff: [u8; 512] = [b' '; 512];
            let msg = match self.read(&mut buff) {
                Ok(0) => {
                    // Si se leen 0 bytes, la conexión está cerrad
                    return Ok(());
                }
                Ok(n) => n,
                Err(err) => {
                    // Verificar si es un error de conexión cerrada
                    if err.kind() == std::io::ErrorKind::ConnectionAborted
                        || err.kind() == std::io::ErrorKind::ConnectionReset
                    {
                        return Ok(());
                    } else {
                        return Err(ErrorServer::from(err));
                    }
                }
            };

            let c = String::from_utf8_lossy(&buff[..msg]);
            let msg_str = c.as_ref().trim();
            if let Ok(message) = DccMessage::from_str(msg_str) {
                match message.command() {
                    crate::dcc::command::DccCommand::Close => {
                        return Ok(());
                    }
                    _ => continue,
                }
            }
            println!("dcc message from -> {}: {}", self.user, msg_str);
            self.sender.send(DirectMessage(
                self.id.clone(),
                self.user.clone(),
                msg_str.to_owned(),
            ))?;
        }
    }

    /// Sends a DCC message on the connection.
    pub fn send_msg(self, dcc_message: DccMessage) -> Result<(), ErrorServer>
    where
        S: Stream,
    {
        match dcc_message.command() {
            super::command::DccCommand::Chat => Ok(()),
            super::command::DccCommand::Send => Ok(()),
            super::command::DccCommand::Resume => Ok(()),
            super::command::DccCommand::Accept => Ok(()),
            super::command::DccCommand::Close => Ok(()),
            super::command::DccCommand::Pause => Ok(()),
            super::command::DccCommand::MSG => {
                let msg = DccTextMsg::new(dcc_message)?;
                msg.response(self)?;
                Ok(())
            }
        }
    }

    /// Gets the user associated with the connection.
    pub fn get_user(self) -> String {
        self.user
    }

    fn see_if_clonable(&self) -> Result<S, ErrorServer> {
        match &self.stream {
            Ok(c) => match c.try_clone() {
                Ok(c) => Ok(c),
                Err(_) => Err(ErrorServer::TcpFail),
            },
            Err(_) => Err(ErrorServer::TcpFail),
        }
    }

    pub fn close(self) {
        if let Ok(s) = self.stream {
            drop(s);
        }
    }

    pub fn get_stream(self) -> Result<S, ErrorServer> {
        self.stream
    }

    pub fn get_id(self) -> String {
        self.id
    }
}

impl<S: Stream> Clone for DccConnection<S> {
    fn clone(&self) -> Self {
        Self {
            user: self.user.clone(),
            stream: self.see_if_clonable(),
            id: self.id.clone(),
            sender: self.sender.clone(),
        }
    }
}

impl<S: Stream> Write for DccConnection<S> {
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

impl<S: Stream> Read for DccConnection<S> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.stream {
            Ok(s) => s.read(buf),
            Err(_) => Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Cursor, sync::mpsc};

    fn create_dcc_connection() -> DccConnection<MockTcpStream> {
        let (tx_dcc, _rx_dcc): (mpsc::Sender<DirectMessage>, mpsc::Receiver<DirectMessage>) =
            mpsc::channel();
        let user = "test_user".to_string();
        let stream = MockTcpStream::new(&[]);
        DccConnection::new(user.clone(), stream, "1".to_string(), tx_dcc)
    }

    // MockTcpStream to simulate TcpStream behavior

    // MockTcpStream to simulate TcpStream behavior
    #[derive(Clone)]
    struct MockTcpStream {
        inner: Cursor<Vec<u8>>,
    }

    impl MockTcpStream {
        fn new(data: &[u8]) -> Self {
            MockTcpStream {
                inner: Cursor::new(data.to_vec()),
            }
        }
    }

    impl Stream for MockTcpStream {
        fn try_clone(&self) -> std::io::Result<Self> {
            Ok(self.clone())
        }
    }

    impl Write for MockTcpStream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.inner.write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.inner.flush()
        }
    }

    impl Read for MockTcpStream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.inner.read(buf)
        }
    }
    #[test]
    fn test_new_dcc_connection() {
        let dcc_connection = create_dcc_connection();
        assert_eq!(dcc_connection.user, "test_user".to_string());
        assert!(dcc_connection.stream.is_ok());
    }

    #[test]
    fn test_clone_dcc_connection() {
        let dcc_connection = create_dcc_connection();
        let cloned_dcc_connection = dcc_connection.clone();

        assert_eq!(dcc_connection.user, cloned_dcc_connection.user);
        assert!(dcc_connection.stream.is_ok());
        assert!(cloned_dcc_connection.stream.is_ok());
    }
    #[test]
    fn test_write_and_flush() {
        let mut dcc_connection = create_dcc_connection();

        let data = "Test data".as_bytes();
        dcc_connection.write_all(data).unwrap();
        dcc_connection.flush().unwrap();

        // Reset the cursor position to the beginning before reading
        dcc_connection
            .stream
            .as_mut()
            .unwrap()
            .inner
            .set_position(0);

        // Check the actual output against the expected output
        let mut output = Vec::new();
        dcc_connection
            .stream
            .as_mut()
            .unwrap()
            .read_to_end(&mut output)
            .unwrap();
        assert_eq!(&output, data);
    }

    #[test]
    fn test_receive_msgs() {
        let (tx_dcc, rx_dcc): (mpsc::Sender<DirectMessage>, mpsc::Receiver<DirectMessage>) =
            mpsc::channel();
        let user = "test_user".to_string();
        let stream = MockTcpStream::new(b"Test DCC Message");
        let mut dcc_connection = DccConnection::new(user.clone(), stream, "1".to_string(), tx_dcc);

        dcc_connection.receive_msgs().unwrap();

        let received_msg = rx_dcc.recv().unwrap();
        assert_eq!(received_msg.get_user(), user);
        assert_eq!(received_msg.get_msg(), "Test DCC Message");
    }

    #[test]
    fn user_gets_correctly() {
        let user = "test_user".to_string();
        let dcc_connection = create_dcc_connection();
        assert_eq!(dcc_connection.get_user(), user)
    }
}
