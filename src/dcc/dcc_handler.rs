use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::error::error_command::ErrorCommand;
use crate::error::error_msg::ErrorMsg;
use crate::error::error_server::ErrorServer;
use crate::parser::dcc_message::DccMessage;

use super::command::accept::DccAccept;
use super::command::chat::DccChat;
use super::command::close::DccClose;
use super::command::pause::DccPause;
use super::command::resume::DccResume;
use super::command::send::DccSend;
use super::command::text_msg::DccTextMsg;
use super::dcc_connection::{DccConnection, DirectMessage, Stream};
use super::download::Download;
use super::upload::Upload;

/// Manages DCC (Direct Client-to-Client) connections and handles incoming messages and file transfers.
#[derive(Debug)]
pub struct DccHandler<S>
where
    S: Stream + 'static,
{
    /// The underlying stream associated with the DCC handler.
    stream: Result<S, ErrorServer>,
    /// Active DCC connections.
    active_connections: Arc<Mutex<Vec<DccConnection<TcpStream>>>>,
    /// Active uploads.
    uploads: Arc<Mutex<Vec<Upload<TcpStream>>>>,
    /// Active downloads.
    downloads: Arc<Mutex<Vec<Download<TcpStream>>>>,
    /// Sender channel for direct messages.
    sender: Sender<DirectMessage>,
}

impl<S: Stream + 'static> DccHandler<S> {
    /// Creates a new instance of `DccHandler` with the specified stream and message sender.
    ///
    /// # Arguments
    ///
    /// * `stream` - The stream used by the handler.
    /// * `sender` - The message sender for sending direct messages.
    ///
    /// # Returns
    ///
    /// A new instance of `DccHandler` with the provided `stream` and `sender`.
    pub fn new(stream: S, sender: Sender<DirectMessage>) -> Self {
        DccHandler {
            stream: Ok(stream),
            active_connections: Arc::new(Mutex::new(Vec::new())),
            uploads: Arc::new(Mutex::new(Vec::new())),
            downloads: Arc::new(Mutex::new(Vec::new())),
            sender,
        }
    }

    /// Adds a new DCC connection client to the list of active connections.
    ///
    /// # Arguments
    ///
    /// * `dcc_connection` - The DccConnection to be added to the list of active connections.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success (`Ok(())`) or an `ErrorServer` if there was an issue with the lock.
    pub fn add_client(&self, dcc_connection: DccConnection<TcpStream>) -> Result<(), ErrorServer> {
        match self.active_connections.lock() {
            Ok(mut clients) => {
                clients.push(dcc_connection);
                Ok(())
            }
            Err(_) => Err(ErrorServer::PoisonedThread),
        }
    }

    /// Initiates a new DCC connection to a remote user's IP address.
    ///
    /// # Arguments
    ///
    /// * `user` - The username associated with the remote user.
    /// * `ip` - The IP address of the remote user to connect to.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success (`Ok(())`) or an `ErrorServer` if there was an issue with the connection.
    pub fn connect_to(&self, user: String, ip: String) -> Result<(), ErrorServer> {
        println!("USER: {:?}", user);

        let handler = self.clone();
        thread::spawn(move || -> Result<(), ErrorServer> {
            let stream = TcpStream::connect(ip.clone())?;
            let mut dcc_connection =
                DccConnection::new(user.clone(), stream, ip.clone(), handler.sender.clone());
            handler.add_client(dcc_connection.clone())?;
            println!("Conectando punto a punto con usuario : {}", user);
            dcc_connection.receive_msgs()?;
            println!("Conexion punto a punto cerrada");
            handler.close_connection(ip.clone())?;
            dcc_connection.close();
            Ok(())
        });
        Ok(())
    }

    /// Initiates a new DCC connection by listening on a specified IP address.
    ///
    /// # Arguments
    ///
    /// * `user` - The username associated with the remote user.
    /// * `ip` - The IP address to bind the listener for incoming connections.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success (`Ok(())`) or an `ErrorServer` if there was an issue with the connection.
    pub fn new_connection(&self, user: String, ip: String) -> Result<(), ErrorServer> {
        match TcpListener::bind(ip.as_str()) {
            Ok(listener) => {
                println!("USER: {:?}", user);
                let handler = self.clone();
                thread::spawn(move || -> Result<(), ErrorServer> {
                    let client = listener.accept().expect("Failed to accept connection");
                    let mut dcc_connection = DccConnection::new(
                        user.clone(),
                        client.0,
                        ip.clone(),
                        handler.sender.clone(),
                    );
                    handler.add_client(dcc_connection.clone())?;
                    println!("Conexion punto a punto recibida");
                    dcc_connection.receive_msgs()?;
                    println!("Conexion punto a punto cerrada");
                    handler.close_connection(ip.clone())?;
                    dcc_connection.close();
                    Ok(())
                });
            }
            Err(e) => println!("Error connecting : {}", e),
        }
        Ok(())
    }

    /// Closes a DCC connection based on the specified IP address.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address associated with the connection to be closed.
    ///
    /// # Returns
    ///
    /// A `Result` containing the closed `DccConnection` if successful, or an `ErrorServer` if the
    /// connection is not found.
    pub fn close_connection(&self, ip: String) -> Result<DccConnection<TcpStream>, ErrorServer> {
        match self.active_connections.lock() {
            Ok(mut c) => {
                // Buscar la conexión por IP y eliminarla si se encuentra
                for (index, connection) in c.iter().enumerate() {
                    if connection.clone().get_id() == ip {
                        // Remover la conexión de active_connections
                        let stopped_connection = c.remove(index);
                        return Ok(stopped_connection);
                    }
                }
                Err(ErrorServer::UnreachableClient)
            }
            Err(_) => Err(ErrorServer::UnreachableClient),
        }
    }

    /// Handles DCC messages related to sending files, chatting, resuming, accepting, closing, pausing, or sending text messages.
    ///
    /// # Arguments
    ///
    /// * `msg` - The DCC message string to be processed.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure. In case of success, `Ok(())` is returned; otherwise,
    /// an `ErrorMsg` is returned, specifying the type of error encountered.
    pub fn handle_dcc_message_send(&mut self, msg: String) -> Result<(), ErrorMsg> {
        let m = DccMessage::from_str(&msg);
        match m {
            Ok(message) => match message.command() {
                super::command::DccCommand::Chat => {
                    let command = DccChat::new(message)?;
                    let ip_port = format!("{}:{}", command.ip(), command.port());
                    if let Err(listener) = TcpListener::bind(ip_port) {
                        println!("Puerto no disponible para el envio");
                        drop(listener);
                        return Err(ErrorMsg::ServerError(ErrorServer::LockedResource));
                    }
                    command.response(self)?;
                    Ok(())
                }
                super::command::DccCommand::Send => {
                    let command = DccSend::new(message.clone())?;
                    let mut handler_clone = self.clone();
                    let mut command_clone = command.clone();
                    let ip_port = format!("{}:{}", command.ip(), command.port());
                    if let Err(listener) = TcpListener::bind(ip_port) {
                        println!("Puerto no disponible para el envio");
                        drop(listener);
                        return Err(ErrorMsg::ServerError(ErrorServer::LockedResource));
                    }
                    thread::spawn(move || -> Result<(), ErrorServer> {
                        command_clone.response(handler_clone.clone())?;
                        let ip_port = format!("{}:{}", command.ip(), command.port());
                        let ip_port_clone = ip_port.clone();
                        let listener = TcpListener::bind(ip_port)?;
                        let client = listener.accept().expect("Failed to accept connection");
                        println!("Conexion send aceptada : enviando archivo");
                        let upload =
                            Upload::new(ip_port_clone.clone(), client.0, command_clone.filename());
                        let upload_clone = upload.clone();
                        if handler_clone.add_upload(upload).is_err() {
                            println!("Falla en la carga {:?}", upload_clone);
                            if handler_clone.close_upload(ip_port_clone).is_ok() {
                                println!("Upload closed");
                            }
                            return Ok(());
                        }
                        if !upload_clone.is_paused()? {
                            println!("Finished upload {:?}", upload_clone);
                            if handler_clone.close_upload(ip_port_clone).is_ok() {
                                println!("Upload closed")
                            }
                        }
                        Ok(())
                    });
                    Ok(())
                }
                super::command::DccCommand::Resume => {
                    let command = DccResume::new(message)?;
                    command.response(self)?;
                    Ok(())
                }
                super::command::DccCommand::Accept => {
                    let command = DccAccept::new(message)?;
                    command.response(self)?;
                    Ok(())
                }
                super::command::DccCommand::Close => {
                    let command = DccClose::new(message)?;
                    let mut connection =
                        self.search_connection(format!("{}:{}", command.ip(), command.port()))?;
                    if connection
                        .write_all(
                            format!("dcc close {} {}", command.ip(), command.port()).as_bytes(),
                        )
                        .is_err()
                    {
                        println!("Error cerrando la conexion");
                        return Ok(());
                    }
                    connection.close();
                    self.close_connection(format!("{}:{}", command.ip(), command.port()))?;
                    Ok(())
                }
                super::command::DccCommand::Pause => {
                    let command = DccPause::new(message)?;
                    command.response(self)?;
                    Ok(())
                }
                super::command::DccCommand::MSG => {
                    let command = DccTextMsg::new(message.clone())?;
                    let connection = self.search_connection(command.get_ip())?;
                    connection.send_msg(message)?;
                    Ok(())
                }
            },
            Err(_) => Err(ErrorMsg::InvalidMsg(ErrorCommand::UnknownCommand)),
        }
    }

    /// Closes a download connection based on the provided IP.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address associated with the download connection to be closed.
    ///
    /// # Returns
    ///
    /// A `Result` containing the closed `Download` on success; otherwise, an `ErrorServer` is returned,
    /// indicating the type of error encountered, such as an unreachable client.
    pub fn close_download(&self, ip: String) -> Result<Download<TcpStream>, ErrorServer> {
        match self.downloads.lock() {
            Ok(mut c) => {
                // Buscar la conexión por IP y eliminarla si se encuentra
                for (index, download) in c.iter().enumerate() {
                    if download.clone().get_id() == ip {
                        // Remover la conexión de active_connections
                        let stopped_download = c.remove(index);
                        return Ok(stopped_download);
                    }
                }
                Err(ErrorServer::UnreachableClient)
            }
            Err(_) => Err(ErrorServer::UnreachableClient),
        }
    }

    /// Closes an upload connection based on the provided IP.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address associated with the upload connection to be closed.
    ///
    /// # Returns
    ///
    /// A `Result` containing the closed `Upload` on success; otherwise, an `ErrorServer` is returned,
    /// indicating the type of error encountered, such as an unreachable client.
    pub fn close_upload(&self, ip: String) -> Result<Upload<TcpStream>, ErrorServer> {
        match self.uploads.lock() {
            Ok(mut c) => {
                // Buscar la conexión por IP y eliminarla si se encuentra
                for (index, upload) in c.iter().enumerate() {
                    if upload.clone().get_id() == ip {
                        // Remover la conexión de active_connections
                        let stopped_upload = c.remove(index);
                        return Ok(stopped_upload);
                    }
                }
                Err(ErrorServer::UnreachableClient)
            }
            Err(_) => Err(ErrorServer::UnreachableClient),
        }
    }

    /// Handles a DCC message during the reception phase.
    ///
    /// # Arguments
    ///
    /// * `msg` - The DccMessage to be processed.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an `ErrorServer` if an error occurs during processing.
    pub fn handle_dcc_message_reception(&mut self, msg: DccMessage) -> Result<(), ErrorServer> {
        match msg.command() {
            super::command::DccCommand::Chat => {
                let command = DccChat::new(msg)?;
                match command.reception(self) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(ErrorServer::TcpFail),
                }
            }
            super::command::DccCommand::Send => {
                if let Ok(send) = DccSend::new(msg.clone()) {
                    println!("{:?}", send.clone());
                    let ip_port = format!("{}:{}", send.ip(), send.port());
                    thread::sleep(Duration::from_millis(150));
                    if let Some(user) = msg.prefix() {
                        let _ = self.sender.send(DirectMessage(
                            ip_port.clone(),
                            user.clone(),
                            format!("Downloading... {}", send.clone().filename()),
                        ));
                    }
                    if let Ok(stream) = TcpStream::connect(ip_port.clone()) {
                        let handler_clone = self.clone();
                        let mut download = send.reception(&mut self.clone(), stream)?;
                        thread::spawn(move || {
                            if download.download_file().is_ok() {
                                println!("Finished download {:?}", download);
                                if let Some(user) = msg.prefix() {
                                    let _ = handler_clone.sender.send(DirectMessage(
                                        ip_port.clone(),
                                        user.clone(),
                                        format!("Downloading Finished {}", send.clone().filename()),
                                    ));
                                }
                                if handler_clone.close_download(ip_port).is_ok() {
                                    println!("Download Closed")
                                }
                            }
                        });

                        Ok(())
                    } else {
                        println!("Error connecting to TCP download");
                        Ok(())
                    }
                } else {
                    println!("Missing parameters in send message");
                    Ok(())
                }
            }
            super::command::DccCommand::Resume => {
                let command = DccResume::new(msg)?;
                if command.reception(&mut self.clone()).is_ok() {
                    if let Some(mut upload) =
                        self.search_upload(format!("{}:{}", command.ip(), command.port()))
                    {
                        thread::spawn(move || {
                            if let Some(offset) = command.offset() {
                                if let Ok(offset) = offset.parse::<u64>() {
                                    let _ = upload.start(offset);
                                }
                            }
                        });
                    }
                }
                Ok(())
            }
            super::command::DccCommand::Accept => {
                println!("Llega mensaje accept");
                Ok(())
            }
            super::command::DccCommand::Close => todo!(),
            super::command::DccCommand::Pause => {
                let command = DccPause::new(msg)?;
                let ip_port = format!("{}:{}", command.ip(), command.port());
                if let Some(upload) = self.clone().search_upload(ip_port) {
                    println!("pausando descarga");
                    let _ = upload.pause();
                }

                Ok(())
            }
            super::command::DccCommand::MSG => todo!(),
        }
    }

    /// Searches for a DCC connection by IP address.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address to search for.
    ///
    /// # Returns
    ///
    /// A `Result` containing the found DccConnection if successful, or an `ErrorServer` if not.
    pub fn search_connection(&self, ip: String) -> Result<DccConnection<TcpStream>, ErrorServer> {
        match self.active_connections.lock() {
            Ok(c) => {
                for connection in c.iter() {
                    if connection.clone().get_id() == ip {
                        return Ok(connection.clone());
                    }
                }
                Err(ErrorServer::UnreachableClient)
            }
            Err(_) => Err(ErrorServer::UnreachableClient),
        }
    }

    /// Adds a new download to the handler's list of active downloads.
    ///
    /// # Arguments
    ///
    /// * `download` - The download to be added.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success if the download was added, or an `ErrorServer`
    pub fn add_download(&mut self, download: Download<TcpStream>) -> Result<(), ErrorServer> {
        let result = self.downloads.lock();
        match result {
            Ok(mut c) => {
                c.push(download.clone());
            }
            Err(_) => return Err(ErrorServer::LockedResource),
        }

        Ok(())
    }

    /// Searches for a specific download by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the download to be searched.
    ///
    /// # Returns
    ///
    /// An `Option` containing the found download if it exists, or `None` otherwise.
    pub fn search_downloads(&mut self, id: String) -> Option<Download<TcpStream>> {
        if let Some(download_mutex) = self.downloads.lock().unwrap().iter().find(|download| {
            println!("{:?}", download);
            println!("{}", id);
            download.get_id() == id
        }) {
            let downloads_clone = download_mutex.clone();
            Some(downloads_clone)
        } else {
            None
        }
    }

    /// Adds a new upload to the handler and starts it.
    ///
    /// # Arguments
    ///
    /// * `upload` - The upload to be added and started.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success (`Ok(())`) or an `ErrorServer` if there was an issue.
    pub fn add_upload(&mut self, mut upload: Upload<TcpStream>) -> Result<(), ErrorServer> {
        let result = self.uploads.lock();
        match result {
            Ok(mut c) => {
                c.push(upload.clone());
            }
            Err(_) => return Err(ErrorServer::LockedResource),
        }
        upload.start(0)?;

        Ok(())
    }

    /// Searches for an upload with the specified ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the upload to search for.
    ///
    /// # Returns
    ///
    /// An `Option` containing the found upload or `None` if not found.
    pub fn search_upload(&mut self, id: String) -> Option<Upload<TcpStream>> {
        if let Some(upload_mutex) = self.uploads.lock().unwrap().iter().find(|upload| {
            println!("{:?}", upload);
            upload.get_id() == id
        }) {
            let upload_clone = upload_mutex.clone();
            Some(upload_clone)
        } else {
            None
        }
    }

    /// Starts an upload with the specified ID and offset.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the upload to start.
    /// * `offset` - The offset to start the upload from.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an `ErrorServer` in case of failure.
    pub fn start_upload(&mut self, id: String, offset: u64) -> Result<(), ErrorServer> {
        if let Some(mut upload) = self.search_upload(id) {
            upload.start(offset)?;
        }
        Ok(())
    }

    /// Checks if the stream is clonable and returns the cloned stream if successful.
    ///
    /// # Returns
    ///
    /// A `Result` containing the cloned stream on success, or an `ErrorServer` on failure.
    fn see_if_clonable(&self) -> Result<S, ErrorServer> {
        match &self.stream {
            Ok(c) => match c.try_clone() {
                Ok(c) => Ok(c),
                Err(_) => Err(ErrorServer::TcpFail),
            },
            Err(_) => Err(ErrorServer::TcpFail),
        }
    }
}

/// Checks if a given message is a DCC chat message.
///
/// # Arguments
///
/// * `msg` - A string representing the message to be checked.
///
/// # Returns
///
/// A boolean value indicating whether the message is a DCC chat message or not.
pub fn is_dcc_chat(msg: &str) -> bool {
    match DccMessage::from_str(msg) {
        Ok(message) => {
            matches!(
                message.command(),
                super::command::DccCommand::Chat
                    | super::command::DccCommand::Send
                    | super::command::DccCommand::Pause
                    | super::command::DccCommand::Accept
                    | super::command::DccCommand::Resume
            )
        }
        Err(_) => false,
    }
}

impl<S: Stream> Clone for DccHandler<S> {
    fn clone(&self) -> Self {
        Self {
            stream: self.see_if_clonable(),
            active_connections: self.active_connections.clone(),
            downloads: self.downloads.clone(),
            uploads: self.uploads.clone(),
            sender: self.sender.clone(),
        }
    }
}

impl<S: Stream> Write for DccHandler<S> {
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

impl<S: Stream> Read for DccHandler<S> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.stream {
            Ok(s) => s.read(buf),
            Err(_) => Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
        }
    }
}
