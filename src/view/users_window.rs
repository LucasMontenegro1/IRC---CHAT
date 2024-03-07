// use irc_project::error::error_client::ErrorClient;
// use irc_project::error::error_view::ErrorView;
// use irc_project::reply::{reply_maker, Reply};

// use irc_project::view::model::Model;

use gtk4 as gtk;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};

#[derive(Clone)]
pub struct UsersWindow {
    menu: gtk::gio::Menu,
    // invite_menu: gtk::gio::Menu,
    contact: String,
    nick: String,
    pub menu_model: gtk::PopoverMenu,
    app: Option<Application>,
    window: Option<ApplicationWindow>,
}

impl UsersWindow {
    pub fn new() -> Self {
        // let _list_store_contacts = ListStore::new(StringObject::static_type());
        let menu_bar = gtk::gio::Menu::new();
        // let _invite_menu = gtk::gio::Menu::new();
        // _invite_menu.append(Some("Invite"), Some("app.invite"));
        // menu_bar.append_submenu(Some("Invite"), &_invite_menu);
        // menu_bar.append(Some("Give Op Privileges"), Some("app.whois"));
        // menu_bar.append(Some("Set private"), Some("app.set_private"));
        // menu_bar.append(Some("Set secret"), Some("app.whois"));
        // menu_bar.append(Some("Set invite only"), Some("app.whois"));
        // menu_bar.append(Some("Set topic"), Some("app.whois"));
        // menu_bar.append(Some("lore"), Some("app.whois"));
        menu_bar.append(Some("Status"), Some("app.channel_status"));
        menu_bar.append(Some("Ajustes"), Some("app.config_user"));

        let popover = gtk::PopoverMenu::builder().build();
        UsersWindow {
            menu: menu_bar,
            contact: String::new(),
            nick: String::new(),
            menu_model: popover,
            app: None,
            window: None,
        }
    }
    pub fn add_window(&mut self, window: ApplicationWindow) {
        self.window = Some(window);
    }
    pub fn add_app(&mut self, app: Application) {
        self.app = Some(app);
    }
    pub fn get_menu(&self) -> &gtk::gio::Menu {
        &self.menu
    }

    // pub fn get_invite_menu(&self) -> &gtk::gio::Menu {
    //     &self.invite_menu
    // }

    // pub fn add_whois(&self, action: &str) {
    //     self.invite_menu.append(Some("Who Is?"), Some(action));
    // }
    // pub fn add_contact(&mut self, item: &impl IsA<glib::Object>) {
    //     self.list_store_contacts.append(item)
    // }

    pub fn set_nick(&mut self, nick: String) {
        self.nick = nick;
    }

    pub fn set_contact(&mut self, contact: String) {
        self.contact = contact;
    }

    pub fn get_nick(&self) -> &String {
        &self.nick
    }

    pub fn get_contact(&self) -> &String {
        &self.contact
    }

    pub fn add_model_to_popover(&self, model: &gtk::gio::Menu) {
        self.menu_model.set_menu_model(Some(model));
    }

    pub fn unparent_popup(&self) {
        self.menu_model.unparent();
    }
}

impl Default for UsersWindow {
    fn default() -> Self {
        Self::new()
    }
}
