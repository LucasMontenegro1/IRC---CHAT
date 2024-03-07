use std::collections::HashSet;

pub mod channel_flag;

use crate::{command::Command, error::error_channel::ErrorChannel, parser::message::Message};

use self::channel_flag::ChannelFlag;

///
///
/// Struct that implements a channel according to the irc procol.
/// A channel is a named group of one or more clients which will all
/// receive messages addressed to that channel.
///
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Channel {
    pub name: String,
    topic: Option<String>,
    clients: HashSet<String>,
    channel_operators: HashSet<String>,
    channel_flags: Vec<ChannelFlag>,
    user_limit: Option<usize>,
    bans: HashSet<String>,
    key: Option<String>,
    invited: HashSet<String>,
    moderated_users: HashSet<String>,
}

impl Channel {
    ///
    /// creates a new channel
    /// # Arguments
    /// * `name` : string containing the name given to the channel
    /// * `channel_operator` : reference to a string containing the name of the channel operator
    ///
    pub fn new(name: String, channel_operator: String) -> Channel {
        let mut clients: HashSet<String> = HashSet::new();
        let mut channel_operators = HashSet::new();
        channel_operators.insert(channel_operator.clone());
        clients.insert(channel_operator);
        Channel {
            name,
            topic: None,
            clients,
            channel_operators,
            channel_flags: Vec::new(),
            user_limit: None,
            bans: HashSet::new(),
            key: None,
            invited: HashSet::new(),
            moderated_users: HashSet::new(),
        }
    }

    ///
    /// Returns true if channel's name is equal to name
    ///
    pub fn is_called(&self, name: &str) -> bool {
        self.name == name
    }

    pub fn is_priv(&self) -> bool {
        if self.channel_flags.contains(&ChannelFlag::P) {
            return true;
        }
        false
    }

    pub fn is_secret(&self) -> bool {
        if self.channel_flags.contains(&ChannelFlag::S) {
            return true;
        }
        false
    }

    pub fn is_invite_only(&self) -> bool {
        if self.channel_flags.contains(&ChannelFlag::I) {
            return true;
        }
        false
    }

    pub fn is_no_message_from_outside(&self) -> bool {
        if self.channel_flags.contains(&ChannelFlag::N) {
            return true;
        }
        false
    }

    pub fn update_member(
        &mut self,
        old_nickname: &str,
        new_nickname: &str,
    ) -> Result<bool, ErrorChannel> {
        if self.remove_member(old_nickname) {
            // ¿Que pasa si me banean y me ambio de nickname?
            // Actualizar HashMaps de moderamiento y baneo.
            // ¿Que pasa si no me invitan y me cambio de nickname al original?
            // Si no existe el usuario, borrar como miembro y borrar de invitados y chanOp.
            // ¿Que pasa si soy chanOp y me cambio de nickname al original?
            self.add_member(new_nickname, &self.key.clone())?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    ///
    /// function that adds a new member to the channel
    ///
    pub fn add_member(&mut self, client: &str, key: &Option<String>) -> Result<(), ErrorChannel> {
        let users = self.clients.len();

        if !self.key_is_correct(key) {
            return Err(ErrorChannel::BadKey);
        }

        if let Some(s) = self.user_limit {
            if users >= s {
                return Err(ErrorChannel::FullChannel);
            }
        }
        if self.bans.contains(client) {
            //println!("no joineo");
            return Err(ErrorChannel::BannedClient);
        }
        if self.is_invite_only() && !self.invited.contains(client) {
            return Err(ErrorChannel::ClientNotInvited);
        }
        //println!("joineo");
        self.clients.insert(client.to_string());
        Ok(())
    }

    pub fn invite_member(&mut self, client: String) {
        if self.is_invite_only() {
            self.invited.insert(client);
        }
        //println!("invited: {:?}", self.invited);
    }

    pub fn remove_all_channel_flags(&mut self) {
        self.channel_flags = Vec::new();
    }

    ///
    /// function that removes a member of the channel, if exists.
    ///
    pub fn remove_member(&mut self, client: &str) -> bool {
        if self.has_member(client) {
            return self.clients.remove(client);
        }
        false
    }
    ///
    /// function that checks if exists any participants.
    ///
    pub fn is_empty(&self) -> bool {
        self.member_amount() == 0
    }

    ///
    /// returns a list with the channel characteristics
    ///
    pub fn list_channel(&self) -> String {
        let mut response = self.name.clone();
        response.push_str(": ");
        for i in &self.clients {
            //println!("cliente en canal: {}", i);
            response.push_str(i);
            response.push('~');
        }
        response.pop();
        response
    }

    ///
    /// return all members in channel
    ///
    pub fn return_members(&self) -> Vec<String> {
        self.clients.clone().into_iter().collect()
    }

    pub fn get_status_name(&self, user: &str) -> String {
        if self.is_priv() && !self.has_member(user) {
            return "Prv".to_string();
        }
        self.name.clone()
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    ///
    /// returns the name of the channel operator
    ///
    pub fn get_channel_operators(&self) -> HashSet<String> {
        self.channel_operators.clone()
    }
    ///
    /// returns true if the CO matches with user
    ///
    pub fn is_channel_operator(&self, user: &str) -> bool {
        //println!("Es chop {:?}? en {:?}", user, self.channel_operators);
        self.channel_operators.contains(user)
    }
    ///
    /// returns true if the channel has the given client
    /// or false in the other case
    ///
    pub fn has_member(&self, client: &str) -> bool {
        self.clients.contains(client)
    }

    pub fn status(&self) -> &str {
        if self.is_priv() {
            "private"
        } else if self.is_secret() {
            "secret"
        } else {
            "public"
        }
    }

    pub fn member_amount(&self) -> usize {
        self.clients.len()
    }

    /// returns channel_flags on a channel
    pub fn return_channel_flags(&self) -> Vec<ChannelFlag> {
        self.channel_flags.clone()
    }
    pub fn return_channel_flags_str(&self) -> String {
        let mut string = String::new();
        string.push_str(" ,");
        for i in &self.channel_flags {
            println!("{}", i);
            string.push_str(&i.to_string());
            string.push_str(" - ");
        }
        string
    }

    fn is_moderated_user(&self, user: &str) -> bool {
        self.moderated_users.contains(user)
    }

    fn moderated_permission(&self, user: &str) -> bool {
        if self.channel_flags.contains(&ChannelFlag::M) {
            return self.is_moderated_user(user);
        }
        true
    }

    pub fn user_can_speak(&self, user: &str) -> bool {
        if self.is_no_message_from_outside() && !self.has_member(user) {
            return false;
        }
        self.moderated_permission(user)
    }

    pub fn user_can_set_topic(&self, user: &str) -> bool {
        if self.channel_flags.contains(&ChannelFlag::T) {
            return self.is_channel_operator(user);
        }
        true
    }

    pub fn user_can_kick_others(&self, user: &str) -> bool {
        self.is_channel_operator(user)
    }

    pub fn key_is_correct(&self, key: &Option<String>) -> bool {
        match &self.key {
            Some(s) => match key {
                Some(k) => k == s,
                None => false,
            },
            None => true,
        }
    }

    pub fn set_topic(&mut self, topic: String) {
        self.topic = Some(topic);
    }

    pub fn get_topic(&self) -> Option<String> {
        self.topic.clone()
    }

    pub fn modify_channel_flag(
        &mut self,
        channel_flag: &str,
        params: Option<String>,
    ) -> Option<HashSet<String>> {
        match channel_flag {
            "+o" => {
                if let Some(s) = params {
                    if !self.is_channel_operator(&s) {
                        self.channel_operators.insert(s);
                    }
                }
            }
            "-o" => {
                if self.channel_operators.len() > 1 {
                    if let Some(s) = params {
                        if self.is_channel_operator(&s) {
                            self.channel_operators.remove(&s);
                        }
                    }
                }
            }
            "+p" => {
                let f = ChannelFlag::P;
                if !(self.channel_flags.contains(&f)) {
                    self.channel_flags.push(f);
                }
            }
            "-p" => {
                if let Some(i) = self.channel_flags.iter().position(|x| *x == ChannelFlag::P) {
                    self.channel_flags.remove(i);
                }
            }
            "+s" => {
                let f = ChannelFlag::S;
                if !(self.channel_flags.contains(&f)) {
                    self.channel_flags.push(f);
                }
            }
            "-s" => {
                if let Some(i) = self.channel_flags.iter().position(|x| *x == ChannelFlag::S) {
                    self.channel_flags.remove(i);
                }
            }
            "+i" => {
                let f = ChannelFlag::I;
                if !(self.channel_flags.contains(&f)) {
                    self.channel_flags.push(f);
                }
            }
            "-i" => {
                if let Some(i) = self.channel_flags.iter().position(|x| *x == ChannelFlag::I) {
                    self.channel_flags.remove(i);
                    self.invited = HashSet::new();
                }
            }
            "+t" => {
                let f = ChannelFlag::T;
                if !(self.channel_flags.contains(&f)) {
                    self.channel_flags.push(f);
                }
            }
            "-t" => {
                if let Some(i) = self.channel_flags.iter().position(|x| *x == ChannelFlag::T) {
                    self.channel_flags.remove(i);
                }
            }
            "+n" => {
                let f = ChannelFlag::N;
                if !(self.channel_flags.contains(&f)) {
                    self.channel_flags.push(f);
                }
            }
            "-n" => {
                if let Some(i) = self.channel_flags.iter().position(|x| *x == ChannelFlag::N) {
                    self.channel_flags.remove(i);
                }
            }
            "+m" => {
                let f = ChannelFlag::M;
                if !(self.channel_flags.contains(&f)) {
                    self.channel_flags.push(f);
                }
            }
            "-m" => {
                if let Some(i) = self.channel_flags.iter().position(|x| *x == ChannelFlag::M) {
                    self.channel_flags.remove(i);
                }
                self.moderated_users = HashSet::new();
            }
            "+l" => {
                let f = ChannelFlag::L;
                if !(self.channel_flags.contains(&f)) {
                    self.channel_flags.push(f);
                }
                match params {
                    Some(s) => match s.parse() {
                        Ok(a) => self.user_limit = Some(a),
                        Err(_e) => self.user_limit = None,
                    },
                    None => self.user_limit = None,
                }
            }
            "-l" => {
                if let Some(i) = self.channel_flags.iter().position(|x| *x == ChannelFlag::L) {
                    self.channel_flags.remove(i);
                }
                self.user_limit = None;
            }
            "+b" => {
                println!("entro ban");
                let f = ChannelFlag::B;
                if !(self.channel_flags.contains(&f)) {
                    self.channel_flags.push(f);
                }
                match params {
                    Some(s) => {
                        self.bans.insert(s);
                        println!("baneo");
                        println!("bans: {:?}", self.bans);
                    }
                    None => return Some(self.bans.clone()),
                }
            }
            "-b" => {
                if let Some(s) = params {
                    self.bans.remove(&s);
                    if self.bans.is_empty() {
                        if let Some(i) =
                            self.channel_flags.iter().position(|x| *x == ChannelFlag::B)
                        {
                            self.channel_flags.remove(i);
                        }
                    }
                }
            }
            "+v" => {
                if let Some(s) = params {
                    self.moderated_users.insert(s);
                }
            }
            "-v" => {
                if let Some(s) = params {
                    if self.is_moderated_user(&s) {
                        self.moderated_users.remove(&s);
                    }
                }
            }
            "+k" => {
                let f = ChannelFlag::K;
                if !(self.channel_flags.contains(&f)) {
                    self.channel_flags.push(f);
                }
                if let Some(s) = params {
                    self.key = Some(s);
                }
            }
            "-k" => {
                if let Some(i) = self.channel_flags.iter().position(|x| *x == ChannelFlag::K) {
                    self.channel_flags.remove(i);
                }
                self.key = None;
            }
            _ => {}
        }
        None
    }

    pub fn build_channel_msg(&self) -> Vec<String> {
        if self.name.starts_with('#') {
            let mut join_msg: Vec<Message> = self
                .clients
                .iter()
                .map(|u| Self::build_join_msg(&self.name, u))
                .collect();

            let mut modes_msgs = vec![];
            if let Some(chop) = join_msg.first() {
                let chop = chop.prefix().unwrap();
                modes_msgs.append(&mut Self::build_moderates_users_msg(
                    &self.name,
                    self.moderated_users
                        .iter()
                        .map(std::ops::Deref::deref)
                        .collect::<Vec<&str>>(),
                    &chop,
                ));
                modes_msgs.append(&mut Self::build_chops_msg(
                    &self.name,
                    self.channel_operators
                        .iter()
                        .map(std::ops::Deref::deref)
                        .collect::<Vec<&str>>(),
                    &chop,
                ));
                modes_msgs.append(&mut self.build_flags_msg(&chop));
            }

            join_msg.append(&mut modes_msgs);
            let msg: Vec<String> = join_msg.iter().map(|msg| msg.to_string()).collect();
            return msg;
        }
        vec![String::from("")]
    }

    fn build_join_msg(name: &str, nick: &str) -> Message {
        let mut prefix = ":".to_string();
        prefix.push_str(nick);
        Message::new(Some(prefix), Command::Join, Some(vec![name.to_string()]))
    }

    fn build_chops_msg(name: &str, chops: Vec<&str>, initial_chop: &str) -> Vec<Message> {
        let mut prefix = ":".to_string();
        prefix.push_str(initial_chop);

        let mut chops_msgs: Vec<Message> = chops
            .iter()
            .map(|u| {
                Message::new(
                    Some(prefix.clone()),
                    Command::Mode,
                    Some(vec![name.to_string(), "+o".to_string(), u.to_string()]),
                )
            })
            .collect();

        if !chops.contains(&initial_chop) {
            chops_msgs.push(Message::new(
                Some(prefix),
                Command::Mode,
                Some(vec![
                    name.to_string(),
                    "-o".to_string(),
                    initial_chop.to_string(),
                ]),
            ));
        }

        chops_msgs
    }

    fn build_moderates_users_msg(
        name: &str,
        moderated_users: Vec<&str>,
        initial_chop: &str,
    ) -> Vec<Message> {
        let mut prefix = ":".to_string();
        prefix.push_str(initial_chop);

        let msgs: Vec<Message> = moderated_users
            .iter()
            .map(|u| {
                Message::new(
                    Some(prefix.clone()),
                    Command::Mode,
                    Some(vec![name.to_string(), "+v".to_string(), u.to_string()]),
                )
            })
            .collect();

        msgs
    }

    fn build_flags_msg(&self, chop: &str) -> Vec<Message> {
        let mut p = ":".to_string();
        p.push_str(chop);

        let mut msg = vec![];
        for flag in &self.channel_flags {
            let prefix = p.clone();
            let mut new_msgs = match flag {
                ChannelFlag::P => vec![Message::new(
                    Some(prefix.clone()),
                    Command::Mode,
                    Some(vec![self.name.clone(), "+p".to_string()]),
                )],
                ChannelFlag::M => vec![Message::new(
                    Some(prefix.clone()),
                    Command::Mode,
                    Some(vec![self.name.clone(), "+m".to_string()]),
                )],
                ChannelFlag::S => vec![Message::new(
                    Some(prefix.clone()),
                    Command::Mode,
                    Some(vec![self.name.clone(), "+m".to_string()]),
                )],
                ChannelFlag::L => vec![Message::new(
                    Some(prefix.clone()),
                    Command::Mode,
                    Some(vec![
                        self.name.clone(),
                        "+l".to_string(),
                        self.user_limit.unwrap().to_string(),
                    ]),
                )],
                ChannelFlag::I => vec![Message::new(
                    Some(prefix.clone()),
                    Command::Mode,
                    Some(vec![self.name.clone(), "+i".to_string()]),
                )],
                ChannelFlag::B => self.build_banned_users_msgs(prefix),
                ChannelFlag::T => self.build_topic_msg(&prefix),
                ChannelFlag::K => vec![Message::new(
                    Some(prefix.clone()),
                    Command::Mode,
                    Some(vec![
                        self.name.clone(),
                        "+k".to_string(),
                        self.key.as_ref().unwrap().to_owned(),
                    ]),
                )],
                ChannelFlag::N => vec![Message::new(
                    Some(prefix.clone()),
                    Command::Mode,
                    Some(vec![self.name.clone(), "+n".to_string()]),
                )],
                _ => continue,
            };
            msg.append(&mut new_msgs);
        }

        msg
    }

    fn build_topic_msg(&self, prefix: &str) -> Vec<Message> {
        let topic = match &self.topic {
            Some(t) => t.to_owned(),
            None => String::from("")
        };
        vec![Message::new(
            Some(prefix.to_owned()),
            Command::Topic,
            Some(vec![
                self.name.clone(),
                topic.to_string()
            ]))]
    }

    fn build_banned_users_msgs(&self, prefix: String) -> Vec<Message> {
        let mut msgs = vec![];
        for ban in &self.bans {
            msgs.push(Message::new(
                Some(prefix.clone()),
                Command::Mode,
                Some(vec![self.name.clone(), "+b".to_string(), ban.clone()]),
            ));
        }
        msgs
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn new_channel() {
        let mut set = HashSet::new();
        set.insert(String::from("juan"));
        let mut channel_operators = HashSet::new();
        channel_operators.insert(String::from("juan"));
        let expected = Channel {
            name: String::from("canal"),
            topic: None,
            clients: set,
            channel_operators,
            channel_flags: Vec::new(),
            user_limit: None,
            bans: HashSet::new(),
            key: None,
            invited: HashSet::new(),
            moderated_users: HashSet::new(),
        };
        let result = Channel::new("canal".to_string(), "juan".to_string());

        assert_eq!(result, expected);
    }

    #[test]
    fn channel_name_is_correct() {
        let channel = Channel::new("canal".to_string(), "juan".to_string());

        assert!(channel.is_called("canal"));
    }

    #[test]
    fn add_member() {
        let mut set = HashSet::new();
        set.insert(String::from("juan"));
        set.insert(String::from("pedro"));
        let mut channel_operators = HashSet::new();
        channel_operators.insert(String::from("juan"));
        let expected = Channel {
            name: String::from("canal"),
            topic: None,
            clients: set,
            channel_operators,
            channel_flags: Vec::new(),
            user_limit: None,
            bans: HashSet::new(),
            key: None,
            invited: HashSet::new(),
            moderated_users: HashSet::new(),
        };
        let mut result = Channel::new("canal".to_string(), "juan".to_string());
        result.add_member("pedro", &None).unwrap();

        assert_eq!(result, expected);
    }
    #[test]
    fn list_correctly() {
        let mut set = HashSet::new();
        set.insert(String::from("juan"));
        let mut channel_operators = HashSet::new();
        channel_operators.insert(String::from("juan"));
        let channel = Channel {
            name: String::from("canal"),
            topic: None,
            clients: set,
            channel_operators,
            channel_flags: Vec::new(),
            user_limit: None,
            bans: HashSet::new(),
            key: None,
            invited: HashSet::new(),
            moderated_users: HashSet::new(),
        };
        let result = channel.list_channel();
        let expected = "canal: juan".to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn remove_member() {
        let mut set = HashSet::new();
        set.insert(String::from("juan"));
        let mut channel_operators = HashSet::new();
        channel_operators.insert(String::from("juan"));
        let expected = Channel {
            name: String::from("canal"),
            topic: None,
            clients: set,
            channel_operators,
            channel_flags: Vec::new(),
            user_limit: None,
            bans: HashSet::new(),
            key: None,
            invited: HashSet::new(),
            moderated_users: HashSet::new(),
        };
        let mut result = Channel::new("canal".to_string(), "juan".to_string());
        result.add_member("pedro", &None).unwrap();
        result.remove_member("pedro");
        assert_eq!(result, expected);
    }

    #[test]
    fn return_members_correctly() {
        let vec: Vec<String> = vec![String::from("juan"), String::from("pedro")];

        let mut channel = Channel::new("canal".to_string(), "juan".to_string());
        channel.add_member("pedro", &None).unwrap();
        channel.add_member("rodrigo", &None).unwrap();
        channel.remove_member("rodrigo");

        assert!(vec
            .iter()
            .all(|item| channel.return_members().contains(item)));
    }

    #[test]
    fn channel_named_correctly() {
        let name = "canal".to_string();

        let channel = Channel::new(name.clone(), "juan".to_string());

        assert!(channel.is_called(&name));
    }

    #[test]
    fn channel_has_member() {
        let name1 = "pedro";
        let name2 = "juan";
        let name3 = "rodrigo";

        let mut channel = Channel::new("canal".to_string(), name1.to_string());
        channel.add_member(name2, &None).unwrap();
        channel.add_member(name3, &None).unwrap();
        assert!(channel.has_member(name3));
        channel.remove_member(name3);

        assert!(channel.has_member(name1));
        assert!(channel.has_member(name2));
        assert!(!channel.has_member(name3));
    }

    #[test]
    fn channel_is_created_without_channel_flags() {
        let channel = Channel::new("canal".to_string(), "pedro".to_string());

        assert_eq!(channel.return_channel_flags(), Vec::new())
    }

    #[test]
    fn channel_can_add_channel_flags() {
        let mut channel = Channel::new("canal".to_string(), "pedro".to_string());

        channel.modify_channel_flag("+s", None);

        let mut vec: Vec<ChannelFlag> = Vec::new();

        vec.push(ChannelFlag::S);

        assert_eq!(channel.return_channel_flags(), vec)
    }

    #[test]
    fn channel_can_erase_channel_flags() {
        let mut channel = Channel::new("canal".to_string(), "pedro".to_string());

        channel.modify_channel_flag("+s", None);

        channel.modify_channel_flag("+k", None);

        channel.modify_channel_flag("-s", None);

        let mut vec: Vec<ChannelFlag> = Vec::new();

        vec.push(ChannelFlag::K);

        assert_eq!(channel.return_channel_flags(), vec)
    }

    #[test]
    fn cant_join_banned_user() {
        let mut channel = Channel::new("canal".to_string(), "pedro".to_string());

        channel.modify_channel_flag("+b", Some("pepe".to_string()));

        assert_eq!(
            channel.add_member("pepe", &None),
            Err(ErrorChannel::BannedClient)
        );
        assert!(!channel.has_member("pepe"));
    }

    #[test]
    fn can_unbanned_user() {
        let mut channel = Channel::new("canal".to_string(), "pedro".to_string());

        channel.modify_channel_flag("+b", Some("pepe".to_string()));

        channel.modify_channel_flag("-b", Some("pepe".to_string()));

        channel.add_member("pepe", &None).unwrap();

        assert!(channel.has_member("pepe"));
    }
}
