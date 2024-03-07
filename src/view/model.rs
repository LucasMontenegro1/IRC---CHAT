use std::collections::HashMap;

pub struct Model {
    name: String,
    nick: String,
    host: String,
    server: String,
    realname: String,
    pass: Option<String>,
    reply: String,
    contacts: HashMap<String, Vec<String>>,
    dcc_connections: HashMap<String, Vec<String>>, //La primera string del vector es la ip:port
    channels: HashMap<String, (Vec<String>, Vec<String>)>,
    selected: Option<String>,
    receive_download: bool,
    ip_host_receive_download: String,
    ip_port_receive_download: String,
    file_name_receive_download: String,
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}

impl Model {
    pub fn new() -> Self {
        Model {
            name: "".to_string(),
            nick: "".to_string(),
            host: "".to_string(),
            server: "".to_string(),
            realname: "".to_string(),
            pass: None,
            reply: "".to_string(),
            contacts: HashMap::new(),
            dcc_connections: HashMap::new(),
            channels: HashMap::new(),
            selected: None,
            receive_download: false,
            ip_host_receive_download: "".to_string(),
            ip_port_receive_download: "".to_string(),
            file_name_receive_download: "".to_string(),
        }
    }

    pub fn reply(&self) -> String {
        self.reply.clone()
    }
    pub fn set_reply(&mut self, reply: String) {
        self.reply = reply;
    }

    pub fn add_dcc_connection(&mut self, name: String, ip_port: String) {
        match self.dcc_connections.get_mut(&name) {
            None => {
                self.dcc_connections.insert(name.clone(), vec![ip_port]);
            }
            Some(dcc) => {
                dcc[0] = ip_port;
            }
        }
    }

    pub fn get_dcc_connection(&mut self, name: String) -> Option<String> {
        match self.dcc_connections.get_mut(&name) {
            None => None,
            Some(dcc) => {
                for element in dcc.iter() {
                    println!("vector:{}", element);
                }
                return dcc.first().cloned();
            }
        }
    }

    pub fn add_dcc_message(
        &mut self,
        name: String,
        sended: bool,
        ip_port: Option<String>,
        msg: String,
    ) {
        //Sended is true if i sended it, false if i received it
        if sended {
            match self.dcc_connections.get_mut(&name) {
                None => { //Throw error?
                }
                Some(dcc) => {
                    dcc.push(msg);
                }
            }
        } else {
            let send_msg: String = name.clone() + ":" + &msg;
            match self.dcc_connections.get_mut(&name) {
                None => {
                    self.dcc_connections
                        .insert(name.clone(), vec![ip_port.unwrap(), send_msg]);
                    // Change unwrap for something safer
                }
                Some(dcc) => {
                    dcc.push(send_msg);
                }
            }
        }
    }

    pub fn add_msg(&mut self, name: &String, msg: String) {
        match self.contacts.get_mut(name) {
            None => {
                self.contacts.insert(name.clone(), vec![msg]);
            }
            Some(user_msgs) => {
                user_msgs.push(msg);
            }
        }
    }

    pub fn add_msg_to_channel(&mut self, name: &String, msg: String) {
        match self.channels.get_mut(name) {
            None => {
                self.channels.insert(name.clone(), (vec![msg], vec![]));
            }
            Some((channel_msgs, _)) => {
                channel_msgs.push(msg);
            }
        }
    }

    pub fn remove_msg_from_channel(&mut self, name: &String) {
        match self.channels.get_mut(name) {
            None => {}
            Some((channel_msgs, _)) => {
                channel_msgs.pop();
            }
        }
    }

    pub fn get_msgs_contact(&self, name: &String) -> Option<Vec<String>> {
        self.contacts.get(name).cloned()
    }

    pub fn get_users_channel(&self, name: &String) -> Option<Vec<String>> {
        self.channels.get(name).map(|(_, users)| users.clone())
    }

    pub fn get_msgs_channel(&self, name: &String) -> Option<Vec<String>> {
        self.channels.get(name).map(|(msgs, _)| msgs.clone())
    }

    pub fn add_user_to_channel(&mut self, channel: &String, user: String) {
        match self.channels.get_mut(channel) {
            None => {}
            Some((_, users)) => {
                if !users.contains(&user) {
                    users.push(user)
                }
            }
        }
    }

    pub fn remove_user_from_channel(&mut self, channel: &String, user: String) {
        match self.channels.get_mut(channel) {
            None => {}
            Some((_, users)) => {
                for _user in users.clone().iter().enumerate() {
                    if user == _user.1.clone() {
                        users.remove(_user.0);
                    }
                }
            }
        }
    }

    pub fn remove_users_from_channel(&mut self, channel: &String, users_in_channel: String) {
        match self.channels.get_mut(channel) {
            None => {}
            Some((_, users)) => {
                for _user in users.clone().iter().enumerate() {
                    if !users_in_channel.contains(_user.1) {
                        users.remove(_user.0);
                    }
                }
            }
        }
    }

    pub fn add_contact(&mut self, contact: String) {
        self.contacts.insert(contact.clone(), vec![]);
        self.selected = Some(contact);
    }

    pub fn add_channel(&mut self, channel: String) {
        self.channels.insert(channel.clone(), (vec![], vec![]));
        self.selected = Some(channel);
    }

    pub fn delete_channel(&mut self, channel: &String) {
        self.channels.remove(channel);
    }

    pub fn set_ip_host_receive_download(&mut self, ip_host: String) {
        self.ip_host_receive_download = ip_host.clone();
    }

    pub fn set_ip_port_receive_download(&mut self, ip_port: String) {
        self.ip_port_receive_download = ip_port.clone();
    }

    pub fn set_file_name_receive_download(&mut self, file_name: String) {
        self.file_name_receive_download = file_name.clone();
    }

    pub fn nick(&self) -> String {
        self.nick.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn host(&self) -> String {
        self.host.clone()
    }

    pub fn server(&self) -> String {
        self.server.clone()
    }

    pub fn realname(&self) -> String {
        self.realname.clone()
    }

    pub fn selected(&self) -> Option<String> {
        self.selected.clone()
    }

    pub fn is_receive_downad(&self) -> bool {
        self.receive_download
    }

    pub fn set_selected(&mut self, name: String) {
        self.selected = Some(name);
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_nick(&mut self, nick: String) {
        self.nick = nick;
    }

    pub fn set_host(&mut self, host: String) {
        self.host = host;
    }

    pub fn set_server(&mut self, server: String) {
        self.server = server;
    }

    pub fn set_realname(&mut self, realname: String) {
        self.realname = realname;
    }

    pub fn set_pass(&mut self, pass: String) {
        self.pass = Some(pass);
    }

    pub fn set_receive_downad(&mut self, is_receive_download: bool) {
        self.receive_download = is_receive_download;
    }

    pub fn get_contacts(&self) -> Vec<String> {
        let mut contacts = Vec::<String>::new();
        for contact in self.contacts.keys() {
            contacts.push(contact.clone());
        }
        contacts
    }

    pub fn get_channels(&self) -> Vec<String> {
        let mut channels = Vec::<String>::new();
        for channel in self.channels.keys() {
            channels.push(channel.clone());
        }
        channels
    }

    pub fn get_ip_host_receive_download(&self) -> String {
        self.ip_host_receive_download.clone()
    }

    pub fn get_ip_port_receive_download(&self) -> String {
        self.ip_port_receive_download.clone()
    }

    pub fn get_file_name_receive_download(&self) -> String {
        self.file_name_receive_download.clone()
    }
}

//TODO: agregar Tests
#[cfg(test)]
mod test {

    #[test]
    fn new_user() {}
}
