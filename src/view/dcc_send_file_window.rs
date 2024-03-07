use std::net::TcpStream;

use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use gtk::prelude::BoxExt;
use gtk::traits::{ButtonExt, WidgetExt};
use gtk4 as gtk;

use gtk4::gio::prelude::*;

use crate::dcc::dcc_handler::DccHandler;

use super::model::Model;

#[derive(Clone)]
pub struct DDCSendFileWindow {
    file_sending_box: gtk::Box,
    dialog: gtk::Dialog,
    ip: Arc<Mutex<String>>,
    port: Arc<Mutex<String>>,
}

impl DDCSendFileWindow {
    pub fn new(
        filename: String,
        ip: String,
        port: String,
        dcc_handler: DccHandler<TcpStream>,
        tx_cont: Sender<String>,
        model_view_arc: Arc<Mutex<Model>>,
        sended_by: String,
    ) -> Self {
        let file_sending_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .margin_bottom(10)
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .build();

        let dialog = gtk::Dialog::builder()
            .child(&file_sending_box)
            .default_height(80)
            .default_width(80)
            .build();

        let filename_label = gtk::Label::builder().label(&filename).build();
        let loading_label = gtk::Label::builder().label("Loading...").build();

        //TODO: cambiar a un set...

        filename_label.set_text("cosita");

        //let send_resume_button = gtk::Button::builder().label("Pause").build();
        let send_resume_button = gtk::Button::builder()
            .icon_name("media-playback-pause")
            .build();
        let abort_button = gtk::Button::builder().icon_name("application-exit").build();
        let button_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();

        /*let users_window_arc_copy = users_window_arc.clone();
        let tx_copy = tx;
        let tx_update_copy = tx_update*/

        let dialog_copy = dialog.clone();

        abort_button.connect_clicked(move |_| {
            dialog_copy.hide();
            //Aca se cierra directamente el envio del archivo, quiza haya que mandar algun comando
        });

        let port_arc = Arc::new(Mutex::new(port.clone()));
        let ip_arc = Arc::new(Mutex::new(ip.clone()));

        let port_arc_clone = port_arc.clone();
        let ip_arc_clone = ip_arc.clone();

        let send_resume_button_clone = send_resume_button.clone();
        let loading_label_clone = loading_label.clone();

        send_resume_button_clone.connect_clicked(move |_| {
            let mut model = model_view_arc.lock().expect("cant lock model view");
            let str = send_resume_button.icon_name();
            match str {
                Some(string) => {
                    let ip = port_arc_clone.lock().expect("Error al bloquear ip");
                    let port = ip_arc_clone.lock().expect("Error al bloquear port");

                    if string == "media-playback-pause" {
                        send_resume_button.set_icon_name("media-playback-start");
                        loading_label_clone.set_label("File sending process paused");

                        let msj_parser = format!(
                            "PRIVMSG {} dcc pause {} {} {} ",
                            sended_by,
                            filename,
                            ip.trim().replace(':', " "),
                            port
                        );
                        let dcc = dcc_handler
                            .clone()
                            .handle_dcc_message_send(msj_parser.clone());

                        match dcc {
                            Ok(_) => {
                                println!("View -> DCC Handler - Sending: {}", msj_parser);

                                model.add_msg(&sended_by, "Paused file send DCC".to_string());
                                tx_cont
                                    .send("true".to_string())
                                    .expect("cant send to update channel");
                            }
                            Err(msg) => {
                                println!("Fallo pausa de envio {}", msg)
                            }
                        }
                    } else {
                        send_resume_button.set_icon_name("media-playback-pause");
                        loading_label_clone.set_label("Descarga en proceso...");

                        let msj_parser = format!(
                            "PRIVMSG {} dcc resume {} {} {} ",
                            sended_by,
                            filename,
                            ip.trim().replace(':', " "),
                            port
                        );
                        let dcc_message_result = dcc_handler
                            .clone()
                            .handle_dcc_message_send(msj_parser.clone());

                        match dcc_message_result {
                            Ok(_) => {
                                println!("View -> DCC Handler - Sending: {} ", msj_parser);

                                model.add_msg(&sended_by, "Resumed file send DCC".to_string());
                                tx_cont
                                    .send("true".to_string())
                                    .expect("cant send to update dcc");
                            }
                            Err(msg) => {
                                println!("Fallo Pausa de send file dcc: {}", msg)
                            }
                        }
                    }
                }
                None => {
                    //nunca deberia entrar
                }
            }
        });

        button_box.append(&send_resume_button_clone);
        button_box.append(&abort_button);

        file_sending_box.append(&filename_label);
        file_sending_box.append(&loading_label);
        file_sending_box.append(&button_box);

        DDCSendFileWindow {
            file_sending_box,
            dialog,
            ip: ip_arc,
            port: port_arc,
        }
    }

    pub fn set_ip(&mut self, ip_received: String) {
        let mut ip = self.ip.lock().expect("Error al bloquear ip");
        *ip = ip_received;
    }

    pub fn set_port(&mut self, p: String) {
        let mut port = self.port.lock().expect("Error al bloquear port");
        *port = p;
    }

    pub fn set_text_name_file_label(&self, name: String) {
        if let Some(first_child) = self.get_box().first_child() {
            if let Ok(label) = first_child.downcast::<gtk::Label>() {
                label.set_text(&name.clone());
            }
        }
    }

    pub fn get_box(&self) -> gtk::Box {
        self.file_sending_box.clone()
    }

    pub fn show(&self) {
        self.dialog.show();
    }
}
