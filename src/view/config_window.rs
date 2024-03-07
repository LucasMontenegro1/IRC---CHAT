// use irc_project::error::error_view::ErrorView;
// use irc_project::reply::{reply_maker, Reply};

// use irc_project::view::model::Model;

use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::sync::Mutex;

use gtk4 as gtk;

use gtk::prelude::*;
use gtk::{Entry, Label};

use super::channels_window::ChannelsWindow;

#[derive(Clone)]
pub struct ConfigWindow {
    config_box: gtk::Box,
    dialog: gtk::Dialog,
    channels_window_arc: Rc<Mutex<ChannelsWindow>>,
    topic_dialog: gtk::Dialog,
}

impl ConfigWindow {
    pub fn new(
        channels_window_arc: Rc<Mutex<ChannelsWindow>>,
        tx: Sender<String>,
        tx_update: Sender<String>,
    ) -> Self {
        let ban_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .margin_bottom(5)
            .margin_start(50)
            .margin_end(50)
            .margin_top(5)
            .build();

        let config_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .margin_bottom(10)
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .build();

        let dialog = gtk::Dialog::builder()
            .child(&config_box)
            .default_height(100)
            .default_width(100)
            .build();

        let private_box = Self::create_box(
            "Es privado",
            String::from("p"),
            channels_window_arc.clone(),
            tx.clone(),
            tx_update.clone(),
        );
        let secret_box = Self::create_box(
            "Es secreto",
            String::from("s"),
            channels_window_arc.clone(),
            tx.clone(),
            tx_update.clone(),
        );
        let invite_only_box = Self::create_box(
            "Es invite only",
            String::from("i"),
            channels_window_arc.clone(),
            tx.clone(),
            tx_update.clone(),
        );
        let topic_change_box = Self::create_box(
            "Solo el operador puede cambiar el tópico",
            String::from("t"),
            channels_window_arc.clone(),
            tx.clone(),
            tx_update.clone(),
        );
        let moderated_box = Self::create_box(
            "Es moderado",
            String::from("m"),
            channels_window_arc.clone(),
            tx.clone(),
            tx_update.clone(),
        );
        let no_outside_messages_box = Self::create_box(
            "No hay mensajes desde fuera",
            String::from("n"),
            channels_window_arc.clone(),
            tx.clone(),
            tx_update.clone(),
        );

        let limit_box = Self::create_entry_box(
            String::from("Límite de usuarios"),
            channels_window_arc.clone(),
            tx.clone(),
            tx_update.clone(),
        );
        let topic_box = Self::create_entry_box(
            String::from("Topic"),
            channels_window_arc.clone(),
            tx.clone(),
            tx_update.clone(),
        );
        let ban_entry_box = Self::create_entry_box(
            String::from("Ban"),
            channels_window_arc.clone(),
            tx.clone(),
            tx_update.clone(),
        );
        let ban_button = gtk::Button::builder().label("Ban").build();
        let unban_button = gtk::Button::builder().label("Unban").build();

        let save_button = gtk::Button::builder().label("Save").build();
        let cancel_button = gtk::Button::builder().label("Cancel").build();
        let buttons_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();

        let private_box_copy = private_box.clone();
        let secret_box_copy = secret_box.clone();
        let invite_only_box_copy = invite_only_box.clone();
        let topic_change_box_copy = topic_change_box.clone();
        let moderated_box_copy = moderated_box.clone();
        let no_outside_messages_box_copy = no_outside_messages_box.clone();
        let dialog_copy = dialog.clone();
        let limit_copy = limit_box.clone();
        let topic_copy = topic_box;
        let ban_entry_copy1 = ban_entry_box.clone();
        let ban_entry_copy2 = ban_entry_box.clone();
        let channels_window_arc_copy = channels_window_arc.clone();
        let channels_window_arc_copy1 = channels_window_arc.clone();
        let channels_window_arc_copy2 = channels_window_arc.clone();
        let tx_copy = tx.clone();
        let tx_update_copy = tx_update.clone();
        let tx_copy1 = tx.clone();
        let tx_update_copy1 = tx_update.clone();
        let tx_copy2 = tx.clone();
        let tx_update_copy2 = tx_update.clone();
        ban_button.connect_clicked(move |_| {
            let ban_entry = ban_entry_copy1
                .last_child()
                .unwrap()
                .dynamic_cast::<gtk::Entry>()
                .unwrap();

            let ban = ban_entry.text().to_string();
            if !ban.is_empty() {
                let channels_window = channels_window_arc_copy1.lock().unwrap();
                let msj_parser = format!("MODE {} +b {}", channels_window.get_channel(), ban);
                tx_update_copy1
                    .send("true".to_string())
                    .expect("cant send to update channel");
                tx_copy1
                    .send(msj_parser)
                    .expect("cant send to View-> controller from channel");
            }
        });
        unban_button.connect_clicked(move |_| {
            let ban_entry = ban_entry_copy2
                .last_child()
                .unwrap()
                .dynamic_cast::<gtk::Entry>()
                .unwrap();

            let ban = ban_entry.text().to_string();
            if !ban.is_empty() {
                let channels_window = channels_window_arc_copy2.lock().unwrap();
                let msj_parser = format!("MODE {} -b {}", channels_window.get_channel(), ban);
                tx_update_copy2
                    .send("true".to_string())
                    .expect("cant send to update channel");
                tx_copy2
                    .send(msj_parser)
                    .expect("cant send to View-> controller from channel");
            }
        });
        save_button.connect_clicked(move |_| {
            let private_checkbox = private_box_copy
                .last_child()
                .unwrap()
                .dynamic_cast::<gtk::Switch>()
                .unwrap();
            let secret_checkbox = secret_box_copy
                .last_child()
                .unwrap()
                .dynamic_cast::<gtk::Switch>()
                .unwrap();
            let invite_only_checkbox = invite_only_box_copy
                .last_child()
                .unwrap()
                .dynamic_cast::<gtk::Switch>()
                .unwrap();
            let topic_change_checkbox = topic_change_box_copy
                .last_child()
                .unwrap()
                .dynamic_cast::<gtk::Switch>()
                .unwrap();
            let moderated_checkbox = moderated_box_copy
                .last_child()
                .unwrap()
                .dynamic_cast::<gtk::Switch>()
                .unwrap();
            let no_outside_msg_checkbox = no_outside_messages_box_copy
                .last_child()
                .unwrap()
                .dynamic_cast::<gtk::Switch>()
                .unwrap();
            let limit_entry = limit_copy
                .last_child()
                .unwrap()
                .dynamic_cast::<gtk::Entry>()
                .unwrap();
            let _topic_entry = topic_copy
                .last_child()
                .unwrap()
                .dynamic_cast::<gtk::Entry>()
                .unwrap();
            // let checkbox = gtk_box.last_child().unwrap();
            let checkboxes = vec![
                private_checkbox,
                secret_checkbox,
                invite_only_checkbox,
                topic_change_checkbox,
                moderated_checkbox,
                no_outside_msg_checkbox,
            ];

            for ch in checkboxes {
                ch.activate();
            }

            let number = limit_entry.text().to_string();
            if !number.is_empty() {
                let channels_window = channels_window_arc_copy.lock().unwrap();
                let msj_parser = format!("MODE {} +l {}", channels_window.get_channel(), number);
                println!("View -> Controller - Sending: {}", msj_parser);
                tx_update_copy
                    .send("true".to_string())
                    .expect("cant send to update channel");
                tx_copy
                    .send(msj_parser)
                    .expect("cant send to View-> controller from channel");
                limit_entry.set_text("");
            }

            // let topic = topic_entry.text().to_string();
            // if !topic.is_empty() {
            //     let channels_window = channels_window_arc_copy.lock().unwrap();
            //     let msj_parser = format!("TOPIC {} :{}", channels_window.get_channel(), topic);
            //     tx_update_copy
            //         .send("true".to_string())
            //         .expect("cant send to update channel");
            //     tx_copy
            //         .send(msj_parser)
            //         .expect("cant send to View-> controller from channel");
            //     limit_entry.set_text("");
            // }

            dialog_copy.hide();
        });

        let dialog_copy = dialog.clone();

        cancel_button.connect_clicked(move |_| {
            dialog_copy.hide();
        });
        buttons_box.append(&save_button);
        buttons_box.append(&cancel_button);

        ban_box.append(&ban_entry_box);
        ban_box.append(&ban_button);
        ban_box.append(&unban_button);
        // config_box.append(&topic_box);
        config_box.append(&private_box);
        config_box.append(&secret_box);
        config_box.append(&invite_only_box);
        config_box.append(&topic_change_box);
        config_box.append(&no_outside_messages_box);
        config_box.append(&moderated_box);
        config_box.append(&ban_box);

        config_box.append(&limit_box);
        config_box.append(&buttons_box);

        let topic_dialog = Self::create_topic_window(channels_window_arc.clone(), tx_update, tx);
        ConfigWindow {
            config_box,
            dialog,
            channels_window_arc,
            topic_dialog,
        }
    }

    fn create_entry_box(
        label: String,
        _channels_window_arc: Rc<Mutex<ChannelsWindow>>,
        _tx: Sender<String>,
        _tx_update: Sender<String>,
    ) -> gtk::Box {
        let limit_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();
        let limit_lbl = Label::new(Some(&label));
        let limit_entry = Entry::new();
        limit_box.append(&limit_lbl);
        limit_box.append(&limit_entry);
        limit_box
    }

    pub fn show_topic_window(&self) {
        self.topic_dialog.show();
    }
    pub fn create_topic_window(
        channels_window_arc: Rc<Mutex<ChannelsWindow>>,
        tx_update: Sender<String>,
        tx: Sender<String>,
    ) -> gtk::Dialog {
        let config_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .margin_bottom(10)
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .build();

        let dialog = gtk::Dialog::builder()
            .child(&config_box)
            .default_height(100)
            .default_width(200)
            .build();

        let topic_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();
        let label = Label::new(Some("Topic"));
        let topic_entry = Entry::new();

        let buttons_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();
        let save_button = gtk::Button::builder().label("Save").build();
        let cancel_button = gtk::Button::builder().label("Cancel").build();
        buttons_box.append(&save_button);
        buttons_box.append(&cancel_button);
        topic_box.append(&label);
        topic_box.append(&topic_entry);
        config_box.append(&topic_box);
        config_box.append(&buttons_box);
        let entry_copy = topic_entry;
        let channels_window_arc_copy = channels_window_arc;
        let dialog_copy = dialog.clone();
        save_button.connect_clicked(move |_| {
            let channels_window = channels_window_arc_copy.lock().unwrap();
            let topic = entry_copy.text().to_string();
            let msj_parser = format!("TOPIC {} :{}", channels_window.get_channel(), topic);
            tx_update
                .send("true".to_string())
                .expect("cant send to update channel");
            tx.send(msj_parser)
                .expect("cant send to View-> controller from channel");

            dialog_copy.hide();
            // .set_text("");
        });

        let dialog_copy = dialog.clone();
        cancel_button.connect_clicked(move |_| {
            dialog_copy.hide();
        });

        // dialog.show()
        dialog
        // let save_button
    }

    fn create_box(
        _label: &str,
        action_letter: String,
        channels_window_arc: Rc<Mutex<ChannelsWindow>>,
        tx: Sender<String>,
        tx_update: Sender<String>,
    ) -> gtk::Box {
        let label = Label::new(Some(_label));
        let checkbox = gtk::Switch::builder().name(&action_letter).build();
        let gtk_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();
        checkbox.connect_activate(move |val| {
            let val = val.state();
            let channels_window = channels_window_arc.lock().unwrap();
            let plus_minus = match val {
                true => "+",
                false => "-",
            };

            let change = format!("{}{}", plus_minus, action_letter);

            let msj_parser = format!("MODE {} {}", channels_window.get_channel(), change);
            println!("View -> Controller - Sending: {}", msj_parser);
            tx_update
                .send("true".to_string())
                .expect("cant send to update channel");
            tx.send(msj_parser)
                .expect("cant send to View-> controller from channel");
        });

        gtk_box.append(&label);
        gtk_box.append(&checkbox);
        gtk_box
    }

    pub fn get_box(&self) -> gtk::Box {
        self.config_box.clone()
    }

    pub fn set_checkboxes_state(&self, reply: String) {
        // println!("Reply dentro de set checkboxes: {}", reply);
        if !reply.contains("not channel operator") {
            let channels_window = self.channels_window_arc.lock().unwrap();
            let _channel = channels_window.get_channel();
            let vec = reply.split(',').collect::<Vec<&str>>();

            if vec.len() >= 2 {
                let modes = match vec.get(1) {
                    Some(s) => String::from(*s),
                    None => String::from(""),
                };

                let mut gtk_box = self.config_box.first_child();

                while gtk_box.is_some() {
                    let checkbox = gtk_box.clone().unwrap().last_child().unwrap();
                    let checkbox = match checkbox.dynamic_cast::<gtk::Switch>() {
                        Ok(c) => c,
                        Err(_) => {
                            gtk_box = gtk_box.unwrap().next_sibling();
                            continue;
                        }
                    };
                    checkbox.set_state(false);
                    // println!("Modes: {}, widget name: {}", modes, &checkbox.widget_name().to_string());
                    if modes.contains(&checkbox.widget_name().to_string()) {
                        checkbox.set_state(true);
                    }

                    gtk_box = gtk_box.unwrap().next_sibling();
                }
            }

            let vec = reply.split(':').collect::<Vec<&str>>();

            if vec.get(1).is_some() {
                let topic = vec.get(1).unwrap();
                let gtk_box = self
                    .topic_dialog
                    .child()
                    .unwrap()
                    .first_child()
                    .unwrap()
                    .last_child()
                    .unwrap();
                // let mut gtk_box = self.config_box.first_child();
                // let entry = gtk_box.clone().unwrap().last_child().unwrap();
                let entry = gtk_box.dynamic_cast::<gtk::Entry>().unwrap();

                entry.set_text(topic);
            }
        } else {
            self.dialog.hide();
            // let mut gtk_box = self.config_box.first_child();

            // while gtk_box.is_some() {
            //     let checkbox = gtk_box.clone().unwrap().last_child().unwrap();
            //     let switch = match checkbox.dynamic_cast::<gtk::Switch>() {
            //         Ok(c) => c,
            //         Err(_) => {
            //             gtk_box = gtk_box.unwrap().next_sibling();
            //             continue
            //         },
            //     };
            //     // switch.hide();
            //     switch.set_sensitive(false);
            //     // checkbox.set_state(false);
            //     // println!("Modes: {}, widget name: {}", modes, &checkbox.widget_name().to_string());
            //     // if modes.contains(&checkbox.widget_name().to_string()) {
            //         // checkbox.set_state(true);
            //     // }

            //     gtk_box = gtk_box.unwrap().next_sibling();
            // }
        }
    }
    pub fn unpop(&self) {
        self.dialog.hide();
    }
    pub fn show(&self) {
        self.dialog.set_modal(true);
        self.dialog.show();
    }
}
