use gtk4::{Button, Entry};

pub struct LoginView {
    button: Button,
    entry_user: Entry,
    entry_host: Entry,
    entry_server: Entry,
    entry_real: Entry,
    entry_nick: Entry,
    entry_pass: Entry,
}

impl LoginView {
    pub fn new(
        button: Button,
        entry_user: Entry,
        entry_host: Entry,
        entry_server: Entry,
        entry_real: Entry,
        entry_nick: Entry,
        entry_pass: Entry,
    ) -> Self {
        LoginView {
            button,
            entry_user,
            entry_host,
            entry_server,
            entry_real,
            entry_nick,
            entry_pass,
        }
    }

    pub fn get_button(&self) -> Button {
        self.button.clone()
    }
    pub fn get_entry_user(&self) -> Entry {
        self.entry_user.clone()
    }
    pub fn get_entry_host(&self) -> Entry {
        self.entry_host.clone()
    }
    pub fn get_entry_server(&self) -> Entry {
        self.entry_server.clone()
    }
    pub fn get_entry_real(&self) -> Entry {
        self.entry_real.clone()
    }
    pub fn get_entry_nick(&self) -> Entry {
        self.entry_nick.clone()
    }
    pub fn get_entry_pass(&self) -> Entry {
        self.entry_pass.clone()
    }
}
