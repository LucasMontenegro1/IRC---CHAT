use crate::user::User;
use std::io::Write;
use std::net::TcpStream;

///
/// struct that implements a connection
/// a connection its an entity that saves
/// the nickname of the user associated with
/// a tcp stream and makes it clonable so it
/// can be used in the repositories.
///
pub struct Connection {
    stream: Option<TcpStream>,
    user: User,
    away_msg: Option<String>,
}

impl Connection {
    ///
    /// function that creates a new connection
    ///
    pub fn new(stream: TcpStream, user: User) -> Self {
        Self {
            stream: Some(stream),
            user,
            away_msg: None,
        }
    }

    ///
    /// creates a connection for a user not in the actual server
    ///
    pub fn connection_away_server(user: User) -> Self {
        Self {
            stream: None,
            user,
            away_msg: None,
        }
    }
    ///
    /// function that tries to clone
    /// the tcp stream into the connection
    ///
    pub fn see_if_clonable(&self) -> Option<TcpStream> {
        if let Some(c) = &self.stream {
            return match c.try_clone() {
                Ok(c) => Some(c),
                Err(_) => None,
            };
        }
        None
    }

    pub fn get_hostname(&self) -> Option<&str> {
        self.user.hostname()
    }

    pub fn get_username(&self) -> Option<&str> {
        self.user.username()
    }

    pub fn get_servername(&self) -> Option<&str> {
        self.user.servername()
    }

    pub fn get_realname(&self) -> Option<&str> {
        self.user.realname()
    }

    pub fn modify_connecion_flag(&mut self, flag: &str) {
        self.user.modify_user_flag(flag)
    }

    pub fn return_connection_flags_str(&self) -> String {
        self.user.return_user_flags_str()
    }

    pub fn get_nickname(&self) -> String {
        match self.user.nickname() {
            Some(str) => str.to_owned(),
            None => String::new(),
        }
    }

    pub fn set_nickname(&mut self, new_nick: &str) {
        self.user.set_nickname(new_nick);
    }

    pub fn get_user(&self) -> User {
        self.user.clone()
    }

    pub fn get_op_privileges(&mut self) {
        self.user.modify_user_flag("+o");
    }

    pub fn is_op_connection(&self) -> bool {
        self.user.is_op()
    }

    pub fn is_invisible_connection(&self) -> bool {
        self.user.is_invisible()
    }

    pub fn get_away_msg(&self) -> Option<String> {
        self.away_msg.clone()
    }

    pub fn set_away_msg(&mut self, msg: Option<String>) {
        self.away_msg = msg
    }

    pub fn is_on_server(&self, servername: &str) -> bool {
        self.get_servername().eq(&Some(servername))
    }
}

impl Clone for Connection {
    ///
    /// clones a connection
    ///
    fn clone(&self) -> Self {
        Self {
            stream: self.see_if_clonable(),
            user: self.user.clone(),
            away_msg: self.away_msg.clone(),
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
impl Write for Connection {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.stream {
            Some(s) => s.write(buf),
            None => Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.stream {
            Some(s) => s.flush(),
            None => Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
        }
    }
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match &mut self.stream {
            Some(s) => s.write_all(buf),
            None => Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
        }
    }
}

#[cfg(test)]
mod test {
    // use super::Connection;
    // use crate::user::User;
    // use std::net::{TcpListener, TcpStream};
    // use std::thread;

    #[test]
    fn new_connection() {
        // let thread = thread::spawn(move || {
        //     let listener = TcpListener::bind("localhost:8080").expect("");
        //     loop {
        //         let x = listener.accept().expect("");
        //         let u = User::new(" ", " ", "", " ", " ", " ");
        //         let connection = Connection::new(x.0, u);
        //         assert_eq!(connection.get_nickname(), " ".to_string());
        //         return;
        //     }
        // });
        // TcpStream::connect("localhost:8080").expect("fallo conecction test");
        //
        // thread.join().expect("fallo esperar la conexion");
    }
}
