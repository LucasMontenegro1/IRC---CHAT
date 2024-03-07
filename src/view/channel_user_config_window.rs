// use irc_project::error::error_view::ErrorView;
// use irc_project::reply::{reply_maker, Reply};

// use irc_project::view::model::Model;

use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::sync::RwLock;

use gtk4 as gtk;

use gtk::prelude::*;
use gtk::Label;

use super::channel_user_menu::ChannelUserMenu;

#[derive(Clone)]
pub struct ChannelUserConfigWindow {
    config_box: gtk::Box,
    dialog: gtk::Dialog,
    channel_user_menu_arc: Rc<RwLock<ChannelUserMenu>>,
}

impl ChannelUserConfigWindow {
    pub fn new(
        channel_user_menu_arc: Rc<RwLock<ChannelUserMenu>>,
        tx: Sender<String>,
        tx_update: Sender<String>,
    ) -> Self {
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

        let operator_box = Self::create_box(
            "Hacer operador",
            String::from("o"),
            channel_user_menu_arc.clone(),
            tx.clone(),
            tx_update.clone(),
        );
        let allow_talk_box = Self::create_box(
            "Permitir hablar",
            String::from("v"),
            channel_user_menu_arc.clone(),
            tx.clone(),
            tx_update.clone(),
        );

        let save_button = gtk::Button::builder().label("Save").build();
        let cancel_button = gtk::Button::builder().label("Cancel").build();
        let buttons_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();

        let operator_box_copy = operator_box.clone();
        let allow_talk_box_copy = allow_talk_box.clone();

        let _channel_user_menu_arc_copy = channel_user_menu_arc.clone();
        let _tx_copy = tx;
        let _tx_update_copy = tx_update;
        let dialog_copy = dialog.clone();
        save_button.connect_clicked(move |_| {
            let operator_checkbox = operator_box_copy
                .last_child()
                .unwrap()
                .dynamic_cast::<gtk::Switch>()
                .unwrap();
            let allow_talk_checkbox = allow_talk_box_copy
                .last_child()
                .unwrap()
                .dynamic_cast::<gtk::Switch>()
                .unwrap();

            // let checkbox = gtk_box.last_child().unwrap();
            let checkboxes = vec![operator_checkbox, allow_talk_checkbox];

            for ch in checkboxes {
                println!("ESTADO CHEKBOX {} : {}", ch.widget_name(), ch.state());
                ch.activate();
            }

            dialog_copy.hide();
        });

        let dialog_copy = dialog.clone();

        cancel_button.connect_clicked(move |_| {
            dialog_copy.hide();
        });
        buttons_box.append(&save_button);
        buttons_box.append(&cancel_button);

        config_box.append(&operator_box);
        config_box.append(&allow_talk_box);
        config_box.append(&buttons_box);

        ChannelUserConfigWindow {
            config_box,
            dialog,
            channel_user_menu_arc,
        }
    }

    fn create_box(
        _label: &str,
        action_letter: String,
        channel_user_menu_arc: Rc<RwLock<ChannelUserMenu>>,
        tx: Sender<String>,
        tx_update: Sender<String>,
    ) -> gtk::Box {
        let label = Label::new(Some(_label));
        let checkbox = gtk::Switch::builder().name(&action_letter).build();
        let gtk_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();
        checkbox.connect_state_set(move |a, _b| {
            println!("cambié el estado de checkbox {}", a.widget_name());
            gtk::Inhibit(false)
        });
        checkbox.connect_activate(move |val| {
            let val = val.state();
            let channels_window = channel_user_menu_arc.read().unwrap();
            let plus_minus = match val {
                true => "+",
                false => "-",
            };

            let change = format!("{}{}", plus_minus, action_letter);

            let msj_parser = format!(
                "MODE {} {} {}",
                channels_window.get_channel(),
                change,
                channels_window.get_contact()
            );
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
        println!(
            "ESTO NO DEBERÍA APARECER CREO QUE NUNCA SE LLAMA ASDJASJDASHFAFDSAcheckboxes: {}",
            reply
        );
        let channels_window = self.channel_user_menu_arc.read().unwrap();
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
            let gtk_box = self.config_box.first_child();
            let entry = gtk_box.unwrap().last_child().unwrap();
            let entry = entry.dynamic_cast::<gtk::Entry>().unwrap();

            entry.set_text(topic);
        }
    }

    pub fn show(&self) {
        self.dialog.show();
    }
}
