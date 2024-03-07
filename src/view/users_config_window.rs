// use irc_project::error::error_view::ErrorView;
// use irc_project::reply::{reply_maker, Reply};

// use irc_project::view::model::Model;

use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::sync::Mutex;

use gtk4 as gtk;

use gtk::prelude::*;
use gtk::{Entry, Label};

use super::users_window::UsersWindow;

#[derive(Clone)]
pub struct UsersConfigWindow {
    config_box: gtk::Box,
    dialog: gtk::Dialog,
    users_window_arc: Rc<Mutex<UsersWindow>>,
}

impl UsersConfigWindow {
    pub fn new(
        users_window_arc: Rc<Mutex<UsersWindow>>,
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

        let invisible_box = Self::create_box(
            "Es invisible",
            String::from("i"),
            users_window_arc.clone(),
            tx.clone(),
            tx_update.clone(),
        );
        let secret_box = Self::create_box(
            "Recive server notices",
            String::from("s"),
            users_window_arc.clone(),
            tx.clone(),
            tx_update.clone(),
        );
        let wallops_box = Self::create_box(
            "Recive wallops",
            String::from("w"),
            users_window_arc.clone(),
            tx.clone(),
            tx_update.clone(),
        );
        let operator_box = Self::create_box(
            "Soy operator",
            String::from("o"),
            users_window_arc.clone(),
            tx.clone(),
            tx_update.clone(),
        );

        let away_box = Self::create_entry_box(
            String::from("Away"),
            users_window_arc.clone(),
            tx.clone(),
            tx_update.clone(),
        );
        let save_button = gtk::Button::builder().label("Save").build();
        let cancel_button = gtk::Button::builder().label("Cancel").build();
        let buttons_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();

        let invisible_box_copy = invisible_box.clone();
        let secret_box_copy = secret_box.clone();
        let wallops_box_copy = wallops_box.clone();
        let operator_box_copy = operator_box.clone();
        let away_box_copy = away_box.clone();
        let dialog_copy = dialog.clone();

        let users_window_arc_copy = users_window_arc.clone();
        let tx_copy = tx;
        let tx_update_copy = tx_update;
        save_button.connect_clicked(move |_| {
            let invisible_checkbox = invisible_box_copy
                .last_child()
                .unwrap()
                .dynamic_cast::<gtk::Switch>()
                .unwrap();
            let secret_checkbox = secret_box_copy
                .last_child()
                .unwrap()
                .dynamic_cast::<gtk::Switch>()
                .unwrap();
            let invite_only_checkbox = wallops_box_copy
                .last_child()
                .unwrap()
                .dynamic_cast::<gtk::Switch>()
                .unwrap();
            let topic_change_checkbox = operator_box_copy
                .last_child()
                .unwrap()
                .dynamic_cast::<gtk::Switch>()
                .unwrap();

            let away_entry = away_box_copy
                .last_child()
                .unwrap()
                .dynamic_cast::<gtk::Entry>()
                .unwrap();

            // let checkbox = gtk_box.last_child().unwrap();
            let checkboxes = vec![
                invisible_checkbox,
                secret_checkbox,
                invite_only_checkbox,
                topic_change_checkbox,
            ];

            for ch in checkboxes {
                ch.activate();
            }

            let away_msg = away_entry.text().to_string();
            if !away_msg.is_empty() {
                let _channels_window = users_window_arc_copy.lock().unwrap();
                let msj_parser = format!("AWAY :{}", away_msg);
                println!("View -> Controller - Sending: {}", msj_parser);
                tx_update_copy
                    .send("true".to_string())
                    .expect("cant send to update channel");
                tx_copy
                    .send(msj_parser)
                    .expect("cant send to View-> controller from channel");
                // limit_entry.set_text("");
            }

            dialog_copy.hide();
        });

        let dialog_copy = dialog.clone();

        cancel_button.connect_clicked(move |_| {
            dialog_copy.hide();
        });
        buttons_box.append(&save_button);
        buttons_box.append(&cancel_button);

        config_box.append(&invisible_box);
        config_box.append(&wallops_box);
        config_box.append(&secret_box);
        config_box.append(&operator_box);
        config_box.append(&away_box);
        config_box.append(&buttons_box);

        UsersConfigWindow {
            config_box,
            dialog,
            users_window_arc,
        }
    }

    fn create_entry_box(
        label: String,
        _users_window_arc: Rc<Mutex<UsersWindow>>,
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
    fn create_box(
        _label: &str,
        action_letter: String,
        users_window_arc: Rc<Mutex<UsersWindow>>,
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
            let users_window = users_window_arc.lock().unwrap();
            let plus_minus = match val {
                true => "+",
                false => "-",
            };

            let change = format!("{}{}", plus_minus, action_letter);

            let msj_parser = format!("MODE {} {}", users_window.get_nick(), change);
            println!("View -> Controller - Sending: {}", msj_parser);
            tx_update
                .send("true".to_string())
                .expect("cant send to update user");
            tx.send(msj_parser)
                .expect("cant send to View-> controller from usee");
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
        let users_window = self.users_window_arc.lock().unwrap();
        let _nick = users_window.get_nick();
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
                if modes.contains(&checkbox.widget_name().to_string()) {
                    checkbox.set_state(true);
                }

                gtk_box = gtk_box.unwrap().next_sibling();
            }
        }
    }

    pub fn show(&self) {
        self.dialog.show();
    }
}
