use std::io::Write;

use crate::{
    dcc::{dcc_connection::Stream, dcc_handler::DccHandler},
    error::{error_command::ErrorCommand, error_msg::ErrorMsg, error_server::ErrorServer},
    parser::dcc_message::DccMessage,
};

/// Represents a DCC chat command.
pub struct DccChat {
    /// The sender of the DCC chat command, if available.
    from: Option<String>,
    /// The target user for the DCC chat command.
    user: String,
    /// The IP address associated with the DCC chat command.
    ip: String,
    /// The port number associated with the DCC chat command.
    port: String,
}

impl DccChat {
    /// Creates a new `DccChat` instance based on the provided DccMessage.
    ///
    /// # Arguments
    ///
    /// * `msg` - A DccMessage containing information for the DccChat creation.
    ///
    /// # Returns
    ///
    /// A Result containing the new DccChat instance or an ErrorMsg if the creation fails.
    pub fn new(msg: DccMessage) -> Result<Self, ErrorMsg> {
        let ip = msg.get_param_from_msg(1);
        let port = msg.get_param_from_msg(2);
        let user = msg.target_user();
        let from = msg.prefix();

        if let (Some(ip), Some(port), Some(user)) = (ip, port, user) {
            Ok(Self {
                ip,
                port,
                user,
                from,
            })
        } else {
            Err(ErrorMsg::InvalidMsg(ErrorCommand::MissingParametersDcc(
                super::DccCommand::Chat,
            )))
        }
    }

    /// Sends a response to the DCC chat command.
    ///
    /// # Arguments
    ///
    /// * `handler` - A mutable reference to the DccHandler responsible for managing connections.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an ErrorServer if the response fails.
    pub fn response<T>(&self, handler: &mut DccHandler<T>) -> Result<(), ErrorServer>
    where
        T: Stream,
    {
        handler.new_connection(
            self.user.clone(),
            format!("{}:{}", self.ip, self.port).to_string(),
        )?;
        handler.write_all(
            format!(
                "privmsg {} dcc chat chat {} {}",
                self.user, self.ip, self.port
            )
            .as_bytes(),
        )?;
        Ok(())
    }

    /// Handles the reception of a DCC chat command.
    ///
    /// # Arguments
    ///
    /// * `handler` - A reference to the DccHandler responsible for managing connections.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an ErrorServer if the reception fails.
    pub fn reception<T>(&self, handler: &DccHandler<T>) -> Result<(), ErrorServer>
    where
        T: Stream,
    {
        let from = self.from.clone().map_or_else(|| " ".to_string(), |s| s);
        handler.connect_to(from, format!("{}:{}", self.ip, self.port).to_string())?;
        Ok(())
    }

    /// Gets the port associated with the DCC chat command.
    pub fn port(&self) -> String {
        self.port.clone()
    }

    /// Gets the IP address associated with the DCC chat command.
    pub fn ip(&self) -> String {
        self.ip.clone()
    }
}
