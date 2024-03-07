pub mod user_handler;

pub mod user_flag;

use std::str::FromStr;

use self::{builder::UserBuilder, user_flag::UserFlag};
use crate::command::Command;
use crate::parser::{convert_into_command_prefix, message::Message};

pub mod builder;

const SPACE_CHAR: u8 = b' ';

///
/// struct that implements a
/// user inside the server
///
///
#[derive(Default, Debug, PartialEq, Eq, Hash, Clone)]
pub struct User {
    nickname: String,
    hostname: String,
    username: String,
    servername: String,
    realname: String,
    password: Option<String>,
    oper: bool,
    user_flags: Vec<UserFlag>,
}

impl User {
    ///
    /// function that is responsible
    /// for creating a new user
    ///
    ///
    pub fn new(
        nickname: &str,
        username: &str,
        hostname: &str,
        servername: &str,
        realname: &str,
        password: &str,
    ) -> Self {
        let mut pwd = None;
        if !password.is_empty() {
            pwd = Some(password.to_string());
        }
        User {
            nickname: String::from(nickname),
            username: String::from(username),
            hostname: String::from(hostname),
            servername: String::from(servername),
            realname: String::from(realname),
            password: pwd,
            oper: false,
            user_flags: Vec::new(),
        }
    }

    pub fn new_empty() -> Self {
        User::new("", "", "", "", "", "")
    }

    pub fn oper_validation(&self, nick: String, password: String) -> bool {
        match &self.password {
            None => password == *"squit",
            Some(p) => nick == self.nickname && &password == p,
        }
    }

    pub fn password(&self) -> Option<String> {
        self.password.clone()
    }

    ///
    /// function that returns the nickname
    /// of the user
    ///
    pub fn nickname(&self) -> Option<&str> {
        if !self.nickname.is_empty() {
            return Some(&self.nickname);
        }
        None
    }

    pub fn set_nickname(&mut self, nickname: &str) {
        self.nickname = nickname.to_string();
    }

    ///
    /// function that returns the username
    /// of the user
    ///
    pub fn username(&self) -> Option<&str> {
        if !self.username.is_empty() {
            return Some(&self.username);
        }
        None
    }

    ///
    /// function that returns the hostname
    /// of the user
    ///
    pub fn hostname(&self) -> Option<&str> {
        if !self.hostname.is_empty() {
            return Some(&self.hostname);
        }
        None
    }

    ///
    /// function that returns the servername
    /// of the user
    ///
    pub fn servername(&self) -> Option<&str> {
        if !self.servername.is_empty() {
            return Some(&self.servername);
        }
        None
    }

    ///
    /// function that returns the realname
    /// of the user
    ///
    pub fn realname(&self) -> Option<&str> {
        if !self.realname.is_empty() {
            return Some(&self.realname);
        }
        None
    }

    pub fn is_op(&self) -> bool {
        self.user_flags.contains(&UserFlag::O)
    }

    pub fn is_invisible(&self) -> bool {
        self.user_flags.contains(&UserFlag::I)
    }

    pub fn receives_wallops(&self) -> bool {
        self.user_flags.contains(&UserFlag::W)
    }

    pub fn receives_server_notices(&self) -> bool {
        self.user_flags.contains(&UserFlag::S)
    }

    pub fn modify_user_flag(&mut self, user_flag: &str) {
        match user_flag {
            "+o" => {
                let f = UserFlag::O;
                if !(self.user_flags.contains(&f)) {
                    println!("new w flag");
                    self.user_flags.push(f);
                }
            }
            "-o" => {
                if let Some(i) = self.user_flags.iter().position(|x| *x == UserFlag::O) {
                    self.user_flags.remove(i);
                }
            }
            "+w" => {
                let f = UserFlag::W;
                if !(self.user_flags.contains(&f)) {
                    println!("new w flag");
                    self.user_flags.push(f);
                }
            }
            "-w" => {
                if let Some(i) = self.user_flags.iter().position(|x| *x == UserFlag::W) {
                    self.user_flags.remove(i);
                }
            }
            "+s" => {
                let f = UserFlag::S;
                if !(self.user_flags.contains(&f)) {
                    self.user_flags.push(f);
                }
            }
            "-s" => {
                if let Some(i) = self.user_flags.iter().position(|x| *x == UserFlag::S) {
                    self.user_flags.remove(i);
                }
            }
            "+i" => {
                let f = UserFlag::I;
                if !(self.user_flags.contains(&f)) {
                    self.user_flags.push(f);
                }
            }
            "-i" => {
                if let Some(i) = self.user_flags.iter().position(|x| *x == UserFlag::I) {
                    self.user_flags.remove(i);
                }
            }
            _ => {}
        }
    }

    pub fn return_user_flags_str(&self) -> String {
        let mut string = String::new();
        string.push_str(" ,");
        for i in &self.user_flags {
            println!("coso:{}", &i.to_string());
            string.push_str(&i.to_string());
            string.push_str(" - ");
        }
        string
    }

    pub fn build_user_msg(&self) -> String {
        if let Some(nick) = self.nickname() {
            if let Some(host) = self.hostname() {
                if let Some(server) = self.servername() {
                    if let Some(realname) = self.realname() {
                        if let Some(username) = self.username() {
                            let prefix = convert_into_command_prefix(nick);
                            let msg = Message::new(
                                Some(prefix),
                                Command::User,
                                Some(vec![
                                    username.to_string(),
                                    host.to_string(),
                                    server.to_string(),
                                    realname.to_string(),
                                ]),
                            );
                            //println!("Builded message {}", msg.to_string());
                            return msg.to_string();
                        }
                    }
                }
            }
        }
        String::from("")
    }
}

///
/// implementation of the FromStr trait for
/// the user struct
///
///
impl FromStr for User {
    type Err = crate::error::error_user::ErrorUser;
    //Agregar robustez
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.trim().split(SPACE_CHAR as char);
        let mut builder = UserBuilder::new();
        if let Some(str) = split.next() {
            let mut str = str.to_string();
            str.remove(0);
            builder.nickname(str.as_str());
            if let Some(str) = split.next() {
                builder.username(str);
                if let Some(str) = split.next() {
                    builder.hostname(str);
                    if let Some(str) = split.next() {
                        builder.servername(str);
                        let mut name: Vec<&str> = vec![];
                        for n in split {
                            name.push(n);
                        }
                        let mut realname = name.join(" ");
                        realname.remove(0);
                        builder.realname(realname.as_str());
                        //¿La contraseña es necesario comunicarlo?
                    }
                }
            }
        }
        builder.build()
    }
}

///
/// implementation of the trait
/// ToString for the user
///
///
impl ToString for User {
    fn to_string(&self) -> String {
        let mut nick = String::from(":");
        nick.push_str(self.nickname.as_str());
        let msg = vec![
            nick,
            self.username.to_owned(),
            self.hostname.to_owned(),
            self.servername.to_owned(),
        ];
        let mut msg = msg.join(" ");
        msg.push_str(" :");
        msg.push_str(self.realname.as_str());
        msg
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn new_user() {
        let expected = User {
            nickname: String::from("Wiz"),
            username: String::from("guest"),
            hostname: String::from("tolmoon"),
            servername: String::from("tolsun"),
            realname: String::from("Ronnie Reagan"),
            password: Some(String::from("secretpasswordhere")),
            oper: false,
            user_flags: Vec::new(),
        };
        let result = User::new(
            "Wiz",
            "guest",
            "tolmoon",
            "tolsun",
            "Ronnie Reagan",
            "secretpasswordhere",
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn user_to_string() {
        let user = User::new(
            "Wiz",
            "guest",
            "tolmoon",
            "tolsun",
            "Ronnie Reagan",
            "secretpasswordhere",
        );

        let expected = ":Wiz guest tolmoon tolsun :Ronnie Reagan";
        assert_eq!(expected, user.to_string());
    }

    #[test]
    fn string_to_user() {
        let user = ":Wiz guest tolmoon tolsun :Ronnie Reagan";

        let expected = User::new("Wiz", "guest", "tolmoon", "tolsun", "Ronnie Reagan", "");

        assert_eq!(expected, User::from_str(user).unwrap());
    }

    #[test]
    fn user_is_invisible() {
        let mut user = User::new(
            "Wiz",
            "guest",
            "tolmoon",
            "tolsun",
            "Ronnie Reagan",
            "secretpasswordhere",
        );

        user.modify_user_flag("+i");

        assert!(user.is_invisible());
    }

    #[test]
    fn user_receives_wallop() {
        let mut user = User::new(
            "Wiz",
            "guest",
            "tolmoon",
            "tolsun",
            "Ronnie Reagan",
            "secretpasswordhere",
        );

        user.modify_user_flag("+w");

        assert!(user.receives_wallops());
    }

    #[test]
    fn user_receives_notices() {
        let mut user = User::new(
            "Wiz",
            "guest",
            "tolmoon",
            "tolsun",
            "Ronnie Reagan",
            "secretpasswordhere",
        );

        user.modify_user_flag("+s");

        assert!(user.receives_server_notices());
    }

    #[test]
    fn user_can_erase_flag() {
        let mut user = User::new(
            "Wiz",
            "guest",
            "tolmoon",
            "tolsun",
            "Ronnie Reagan",
            "secretpasswordhere",
        );

        user.modify_user_flag("+i");

        user.modify_user_flag("-i");

        assert!(!user.is_invisible());
    }
}
