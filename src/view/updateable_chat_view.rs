use gtk4::gio::ListStore;
use gtk4::Label;

#[derive(Clone)]
pub struct UpdatableChatView {
    list_store_users: ListStore,
    list_store_msgs: ListStore,
    list_store_contacts: ListStore,
    list_store_channels: ListStore,
    list_store_users_in_channel: ListStore,
    label_name: Label,
    label_reply: Label,
}

impl UpdatableChatView {
    pub fn new(
        list_store_users: ListStore,
        list_store_msgs: ListStore,
        list_store_contacts: ListStore,
        list_store_channels: ListStore,
        list_store_users_in_channel: ListStore,
        label_name: Label,
        label_reply: Label,
    ) -> Self {
        UpdatableChatView {
            list_store_users,
            list_store_msgs,
            list_store_contacts,
            list_store_channels,
            list_store_users_in_channel,
            label_name,
            label_reply,
        }
    }
    pub fn get_list_store_users(&self) -> ListStore {
        self.list_store_users.clone()
    }
    pub fn get_list_store_msgs(&self) -> ListStore {
        self.list_store_msgs.clone()
    }
    pub fn get_list_store_contacts(&self) -> ListStore {
        self.list_store_contacts.clone()
    }
    pub fn get_list_store_channels(&self) -> ListStore {
        self.list_store_channels.clone()
    }
    pub fn get_list_store_users_in_channel(&self) -> ListStore {
        self.list_store_users_in_channel.clone()
    }
    pub fn get_name_label(&self) -> Label {
        self.label_name.clone()
    }
    pub fn get_reply_label(&self) -> Label {
        self.label_reply.clone()
    }
}
