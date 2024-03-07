use crate::view::channel_user_menu::ChannelUserMenu;
use crate::view::channels_window::ChannelsWindow;
use crate::view::contacts_window::ContactsWindow;
use crate::view::users_config_window::UsersConfigWindow;
use crate::view::users_window::UsersWindow;
use std::{
    rc::Rc,
    sync::{Mutex, RwLock},
};

#[derive(Clone)]
pub struct ArcWidgetChatModel {
    contacts_window_arc: Rc<Mutex<ContactsWindow>>,
    channel_window_arc: Rc<Mutex<ChannelsWindow>>,
    users_window_arc: Rc<Mutex<UsersWindow>>,
    channel_users_window_arc: Rc<RwLock<ChannelUserMenu>>,
    user_config_window: Rc<Mutex<UsersConfigWindow>>,
}

impl ArcWidgetChatModel {
    pub fn new(
        contacts_window_arc: Rc<Mutex<ContactsWindow>>,
        channel_window_arc: Rc<Mutex<ChannelsWindow>>,
        users_window_arc: Rc<Mutex<UsersWindow>>,
        channel_users_window_arc: Rc<RwLock<ChannelUserMenu>>,
        user_config_window: Rc<Mutex<UsersConfigWindow>>,
    ) -> Self {
        ArcWidgetChatModel {
            contacts_window_arc,
            channel_window_arc,
            users_window_arc,
            channel_users_window_arc,
            user_config_window,
        }
    }

    pub fn get_contacts_window_arc(&self) -> Rc<Mutex<ContactsWindow>> {
        self.contacts_window_arc.clone()
    }
    pub fn get_channel_window_arc(&self) -> Rc<Mutex<ChannelsWindow>> {
        self.channel_window_arc.clone()
    }
    pub fn get_users_window_arc(&self) -> Rc<Mutex<UsersWindow>> {
        self.users_window_arc.clone()
    }
    pub fn get_channel_users_window_arc(&self) -> Rc<RwLock<ChannelUserMenu>> {
        self.channel_users_window_arc.clone()
    }
    pub fn get_user_config_window(&self) -> Rc<Mutex<UsersConfigWindow>> {
        self.user_config_window.clone()
    }
}
