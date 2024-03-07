// use irc_project::error::error_client::ErrorClient;
// use irc_project::error::error_view::ErrorView;
// use irc_project::reply::{reply_maker, Reply};
use crate::string_object::StringObject;
// use irc_project::view::model::Model;

use gtk4 as gtk;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};

use gtk4::gio::ListStore;
use gtk4::glib;

#[derive(Clone)]
pub struct ChannelsWindow {
    list_store_channels: ListStore,
    // list_store_contacts: ListStore,
    menu: gtk::gio::Menu,
    // invite_menu: gtk::gio::Menu,
    contact: String,
    channel: String,
    pub menu_model: gtk::PopoverMenu,
    app: Option<Application>,
    pub window: Option<ApplicationWindow>,
}

impl ChannelsWindow {
    pub fn new() -> Self {
        let _list_store_channels = ListStore::new(StringObject::static_type());
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
        menu_bar.append(Some("Topic"), Some("app.change_topic"));
        menu_bar.append(Some("Ajustes"), Some("app.config"));

        let popover = gtk::PopoverMenu::builder().build();
        popover.connect_move_focus(move |a, _b| {
            println!("Move focus");
            a.hide();
        });
        ChannelsWindow {
            list_store_channels: _list_store_channels,
            menu: menu_bar,
            contact: String::new(),
            channel: String::new(),
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

    pub fn add_channel(&mut self, item: &impl IsA<glib::Object>) {
        self.list_store_channels.append(item)
    }

    pub fn set_channel(&mut self, channel: String) {
        self.channel = channel;
    }

    pub fn set_contact(&mut self, contact: String) {
        self.contact = contact;
    }

    pub fn get_channel(&self) -> &String {
        &self.channel
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

    //     pub fn add_action(&self, name: String) {
    //     let action_invite = gtk::gio::SimpleAction::new_stateful(&name,None,  &false.to_variant());
    //     let window = self.window.unwrap().lock().as_deref().unwrap().to_owned();
    //     action_invite.connect_activate(clone!(@weak window => move |_, _| {
    //     let contacts_window_copy = contacts_window_arc_copy.lock().unwrap().clone();
    //     let model = model_copy.lock().expect("cant lock model view");

    //         match model.selected() {
    //             None => {},//label_info.set_label("this is not selected contact/channel"),
    //             Some(selected) => {
    //                 println!("SELECTED: : : : : : {}", selected);
    //                 println!("mensaje: INVITE {} {}", contacts_window_copy.get_contact(), contacts_window_copy.get_channel());
    //                 let msj_parser = format!("INVITE {} {}", contacts_window_copy.get_contact(), contacts_window_copy.get_channel());
    //                 // println!("View -> Controller - Sending: {}", msj_parser);
    //                 tx_update_copy
    //                     .send("true".to_string())
    //                     .expect("cant send to update channel");
    //                 tx_copy.send(msj_parser)
    //                     .expect("cant send to View-> controller from channel");
    //             }
    //         }
    //         contacts_window_copy.unparent_popup();
    //         println!("Invite ACTION");
    //     }));
    //     }
}

impl Default for ChannelsWindow {
    fn default() -> Self {
        Self::new()
    }
}
