use std::{
    fs,
    io::{self, Write},
    net::TcpStream,
    path::Path,
};

use sha::{sha256::Sha256, utils::DigestExt};

use crate::{
    dcc::{dcc_connection::Stream, dcc_handler::DccHandler, download::Download, zipper},
    error::{error_command::ErrorCommand, error_msg::ErrorMsg, error_server::ErrorServer},
    parser::dcc_message::DccMessage,
};

/// Represents a DCC SEND command, providing functionality for handling file transfers.
#[derive(Debug, Clone)]
pub struct DccSend {
    /// The identifier associated with the DCC SEND command.
    pub id: String,
    /// The IP address associated with the DCC SEND command.
    pub ip: String,
    /// The port number associated with the DCC SEND command.
    pub port: String,
    /// The filename being transferred in the DCC SEND command.
    pub filename: String,
    /// Flag indicating whether the file should be zipped before transfer.
    pub zip: bool,
    /// The original DccMessage containing the SEND command details.
    pub msg: DccMessage,
}

impl DccSend {
    /// Creates a new instance of `DccSend` from a DccMessage.
    ///
    /// # Arguments
    ///
    /// * `msg` - The DccMessage containing the parameters for DCC SEND.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `DccSend` or an `ErrorMsg` if parsing fails.
    pub fn new(msg: DccMessage) -> Result<Self, ErrorMsg> {
        let id = msg.target_user();
        let ip = msg.get_param_from_msg(1);
        let port = msg.get_param_from_msg(2);
        let filename = msg.get_param_from_msg(0);
        let zip = msg.get_param_from_msg(3).is_some();

        if let (Some(ip), Some(port), Some(filename), Some(id)) = (ip, port, filename, id) {
            Ok(Self {
                id,
                ip,
                port,
                filename,
                msg,
                zip,
            })
        } else {
            Err(ErrorMsg::InvalidMsg(ErrorCommand::MissingParametersDcc(
                super::DccCommand::Send,
            )))
        }
    }

    /// Sends a response for the DCC SEND command.
    ///
    /// # Arguments
    ///
    /// * `dcc_handler` - The DccHandler responsible for handling the DCC connection.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure in sending the response.
    pub fn response<T>(&mut self, mut dcc_handler: DccHandler<T>) -> Result<(), ErrorServer>
    where
        T: Stream,
    {
        if self.zip {
            let result = zipper::file_to_zip(self.filename.as_str())?;
            self.filename = result.to_string_lossy().to_string();
        }
        let path = Path::new(&self.filename);
        if let Some(file_name) = path.file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                println!("Nombre del archivo: {}", file_name_str);
                let metadata = fs::metadata(path)?;
                let mut bytes = fs::File::open(path)?;
                let mut hasher = Sha256::default();
                io::copy(&mut bytes, &mut hasher)?;
                hasher.flush()?;
                let bytes = hasher.to_hex();

                let size = metadata.len();
                dcc_handler.write_all(
                    format!(
                        "privmsg {} dcc send {} {} {} {} {}",
                        self.id, file_name_str, self.ip, self.port, size, bytes
                    )
                    .as_bytes(),
                )?;
            } else {
                println!("No se pudo convertir el nombre del archivo a una cadena válida.");
                return Err(ErrorServer::BadQuery);
            }
        } else {
            println!("La ruta no contiene un nombre de archivo válido.");
            return Err(ErrorServer::BadQuery);
        }

        Ok(())
    }

    /// Handles the reception of the DCC SEND command, creating and returning a Download instance.
    ///
    /// # Arguments
    ///
    /// * `dcc_connection` - The DccHandler responsible for handling the DCC connection.
    /// * `stream` - The TcpStream associated with the DCC connection.
    ///
    /// # Returns
    ///
    /// A `Result` containing the created `Download` instance or an `ErrorServer` if reception fails.
    pub fn reception<T>(
        &self,
        dcc_connection: &mut DccHandler<T>,
        stream: TcpStream,
    ) -> Result<Download<TcpStream>, ErrorServer>
    where
        T: Stream,
    {
        let _id = self.id.clone();
        let size = self.msg.get_param_from_msg(3);
        let hash = self.msg.get_param_from_msg(4);
        let filename = self.filename.clone();

        if let (Some(size), Some(hash)) = (size, hash) {
            if let Ok(size) = size.parse::<usize>() {
                let download = Download::new(
                    format!("{}:{}", self.ip(), self.port()),
                    stream,
                    filename,
                    size,
                    hash,
                );
                dcc_connection.add_download(download.clone())?;
                Ok(download)
            } else {
                Err(ErrorServer::BadQuery)
            }
        } else {
            Err(ErrorServer::BadQuery)
        }
    }

    /// Gets the IP associated with the DCC SEND command.
    pub fn ip(&self) -> String {
        self.ip.clone()
    }

    /// Gets the identifier associated with the DCC SEND command.
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Gets the port associated with the DCC SEND command.
    pub fn port(&self) -> String {
        self.port.clone()
    }

    /// Gets the filename associated with the DCC SEND command.
    pub fn filename(&self) -> String {
        self.filename.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Read, str::FromStr};

    #[derive(Clone)]
    struct MockTcpStream {
        inner: std::io::Cursor<Vec<u8>>,
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

        fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
            self.inner.write_all(buf)
        }
    }

    impl Read for MockTcpStream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.inner.read(buf)
        }
    }

    #[test]
    fn test_dcc_send_new_valid() {
        let dcc_message =
            DccMessage::from_str("privmsg pepe DCC SEND file localhost 8080").unwrap();
        let dcc_send = DccSend::new(dcc_message);
        assert!(dcc_send.is_ok());

        let dcc_send = dcc_send.unwrap();
        assert_eq!(dcc_send.id, "pepe");
        assert_eq!(dcc_send.ip, "localhost");
        assert_eq!(dcc_send.port, "8080");
        assert_eq!(dcc_send.filename, "file");
        assert!(!dcc_send.zip);
    }

    #[test]
    fn test_dcc_send_new_invalid() {
        // Test case with missing parameters
        let dcc_message = DccMessage::from_str("privmsg pepe DCC SEND").unwrap();
        let dcc_send = DccSend::new(dcc_message);
        assert!(dcc_send.is_err());
    }
}
