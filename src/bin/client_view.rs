use crate::dcc::dcc_connection::DirectMessage;
use crate::dcc::dcc_handler::DccHandler;
use irc_project::dcc;
use irc_project::parser::dcc_message::DccMessage;
use irc_project::view::dcc_send_file_window::DDCSendFileWindow;

use gtk::glib::VariantTy;
use irc_project::error::error_client::ErrorClient;
use irc_project::error::error_view::ErrorView;
use irc_project::reply::{code::Code, reply_maker, Reply};
use irc_project::string_object::StringObject;

use irc_project::view::channel_user_config_window::ChannelUserConfigWindow;
use irc_project::view::channel_user_menu::ChannelUserMenu;
use irc_project::view::channels_window::ChannelsWindow;
use irc_project::view::config_window::ConfigWindow;
use irc_project::view::contacts_window::ContactsWindow;
use irc_project::view::model::Model;
use irc_project::view::users_config_window::UsersConfigWindow;
use irc_project::view::users_window::UsersWindow;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

use irc_project::error;

use gtk4 as gtk;

use gtk::{prelude::*, FileChooserAction, FileChooserDialog, ResponseType};
use gtk::{
    Align, Application, ApplicationWindow, Button, Entry, Label, ListView, NoSelection,
    Orientation, PolicyType, ScrolledWindow, SignalListItemFactory, SingleSelection, Widget,
};

use gtk4::gio::ListStore;
use gtk4::glib;
use gtk4::glib::{clone, Continue, MainContext, PRIORITY_DEFAULT};
use irc_project::view::arc_widget_chat_model::ArcWidgetChatModel;
use irc_project::view::login_view_window::LoginView;
use irc_project::view::updateable_chat_view::UpdatableChatView;

const WHITESPACE_U8: u8 = b' ';

const TAMANIO_MSJ: usize = 512;
const TAMANIO_CONTACTO: u16 = 40;

pub struct WindowsReferences {
    pub contacts_window_arc: Rc<Mutex<ContactsWindow>>,
    pub channel_window_arc: Rc<Mutex<ChannelsWindow>>,
    pub config_window_arc: Rc<Mutex<ConfigWindow>>,
    pub user_config_window_arc: Rc<Mutex<UsersConfigWindow>>,
    pub dcc_send_window_arc: Rc<Mutex<DDCSendFileWindow>>,
}

impl WindowsReferences {
    pub fn new(
        contacts_window_arc: Rc<Mutex<ContactsWindow>>,
        channel_window_arc: Rc<Mutex<ChannelsWindow>>,
        config_window_arc: Rc<Mutex<ConfigWindow>>,
        user_config_window_arc: Rc<Mutex<UsersConfigWindow>>,
        dcc_send_window_arc: Rc<Mutex<DDCSendFileWindow>>,
    ) -> Self {
        WindowsReferences {
            contacts_window_arc,
            channel_window_arc,
            config_window_arc,
            user_config_window_arc,
            dcc_send_window_arc,
        }
    }
}

fn main() -> Result<(), ErrorClient> {
    use std::io::{stdin, stdout};
    let mut s = String::new();
    print!("Please server IP: ");
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }

    let read_stream = Arc::new(TcpStream::connect(s)?);
    println!(
        "Connection established with address: {}",
        read_stream.local_addr()?
    );

    //let mut dcc_handler;
    let rs = read_stream
        .clone()
        .try_clone()
        .expect("Error in dcc handle");

    let (tx_dcc, rx_dcc): (mpsc::Sender<DirectMessage>, mpsc::Receiver<DirectMessage>) =
        mpsc::channel();

    let dcc_handler = DccHandler::new(rs, tx_dcc.clone());

    //let dcc_handler = DccHandler::new(rs_clone);

    run_client_view(read_stream, dcc_handler, rx_dcc)
}

fn run_client_view(
    arc_tcpstream: Arc<TcpStream>,
    dcc_handler: DccHandler<TcpStream>,
    rx_dcc: Receiver<DirectMessage>,
) -> Result<(), ErrorClient> {
    let model_view = init_model();

    let (tx_view, rx_view) = mpsc::channel::<String>();
    let (tx_cont, rx_cont) = mpsc::channel::<String>();
    let rx_cont_arc = Arc::new(Mutex::new(rx_cont));

    let model_view_arc = Arc::new(Mutex::new(model_view));
    let model_view_copy = model_view_arc.clone();

    let tx_update_share = tx_cont.clone();
    let list_store: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let list_store_clone = Arc::clone(&list_store);

    let dcc_clone = dcc_handler.clone();

    let th1 = thread::spawn(move || {
        init_app_view(
            model_view_copy,
            tx_view.clone(),
            tx_update_share,
            rx_cont_arc,
            dcc_clone,
        );
    });

    let tcp_stream_copy = arc_tcpstream.clone(); // TODO: Se eliminaria, cambio x DCC
    let dcc_handler_clone2 = dcc_handler.clone();

    let th2: thread::JoinHandle<Result<(), ErrorClient>> = thread::spawn(move || {
        handle_msgs_from_tcp(
            dcc_handler_clone2,
            rx_dcc,
            tcp_stream_copy,
            model_view_arc,
            tx_cont,
            list_store_clone,
        )
    });
    let th3 = thread::spawn(move || handle_msgs_to_tcp(dcc_handler, arc_tcpstream, rx_view));

    if th1.join().is_err() {
        return Err(ErrorClient::ChannelError);
    };
    if th2.join().is_err() {
        return Err(ErrorClient::ChannelError);
    };
    if th3.join().is_err() {
        return Err(ErrorClient::ChannelError);
    };

    Ok(())
}

fn handle_msgs_to_tcp(
    mut dcc_handler: DccHandler<TcpStream>,
    tcp_stream: Arc<TcpStream>,
    rx_view: Receiver<String>,
) -> Result<(), ErrorClient> {
    for buff in rx_view {
        let mut is_dcc = false;
        // println!("(109)Controller <- View - Received: {}", received);
        let mut write_steam = tcp_stream.try_clone().expect("cant clone tcp");

        //TODO : LO NUEVO!
        let msg = buff.trim().to_string();
        let dcc = dcc_handler.handle_dcc_message_send(msg.clone());
        match dcc {
            Ok(_) => is_dcc = true,
            Err(e) => match e {
                crate::error::error_msg::ErrorMsg::EmptyMsg => println!("error en el mensaje dcc"),
                crate::error::error_msg::ErrorMsg::InvalidMsg(e) => match e {
                    crate::error::error_command::ErrorCommand::UnknownCommand => is_dcc = false,
                    crate::error::error_command::ErrorCommand::MissingParameters(_) => {
                        is_dcc = false
                    }
                    crate::error::error_command::ErrorCommand::MissingParametersDcc(_) => {
                        is_dcc = true
                    }
                },
                crate::error::error_msg::ErrorMsg::ServerError(e) => {
                    println!("Server error dcc :{}", e)
                }
            },
        }

        if !is_dcc {
            let _bytes_written = write_steam.write(msg.as_bytes());
        }

        thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}

fn handle_msgs_from_tcp(
    dcc_handler: DccHandler<TcpStream>,
    rx_dcc: Receiver<DirectMessage>,
    tcp_stream: Arc<TcpStream>,
    model_view_arc: Arc<Mutex<Model>>,
    tx_cont: Sender<String>,
    list_store: Arc<Mutex<Vec<String>>>,
) -> Result<(), ErrorClient> {
    handle_msgs_from_tcp_chat(
        dcc_handler,
        rx_dcc,
        tcp_stream,
        tx_cont,
        model_view_arc,
        list_store,
    )?;
    Ok(())
}

fn handle_msgs_from_tcp_chat(
    mut dcc_handler: DccHandler<TcpStream>,
    rx_dcc: Receiver<DirectMessage>,
    tcp_stream: Arc<TcpStream>,
    tx_cont: Sender<String>,
    model_view_arc: Arc<Mutex<Model>>,
    list_store: Arc<Mutex<Vec<String>>>,
) -> Result<(), ErrorClient> {
    match list_store.lock() {
        Ok(mut guard) => {
            guard.push(String::from("hola"));
        }
        Err(_) => {
            println!("Error al bloquear el Mutex D: ");
        }
    };

    let model_view_arc_clone = model_view_arc.clone();
    let tx_cont_clone = tx_cont.clone();

    thread::spawn(move || -> Result<(), ErrorClient> {
        loop {
            if let Ok(message) = rx_dcc.try_recv() {
                println!("{:?}", message);

                let mut model2 = model_view_arc_clone.lock().expect("cant lock model view");

                model2.add_msg(&message.get_user(), message.get_msg() + " (dcc)");

                if tx_cont_clone.send("true".to_string()).is_err() {
                    return Err(ErrorClient::ChannelError);
                };
            }
        }
    });

    loop {
        println!();
        let mut read_stream = tcp_stream.try_clone().expect("cant clone tcp");

        let mut buff: [u8; TAMANIO_MSJ] = [WHITESPACE_U8; TAMANIO_MSJ];
        read_stream
            .flush()
            .expect("no se pudo realizar flush de TCPStream");
        if let Ok(bytes_read) = read_stream.read(&mut buff) {
            if bytes_read == 0 {
                // return Err(ErrorClient::ServerClosed);
                break;
            }
            let msg = String::from_utf8_lossy(&buff);
            let x = msg.as_ref();
            println!("(180) MENSAJE RECIBIDO: {}", x);

            let mut model = model_view_arc.lock().expect("cant lock model view");

            if let Ok(c) = DccMessage::from_str(x) {
                println!("DCC MESSAGE: {:?}", c);
                match dcc_handler.handle_dcc_message_reception(c.clone()) {
                    Ok(_) => {
                        if c.command().eq(&dcc::command::DccCommand::Send) {
                            println!("Me llego un dcc send: {:?}", c);
                            println!("PREFIX {:?}", c.prefix());
                            let sender_user = c.prefix().expect("prefijo incorrecto");
                            let file_name =
                                c.get_param_from_msg(0).expect("parametros incorrectos");
                            let ip_host = c.get_param_from_msg(1).expect("parametros incorrectos");
                            let port = c.get_param_from_msg(2).expect("parametros incorrectos");

                            println!("sender_user {}", sender_user);
                            println!("name_file {}", file_name);
                            println!("ip_addres {}", ip_host);
                            println!("port {}", port);

                            model.add_msg(
                                &sender_user,
                                format!(
                                    "receiving file {}... on ip: {}, port: {}",
                                    file_name, ip_host, port
                                ),
                            );

                            model.set_ip_host_receive_download(ip_host);
                            model.set_ip_port_receive_download(port);
                            model.set_file_name_receive_download(file_name);

                            model.set_receive_downad(true);

                            if tx_cont.send("true".to_string()).is_err() {
                                return Err(ErrorClient::ChannelError);
                            };
                        } else if c.command().eq(&dcc::command::DccCommand::Pause) {
                            println!("Me llego un dcc pause: {:?}", c);

                            let sender_user = c.prefix().expect("prefijo incorrecto");

                            let ip_host = c.get_param_from_msg(1).expect("parametros incorrectos");
                            let port = c.get_param_from_msg(2).expect("parametros incorrectos");

                            println!("PAUSE: ip:{} port:{}", ip_host, port);

                            model.add_msg(
                                &sender_user,
                                format!(
                                    "paused file send on ip: {}, port: {}"
                                    , ip_host, port
                                ),
                            );

                            if tx_cont.send("true".to_string()).is_err() {
                                return Err(ErrorClient::ChannelError);
                            };
                        } else if c.command().eq(&dcc::command::DccCommand::Resume) {
                            println!("Me llego un dcc resume: {:?}", c);

                            let sender_user = c.prefix().expect("prefijo incorrecto");

                            let ip_host = c.get_param_from_msg(1).expect("parametros incorrectos");
                            let port = c.get_param_from_msg(2).expect("parametros incorrectos");

                            model.add_msg(
                                &sender_user,
                                format!(
                                    "resumed file send on ip: {}, port: {}",
                                     ip_host, port
                                ),
                            );

                            if tx_cont.send("true".to_string()).is_err() {
                                return Err(ErrorClient::ChannelError);
                            };
                        } else if c.command().eq(&dcc::command::DccCommand::MSG) {
                            println!("Me llego un dcc message: {:?}", c);

                            let sender_user = c.prefix().expect("prefijo incorrecto");

                            model.add_msg(&sender_user, c.to_string());

                            if tx_cont.send("true".to_string()).is_err() {
                                return Err(ErrorClient::ChannelError);
                            };
                        } else {
                            println!("es un DCC chat entrante!!! ");

                            let palabras: Vec<&str> = x.split_whitespace().collect();

                            println!("palabras: {:?}", palabras);
                            // El nick siempre estará en la segunda palabra
                            let nick = if let Some(nick) = palabras[0].strip_prefix(':') {
                                nick
                            } else {
                                panic!("No se encontró el nick en la cadena");
                            };

                            // El host estará en la penúltima palabra
                            let host = if let Some(host) = palabras.iter().rev().nth(1) {
                                host
                            } else {
                                panic!("No se encontró el host en la cadena");
                            };

                            // El puerto estará en la última palabra
                            let port = if let Some(port) = palabras.last() {
                                port
                            } else {
                                panic!("No se encontró el puerto en la cadena");
                            };

                            println!("Nick: {}", nick);
                            println!("Host: {}", host);
                            println!("Port: {}", port);

                            let tcp_ip = format!("{}:{}", host, port);

                            println!("Controller -> View: Sending signal");
                            if tx_cont.send("true".to_string()).is_err() {
                                return Err(ErrorClient::ChannelError);
                            };

                            model.add_dcc_connection(nick.to_string(), tcp_ip.clone());
                            model.add_msg(
                                &nick.to_string(),
                                format!("[Dcc connection established on tpc address: {}]", tcp_ip),
                            );

                            println!("{:?}", model.get_dcc_connection(nick.to_string()));
                        }
                    }
                    Err(e) => {
                        println!("Error DCC MESSAGE: {:?}", e)
                    }
                }
                continue;
            }

            match Reply::from_str(x) {
                Ok(c) => {
                    let reply = reply_maker::make_reply_format(c);
                    // println!("(183) REPLY: {}", reply);
                    model.set_reply(reply.clone());
                    println!("(187) Server -> Controller - received reply:{}||", reply);
                    println!("(188) Controller -> view - sending reply:{}||", reply);
                    // let selected = model.selected().unwrap();
                    // if reply.contains("Cannot join channel") || reply.contains("not channel operator") {
                    //     show_error_box(model.reply().clone().split(':').nth(1).unwrap_or("Error"));
                    // }
                    // if reply.contains("Cannot send to channel") {
                    //     println!("(194) CONTAINS CANNOT SEND CHANNEL");
                    //     show_error_box(model.reply().clone().split(':').nth(1).unwrap_or("Error"));
                    //     model.remove_msg_from_channel(&selected);
                    //     break;
                    // }
                    let x_copy = String::from(x);
                    let reply_copy = reply.clone();
                    if x_copy
                        .split(':')
                        .next()
                        .unwrap()
                        .contains(&Code::RplyAway.to_string())
                    {
                        let new_reply = format!("Unavailable User: {}", reply_copy);
                        model.set_reply(new_reply.clone());
                    }

                    if reply.contains('#') && !reply.contains('/') {
                        let mut reply_parsed = reply.trim().split(':');
                        let channel = reply_parsed.next().expect("");
                        let users_in_channel = reply_parsed.next().unwrap_or("");
                        if !users_in_channel.is_empty() {
                            println!(
                                "Reply Join Add model -chanel:{},-users:{}",
                                channel, users_in_channel
                            );
                            let users_iter = users_in_channel.split('~');
                            for user in users_iter {
                                if user.contains("Cannot send to channel") {
                                    model.remove_msg_from_channel(&channel.to_string());
                                    // show_error_box(user);
                                    break;
                                }
                                if user.contains("Cannot join channel") {
                                    model.delete_channel(&channel.to_string());
                                    break;
                                }
                                if user.contains("not channel operator") {
                                    break;
                                }
                                model.add_user_to_channel(&channel.to_string(), user.to_string());
                            }
                            model.remove_users_from_channel(
                                &channel.to_string(),
                                users_in_channel.to_string(),
                            );
                        }
                    }
                    if tx_cont.send(reply).is_err() {
                        return Err(ErrorClient::ChannelError);
                    };
                }
                Err(_) => {
                    println!("(227) Server -> Controler - received msj:{}||", msg.trim());

                    if msg.contains("dcc chat") {
                    } else {
                        let mut split_msg = x.split(':');
                        let nick = split_msg.next().expect("cant parse msg when splitting");
                        let msg_parsed = split_msg.next().expect("").trim();
                        if nick.contains('#') {
                            let msg_channel_parsed =
                                format!("{} : {}", msg_parsed, split_msg.next().expect("").trim());
                            println!("Model add: -channel:{},-msg:{}||", nick, msg_channel_parsed);
                            model.add_msg_to_channel(
                                &nick.to_string(),
                                msg_channel_parsed.to_string().clone(),
                            );
                        } else {
                            println!("Model add: -nick:{},-msg:{}||", nick, msg_parsed);
                            model.add_msg(&nick.to_string(), msg_parsed.to_string().clone());
                        }
                        println!("Controller -> View: Sending signal");
                        if tx_cont.send("true".to_string()).is_err() {
                            return Err(ErrorClient::ChannelError);
                        };
                    }
                }
            }
        };
    }
    Ok(())
}

fn init_model() -> Model {
    Model::new()
}

fn init_app_view(
    model_arc: Arc<Mutex<Model>>,
    tx_view: Sender<String>,
    tx_update: Sender<String>,
    rx_cont_arc: Arc<Mutex<Receiver<String>>>,
    dcc_handler: DccHandler<TcpStream>,
) {
    let app = Application::builder().application_id("APP_ID").build();
    let rx_cont_arc_share = rx_cont_arc;

    app.connect_activate(move |app| {
        build_ui(
            app,
            model_arc.clone(),
            tx_view.clone(),
            &tx_update,
            &rx_cont_arc_share,
            dcc_handler.clone(),
        );
    });
    app.run();
}

fn build_ui(
    app: &Application,
    model_arc: Arc<Mutex<Model>>,
    tx: Sender<String>,
    tx_update: &Sender<String>,
    rx_cont_arc: &Arc<Mutex<Receiver<String>>>,
    dcc_handler: DccHandler<TcpStream>,
) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("IRC")
        .default_width(600)
        .default_height(300)
        .build();
    /************************************************************************ */
    let model_copy1 = model_arc.clone();
    let model1 = model_copy1.lock().expect("cant lock model view");

    let mut contacts_window = ContactsWindow::new();
    let mut channels_window = ChannelsWindow::new();
    let mut users_window = UsersWindow::new();

    //TODO: Reallocated DCCStream

    //let mut ddc_chat_window = DDCChatWindow::new(tx.clone(),tx_update.clone(),Arc::new(Mutex::new(window.clone())),tcp_stream,list_store,ip_ddc_clone);
    let channel_user_window = ChannelUserMenu::new(window.clone(), app.clone());
    let channel_user_window_arc: Rc<RwLock<ChannelUserMenu>> =
        Rc::new(RwLock::new(channel_user_window));

    contacts_window.add_app(app.clone());
    contacts_window.add_window(window.clone());
    let contacts_window_arc = Rc::new(Mutex::new(contacts_window));

    channels_window.add_app(app.clone());
    channels_window.add_window(window.clone());
    let channels_window_arc = Rc::new(Mutex::new(channels_window));

    users_window.add_app(app.clone());
    users_window.add_window(window.clone());
    let users_window_arc = Rc::new(Mutex::new(users_window));

    //ddc_chat_window.add_app(Arc::new(Mutex::new(app.to_owned())));
    //ddc_chat_window.add_window(Arc::new(Mutex::new(window.clone())));
    //let ddc_chat_window_arc = Arc::new(Mutex::new(ddc_chat_window));
    //let ddc_chat_window_arc = Arc::new(Mutex::new(ddc_chat_window));

    let contacts_window_copy = contacts_window_arc.clone();
    let channels_window_copy = channels_window_arc.clone();
    let users_window_copy = users_window_arc.clone();
    let channel_user_window_copy = channel_user_window_arc.clone();

    let gesture = gtk::GestureClick::new();
    gesture.connect_pressed(move |gesture, _, _x, _y| {
        // gesture.set_state(gtk::EventSequenceState::Claimed);
        gesture.set_state(gtk::EventSequenceState::None);
        let contc = contacts_window_copy.lock().unwrap();
        let channel = channels_window_copy.lock().unwrap();
        let users = users_window_copy.lock().unwrap();
        let channel_user = channel_user_window_copy.write().unwrap();
        channel_user.unparent_popup();
        contc.unparent_popup();
        channel.unparent_popup();
        users.unparent_popup();
    });
    window.add_controller(&gesture);

    /************************************************************************ */
    let list_store_contacts = ListStore::new(StringObject::static_type());
    let tx_update_copy = tx_update.clone();
    let tx_copy = tx.clone();
    let model_copy = model_arc.clone();

    let contacts_window_arc_copy = contacts_window_arc.clone();
    let action_invite = gtk::gio::SimpleAction::new_stateful(
        "invite",
        Some(VariantTy::STRING),
        &false.to_variant(),
    );
    action_invite.set_enabled(true);
    action_invite.connect_activate(clone!(@weak window => move |_, channel_name| {
    let contacts_window_copy = contacts_window_arc_copy.lock().unwrap().clone();
    let model = model_copy.lock().expect("cant lock model view");
    let channel_name = channel_name.unwrap();

        println!("Channel name desdpués de apretar botón del meú: {}", channel_name.to_owned().str().unwrap());
        match model.selected() {
            None => {},//label_info.set_label("this is not selected contact/channel"),
            Some(selected) => {
                println!("SELECTED: : : : : : {}", selected);
                println!("mensaje: INVITE {} {}/{}", contacts_window_copy.get_contact(), contacts_window_copy.get_channel(), channel_name.to_owned().str().unwrap());
                let msj_parser = format!("INVITE {} {}", contacts_window_copy.get_contact(), channel_name.to_owned().str().unwrap());
                // println!("View -> Controller - Sending: {}", msj_parser);
                tx_update_copy
                    .send("true".to_string())
                    .expect("cant send to update channel");
                tx_copy.send(msj_parser)
                    .expect("cant send to View-> controller from channel");
            }
        }
        contacts_window_copy.unparent_popup();
        println!("Invite ACTION");
    }));

    let contacts_window_arc_copy2 = contacts_window_arc.clone();
    let tx_update_copy2 = tx_update.clone();
    let tx_copy2 = tx.clone();
    let model_copy2 = model_arc.clone();
    let action_invite_debug = action_invite.clone();
    let action_whois = gtk::gio::SimpleAction::new_stateful("whois", None, &false.to_variant());
    action_whois.connect_activate(clone!(@weak window => move |_, _| {
        println!("action invite data: {} ---- ", action_invite_debug.is_enabled());
        let contacts_window_copy = contacts_window_arc_copy2.lock().unwrap().clone();
        let model = model_copy2.lock().expect("cant lock model view");

        match model.selected() {
            None => {},//label_info.set_label("this is not selected contact/channel"),
            Some(_selected) => {
                let msj_parser = format!("WHOIS {}", contacts_window_copy.get_contact());
                // println!("View -> Controller - Sending: {}", msj_parser);
                tx_update_copy2
                    .send("true".to_string())
                    .expect("cant send to update channel");
                tx_copy2.send(msj_parser)
                    .expect("cant send to View-> controller from channel");
            }
        }
        contacts_window_copy.unparent_popup();
        println!("Whois ACTION");
    }));

    let model_copy3 = model_arc.clone();
    let tx_update_copy3 = tx_update.clone();
    let tx_copy3 = tx.clone();
    let channels_window_arc_copy = channels_window_arc.clone();
    let action_channel_status =
        gtk::gio::SimpleAction::new_stateful("channel_status", None, &false.to_variant());
    action_channel_status.connect_activate(clone!(@weak window => move |_, _| {
        let model = model_copy3.lock().expect("cant lock model view");
        let channels_window = channels_window_arc_copy.lock().unwrap();
        match model.selected() {
            None => {},//label_info.set_label("this is not selected contact/channel"),
            Some(_selected) => {
                let msj_parser = format!("MODE {}", channels_window.get_channel());
                // println!("View -> Controller - Sending: {}", msj_parser);
                tx_update_copy3
                    .send("true".to_string())
                    .expect("cant send to update channel");
                tx_copy3.send(msj_parser)
                    .expect("cant send to View-> controller from channel");
            }
        }
        channels_window.unparent_popup();
        println!("Status ACTION");
    }));

    let model_copy3 = model_arc.clone();
    let tx_update_copy3 = tx_update.clone();
    let tx_copy3 = tx.clone();
    let channels_window_arc_copy = channels_window_arc.clone();
    let action_config = gtk::gio::SimpleAction::new_stateful("config", None, &false.to_variant());

    let config_window = ConfigWindow::new(channels_window_arc_copy, tx_copy3, tx_update_copy3);
    let config_window_arc = Rc::new(Mutex::new(config_window.clone()));

    let channel_users_config_window = ChannelUserConfigWindow::new(
        channel_user_window_arc.clone(),
        tx.clone(),
        tx_update.clone(),
    );

    let tx_update_copy3 = tx_update.clone();
    let tx_copy3 = tx.clone();
    let channels_window_arc_copy = channels_window_arc.clone();
    let config_window_copy = config_window;
    action_config.connect_activate(clone!(@weak window => move |_, _| {
        let _model = model_copy3.lock().expect("cant lock model view");
        let channels_window = channels_window_arc_copy.lock().unwrap();
        println!("El channel que seleccioné: {}", channels_window.get_channel());

        let msj_parser = format!("MODE {}", channels_window.get_channel());
        // println!("View -> Controller - Sending: {}", msj_parser);
        tx_update_copy3
            .send("true".to_string())
            .expect("cant send to update channel");
        tx_copy3.send(msj_parser)
            .expect("cant send to View-> controller from channel");

        // let msj_parser2 = format!("TOPIC {}", channels_window.get_channel());
        // tx_update_copy4
        //     .send("true".to_string())
        //     .expect("cant send to update channel");
        // tx_copy4.send(msj_parser2)
        //     .expect("cant send to View-> controller from channel");
        config_window_copy.show();
        channels_window.unparent_popup();
        println!("Status ACTION");
    }));

    let topic_action =
        gtk::gio::SimpleAction::new_stateful("change_topic", None, &false.to_variant());
    let model_copy3 = model_arc.clone();
    let tx_update_copy3 = tx_update.clone();
    let tx_copy3 = tx.clone();
    let channels_window_arc_copy = channels_window_arc.clone();
    let config_window_copy = config_window_arc.clone();
    topic_action.connect_activate(clone!(@weak window => move |_, _|{
        let _model = model_copy3.lock().expect("cant lock model view");
        let channels_window = channels_window_arc_copy.lock().unwrap();
        let config_window = config_window_copy.lock().unwrap();
        let tx_upd = tx_update_copy3.clone();
        let tx = tx_copy3.clone();

        let msj_parser2 = format!("TOPIC {}", channels_window.get_channel());
        tx_upd
            .send("true".to_string())
            .expect("cant send to update channel");
        tx.send(msj_parser2)
            .expect("cant send to View-> controller from channel");

        config_window.show_topic_window();
        channels_window.unparent_popup();
    }));

    //USERS
    let model_copy5 = model_arc.clone();
    let tx_update_copy5 = tx_update.clone();
    let tx_copy5 = tx.clone();
    let users_window_arc_copy = users_window_arc.clone();
    let action_users_status =
        gtk::gio::SimpleAction::new_stateful("user_status", None, &false.to_variant());
    action_users_status.connect_activate(clone!(@weak window => move |_, _| {
        let model = model_copy5.lock().expect("cant lock model view");
        let users_window = users_window_arc_copy.lock().unwrap();
        match model.selected() {
            None => {},//label_info.set_label("this is not selected contact/channel"),
            Some(_selected) => {
                let msj_parser = format!("MODE {}", users_window.get_nick());
                // println!("View -> Controller - Sending: {}", msj_parser);
                tx_update_copy5
                    .send("true".to_string())
                    .expect("cant send to update channel");
                tx_copy5.send(msj_parser)
                    .expect("cant send to View-> controller from channel");
            }
        }
        users_window.unparent_popup();
        println!("Status ACTION");
    }));

    let model_copy6 = model_arc.clone();
    let tx_update_copy6 = tx_update.clone();
    let tx_copy6 = tx.clone();
    let users_window_arc_copy = users_window_arc.clone();
    let user_action_config =
        gtk::gio::SimpleAction::new_stateful("config_user", None, &false.to_variant());

    let user_config_window =
        UsersConfigWindow::new(users_window_arc_copy, tx_copy6, tx_update_copy6);
    let user_config_window_arc = Rc::new(Mutex::new(user_config_window.clone()));

    let tx_update_copy7 = tx_update.clone();
    let tx_copy7 = tx.clone();
    let users_window_arc_copy = users_window_arc.clone();
    let users_config_window_copy = user_config_window;
    user_action_config.connect_activate(clone!(@weak window => move |_, _| {
        let _model = model_copy6.lock().expect("cant lock model view");
        let users_window = users_window_arc_copy.lock().unwrap();
        println!("El user que seleccioné: {}", users_window.get_nick());

        let msj_parser = format!("MODE {}", users_window.get_nick());
        // println!("View -> Controller - Sending: {}", msj_parser);
        tx_update_copy7
            .send("true".to_string())
            .expect("cant send to update user");
        tx_copy7.send(msj_parser)
            .expect("cant send to View-> controller from user");

        users_config_window_copy.show();
        users_window.unparent_popup();
    }));
    //END USERS

    let channel_user_config =
        gtk::gio::SimpleAction::new_stateful("channel_user_config", None, &false.to_variant());
    let model_copy = model_arc.clone();
    let channel_user_menu_arc_copy = channel_user_window_arc.clone();
    let channel_user_config_window_copy = channel_users_config_window;
    channel_user_config.connect_activate(clone!(@weak window => move |_, _| {
        let _model = model_copy.lock().expect("cant lock model view");
        let channel_users_window = channel_user_menu_arc_copy.write().unwrap();
        channel_user_config_window_copy.show();
        channel_users_window.unparent_popup();
    }));

    let oper_add_action =
        gtk::gio::SimpleAction::new_stateful("oper_add_action", None, &false.to_variant());
    let model_copy = model_arc.clone();
    let channel_user_menu_arc_copy = channel_user_window_arc.clone();
    let tx_update_copy = tx_update.clone();
    let tx_copy = tx.clone();
    oper_add_action.connect_activate(clone!(@weak window => move |_, _| {
        let mut _model = model_copy.lock().expect("cant lock model view");
        let channel_users_window = channel_user_menu_arc_copy.write().unwrap();
        // println!("El user que seleccioné: {}", users_window.get_nick());

        let msj_parser = format!("MODE {} +o {}", channel_users_window.get_channel(), channel_users_window.get_contact());
        // // println!("View -> Controller - Sending: {}", msj_parser);
        tx_update_copy
            .send("true".to_string())
            .expect("cant send to update user");
            tx_copy.send(msj_parser)
            .expect("cant send to View-> controller from user");
        // model.remove_user_from_channel(channel_users_window.get_channel(), channel_users_window.get_contact().to_string());
        channel_users_window.unparent_popup();
    }));

    let oper_rmv_action =
        gtk::gio::SimpleAction::new_stateful("oper_rmv_action", None, &false.to_variant());
    let model_copy = model_arc.clone();
    let channel_user_menu_arc_copy = channel_user_window_arc.clone();
    let tx_update_copy = tx_update.clone();
    let tx_copy = tx.clone();
    oper_rmv_action.connect_activate(clone!(@weak window => move |_, _| {
        let mut _model = model_copy.lock().expect("cant lock model view");
        let channel_users_window = channel_user_menu_arc_copy.write().unwrap();
        // println!("El user que seleccioné: {}", users_window.get_nick());

        let msj_parser = format!("MODE {} -o {}", channel_users_window.get_channel(), channel_users_window.get_contact());
        // // println!("View -> Controller - Sending: {}", msj_parser);
        tx_update_copy
            .send("true".to_string())
            .expect("cant send to update user");
            tx_copy.send(msj_parser)
            .expect("cant send to View-> controller from user");
        // model.remove_user_from_channel(channel_users_window.get_channel(), channel_users_window.get_contact().to_string());
        channel_users_window.unparent_popup();
    }));

    let allow_spk_action =
        gtk::gio::SimpleAction::new_stateful("allow_spk_action", None, &false.to_variant());
    let model_copy = model_arc.clone();
    let channel_user_menu_arc_copy = channel_user_window_arc.clone();
    let tx_update_copy = tx_update.clone();
    let tx_copy = tx.clone();
    allow_spk_action.connect_activate(clone!(@weak window => move |_, _| {
        let mut _model = model_copy.lock().expect("cant lock model view");
        let channel_users_window = channel_user_menu_arc_copy.write().unwrap();
        // println!("El user que seleccioné: {}", users_window.get_nick());

        let msj_parser = format!("MODE {} +v {}", channel_users_window.get_channel(), channel_users_window.get_contact());
        // // println!("View -> Controller - Sending: {}", msj_parser);
        tx_update_copy
            .send("true".to_string())
            .expect("cant send to update user");
            tx_copy.send(msj_parser)
            .expect("cant send to View-> controller from user");
        // model.remove_user_from_channel(channel_users_window.get_channel(), channel_users_window.get_contact().to_string());
        channel_users_window.unparent_popup();
    }));

    let forbid_spk_action =
        gtk::gio::SimpleAction::new_stateful("forbid_spk_action", None, &false.to_variant());
    let model_copy = model_arc.clone();
    let channel_user_menu_arc_copy = channel_user_window_arc.clone();
    let tx_update_copy = tx_update.clone();
    let tx_copy = tx.clone();
    forbid_spk_action.connect_activate(clone!(@weak window => move |_, _| {
        let mut _model = model_copy.lock().expect("cant lock model view");
        let channel_users_window = channel_user_menu_arc_copy.write().unwrap();
        // println!("El user que seleccioné: {}", users_window.get_nick());

        let msj_parser = format!("MODE {} -v {}", channel_users_window.get_channel(), channel_users_window.get_contact());
        // // println!("View -> Controller - Sending: {}", msj_parser);
        tx_update_copy
            .send("true".to_string())
            .expect("cant send to update user");
            tx_copy.send(msj_parser)
            .expect("cant send to View-> controller from user");
        // model.remove_user_from_channel(channel_users_window.get_channel(), channel_users_window.get_contact().to_string());
        channel_users_window.unparent_popup();
    }));

    let kick_action = gtk::gio::SimpleAction::new_stateful("kick", None, &false.to_variant());
    let model_copy = model_arc.clone();
    let channel_user_menu_arc_copy = channel_user_window_arc.clone();
    let tx_copy = tx.clone();
    let tx_update_copy = tx_update.clone();
    let tx_update_copy2 = tx_update.clone();
    kick_action.connect_activate(clone!(@weak window => move |_, _| {
        let mut model = model_copy.lock().expect("cant lock model view");
        let channel_users_window = channel_user_menu_arc_copy.write().unwrap();
        // println!("El user que seleccioné: {}", users_window.get_nick());

        let msj_parser = format!("KICK {} {}", channel_users_window.get_channel(), channel_users_window.get_contact());
        // // println!("View -> Controller - Sending: {}", msj_parser);
        tx_update_copy
            .send("true".to_string())
            .expect("cant send to update user");
            tx_copy.send(msj_parser)
            .expect("cant send to View-> controller from user");
        model.remove_user_from_channel(channel_users_window.get_channel(), channel_users_window.get_contact().to_string());
        channel_users_window.unparent_popup();
    }));

    app.add_action(&action_config);
    app.add_action(&topic_action);
    app.add_action(&action_channel_status);
    app.add_action(&action_invite);
    app.add_action(&action_whois);
    app.add_action(&action_users_status);
    app.add_action(&channel_user_config);
    app.add_action(&kick_action);
    app.add_action(&user_action_config);
    app.add_action(&oper_add_action);
    app.add_action(&oper_rmv_action);
    app.add_action(&allow_spk_action);
    app.add_action(&forbid_spk_action);

    let list_store_msgs = ListStore::new(StringObject::static_type());
    let list_store_channels = ListStore::new(StringObject::static_type());
    let list_store_users = ListStore::new(StringObject::static_type());
    list_store_users.append(&StringObject::new(model1.nick()));
    let list_store_users_in_channel = ListStore::new(StringObject::static_type());
    let label_name = Label::builder().margin_top(1).margin_bottom(1).build(); //<- Esta es la parte de arriba, te muestra las respuestas del server.
    let label_reply = Label::builder().margin_top(1).margin_bottom(1).build();

    let updateable_chat_view = UpdatableChatView::new(
        list_store_users,
        list_store_msgs,
        list_store_contacts,
        list_store_channels,
        list_store_users_in_channel,
        label_name,
        label_reply,
    );

    //ddc_chat_window.add_list_store(updateable_chat_view.get_list_store_msgs());//?

    let arc_widget_chat_model = ArcWidgetChatModel::new(
        contacts_window_arc.clone(),
        channels_window_arc.clone(),
        users_window_arc,
        channel_user_window_arc,
        user_config_window_arc.clone(),
    );

    let box_chat = get_box_chat(
        tx.clone(),
        tx_update.clone(),
        model_arc.clone(),
        updateable_chat_view.clone(),
        arc_widget_chat_model,
        dcc_handler.clone(),
    );

    let box_loggin = get_box_login(tx.clone(), model_arc.clone(), window.clone(), box_chat);

    let ddc_send_file_window_arc = Rc::new(Mutex::new(DDCSendFileWindow::new(
        "p.txt".to_string(),
        "localhost".to_string(),
        "8090".to_string(),
        dcc_handler,
        tx_update_copy2,
        model_arc.clone(),
        "a".to_string(),
    )));
    //TODO: no mostrar aca;
    //ddc_send_file_window.show();

    init_update_handler(
        model_arc,
        rx_cont_arc.clone(),
        updateable_chat_view,
        WindowsReferences::new(
            contacts_window_arc,
            channels_window_arc,
            config_window_arc,
            user_config_window_arc,
            ddc_send_file_window_arc,
        ),
    );

    window.set_child(Some(&box_loggin));
    window.present();
}

fn init_update_handler(
    model_arc: Arc<Mutex<Model>>,
    rx_update: Arc<Mutex<Receiver<String>>>,
    updateable_chat_view: UpdatableChatView,
    windows_references: WindowsReferences,
) {
    let contacts_window_arc = windows_references.contacts_window_arc.to_owned();
    let _channels_window_arc = windows_references.channel_window_arc.to_owned();
    let config_window_arc = windows_references.config_window_arc.to_owned();
    let user_config_window_arc = windows_references.user_config_window_arc.to_owned();
    let dcc_send_window_arc = windows_references.dcc_send_window_arc.to_owned();

    let list_store_msgs = updateable_chat_view.get_list_store_msgs();
    let list_store_contacts = updateable_chat_view.get_list_store_contacts();
    let list_store_channels = updateable_chat_view.get_list_store_channels();
    let list_store_users_in_channel = updateable_chat_view.get_list_store_users_in_channel();
    let label_name = updateable_chat_view.get_name_label();
    let label_reply = updateable_chat_view.get_reply_label();

    let model_share = model_arc;
    let list_msgs_share = list_store_msgs;
    let list_contact_share = list_store_contacts;
    let list_channels_share = list_store_channels;
    let menu = contacts_window_arc
        .lock()
        .unwrap()
        .get_invite_menu()
        .to_owned();

    let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
    thread::spawn(move || -> Result<(), ErrorView> {
        let a = rx_update.lock().expect("cant lock rx_cont");
        loop {
            println!(
                "signal:{}",
                a.recv().expect("error signal recive to update view")
            );
            if sender.send(true).is_err() {
                return Err(ErrorView::ChannelError);
            };
        }
    });

    receiver.attach(
        None,
        clone!(@weak model_share , @weak list_msgs_share, @weak list_contact_share, @weak menu, @strong dcc_send_window_arc, => @default-return Continue(false),
                    move |_| {
                        list_msgs_share.remove_all();
                        list_contact_share.remove_all();
                        list_channels_share.remove_all();
                        list_store_users_in_channel.remove_all();
                        menu.remove_all();

                        let mut model = model_share.lock().expect("");
                        let selected = model.selected().unwrap_or_else(|| String::from("No selection"));
                        let msj_chat = format!("***Chat Session --> Nick:{}<---->Name:{}<--***",model.nick(),model.name());
                        label_name.set_property("label",msj_chat);
                        let msj_reply = format!("REPLY:{}", model.reply());
                        println!("(705) MSG REPLY: {}", msj_reply);
                        let config_window = config_window_arc.lock().unwrap();
                        if msj_reply.contains("Cannot join channel") || msj_reply.contains("not channel operator") || msj_reply.contains("no such nick") {
                            show_error_box(model.reply().split(':').nth(1).unwrap_or("Error"));
                            config_window.unpop();
                        }
                        if msj_reply.contains("Cannot send to channel") {
                            show_error_box(model.reply().split(':').nth(1).unwrap_or("Error"));
                            model.remove_msg_from_channel(&selected);
                        }

                        if msj_reply.contains("Unavailable User") {
                            show_error_box(&model.reply());
                            config_window.unpop();
                        }
                        label_reply.set_property("label",msj_reply);
                        // let config_window = config_window_arc.lock().unwrap();
                        config_window.set_checkboxes_state(model.reply());

                        let user_config_window = user_config_window_arc.lock().unwrap();
                        user_config_window.set_checkboxes_state(model.reply());
                        model.set_reply(String::from(""));
                        // update contacts
                        for contact in model.get_contacts(){
                            list_contact_share.append(&StringObject::new(contact));
                        }

                        // let sub_menu: gtk::gio::Menu = gtk::gio::Menu::new();
                        // update channels
                        let channels = model.get_channels();
                        for channel in channels{
                            list_channels_share.append(&StringObject::new(channel.clone()));
                            let menu_item = gtk::gio::MenuItem::new(Some(&channel), Some("app.invite"));
                            let channel_name = &*channel;
                            menu_item.set_action_and_target_value(Some("app.invite"), Some(&channel_name.to_variant()));
                            menu.append_item(&menu_item);
                            // menu.append(Some(&channel), Some("app.invite"));
                        }

                        if model.is_receive_downad(){
                            //TODO: deberia mostrarse aca
                            let mut download_window = dcc_send_window_arc.lock().expect("");
                            download_window.set_port(model.get_ip_host_receive_download());
                            download_window.set_ip(model.get_ip_port_receive_download());
                            // download_window.set
                            download_window.set_text_name_file_label(model.get_file_name_receive_download());
                            download_window.show();
                            model.set_receive_downad(false);
                        }

                        // menu.append_submenu(Some("Invite"), &sub_menu);
                        match model.selected() {
                            None => {println!("there is not selected contact/channel")}
                            Some(selected) => {
                                if selected.contains('#'){
                                    println!("selected channel:{}||",selected);
                                    match model.get_msgs_channel(&selected) {
                                        None => {println!("do not exist msgs for selected channel")}
                                        Some(msgs) => {
                                            // println!("there is #{} msgs from channel:{}",msgs.len(),selected);
                                            for msg in msgs{
                                                list_msgs_share.append(&StringObject::new(msg));
                                            }
                                        }
                                    };
                                    match model.get_users_channel(&selected) {
                                        None => {println!("do not exist users for selected channel")}
                                        Some(users) => {
                                            println!("there is #{} users from channel:{}", users.len(), selected);
                                            for user in users{
                                                list_store_users_in_channel.append(&StringObject::new(user));
                                            }
                                        }
                                    }
                                }else{
                                    println!("selected contact:{}||",selected);
                                    match model.get_msgs_contact(&selected) {
                                        None => {println!("do not exist msgs for selected contact")}
                                        Some(msgs) => {
                                            println!("there is #{} msgs from {}",msgs.len(),selected);
                                            for msg in msgs{
                                                list_msgs_share.append(&StringObject::new(msg));
                                            }
                                        }
                                    };
                                }
                            }
                        }
                        Continue(true)
                    }
        ),
    );
}

fn get_box_chat(
    tx: Sender<String>,
    tx_update: Sender<String>,
    model_arc: Arc<Mutex<Model>>,
    updatable_chat_view: UpdatableChatView,
    arc_widget_chat_model: ArcWidgetChatModel,
    dcc_handler: DccHandler<TcpStream>,
) -> gtk::Box {
    let chat_frame = construct_v_box();
    let chat_container = construct_h_box();

    let gtk_box_msgs = get_box_msgs(
        tx.clone(),
        tx_update.clone(),
        model_arc.clone(),
        updatable_chat_view.get_list_store_msgs(),
        dcc_handler,
    );
    let gtk_box_left_panel = get_box_left_panel(
        model_arc,
        tx_update,
        tx,
        updatable_chat_view.clone(),
        arc_widget_chat_model,
    );
    chat_container.append(&gtk_box_left_panel);
    chat_container.append(&gtk_box_msgs);

    chat_frame.append(&updatable_chat_view.get_name_label());
    chat_frame.append(&chat_container);
    chat_frame.append(&updatable_chat_view.get_reply_label());

    chat_frame
}

fn get_box_left_panel(
    model_arc: Arc<Mutex<Model>>,
    tx_update: Sender<String>,
    tx: Sender<String>,
    updatable_chat_view: UpdatableChatView,
    arc_widget_chat_model: ArcWidgetChatModel,
) -> gtk::Box {
    let box_left_panel = construct_v_box();
    let gtk_box_directory_contacts = get_box_directory_contacts(
        model_arc.clone(),
        tx_update.clone(),
        tx.clone(),
        updatable_chat_view.get_list_store_contacts(),
        arc_widget_chat_model.get_contacts_window_arc(),
    );

    let gtk_box_directory_channels = get_box_directory_channels(
        model_arc.clone(),
        tx_update.clone(),
        tx.clone(),
        updatable_chat_view.get_list_store_channels(),
        updatable_chat_view.get_list_store_users_in_channel(),
        arc_widget_chat_model.get_channel_window_arc(),
        arc_widget_chat_model.get_channel_users_window_arc(),
    );

    let gtk_box_directory_users = get_box_directory_users(
        model_arc.clone(),
        tx_update.clone(),
        tx.clone(),
        updatable_chat_view.get_list_store_users(),
        arc_widget_chat_model.get_users_window_arc(),
        arc_widget_chat_model.get_user_config_window(),
    );
    /******************************start********************************** */
    let box_who = gtk::Box::builder()
        .halign(Align::Fill)
        .hexpand(true)
        .vexpand(false)
        .orientation(Orientation::Horizontal)
        .build();
    let who_entry = construct_entry();
    let who_entry_copy1 = who_entry.clone();
    let who_entry_copy2 = who_entry.clone();
    let button_who = gtk::Button::builder().label("Who").build();
    let button_whois = gtk::Button::builder().label("Whois").build();
    /*******************************end********************************* */
    let tx_controller = tx.clone();
    let tx_update_share = tx_update.clone();
    let tx_controller2 = tx;
    let tx_update_share2 = tx_update;
    let model_arc_share = model_arc.clone();
    let model_arc_share2 = model_arc;
    button_who.connect_clicked(move |_| {
        let who = who_entry_copy1.clone().buffer().text();

        if !who.is_empty() {
            who_entry_copy1
                .buffer()
                .delete_text(0, Some(TAMANIO_CONTACTO));

            //TODO: Quitar si se encarga el CONTROLLER
            let _model = model_arc_share.lock().expect("cant lock model view");
            // let who = format!("#{}", who);
            // model.add_channel(who.clone());

            //TODO: enviar al controller -> server JOIN
            let who_command = format!("WHO {}", who);
            println!("View -> Controller - sending:{}||", who_command);

            // let model_test = model.selected().clone();
            // model.add_msg(&String::from("aaaa"), who_command.clone());
            tx_controller
                .send(who_command)
                .expect("cant send for channel");

            tx_update_share
                .send("true".to_string())
                .expect("cant send for channel");
        }
    });

    button_whois.connect_clicked(move |_| {
        let whois = who_entry_copy2.clone().buffer().text();

        if !whois.is_empty() {
            who_entry_copy2
                .buffer()
                .delete_text(0, Some(TAMANIO_CONTACTO));

            //TODO: Quitar si se encarga el CONTROLLER
            let _model = model_arc_share2.lock().expect("cant lock model view");
            // let whois = format!("#{}", whois);
            // model.add_channel(whois.clone());

            //TODO: enviar al controller -> server JOIN
            let whois_command = format!("WHOIS {}", whois);
            println!();
            println!("View -> Controller - sending:{}||", whois_command);
            tx_controller2
                .send(whois_command)
                .expect("cant send for channel");

            tx_update_share2
                .send("true".to_string())
                .expect("cant send for channel");
        }
    });
    box_left_panel.append(&gtk_box_directory_contacts);
    box_left_panel.append(&gtk_box_directory_channels);
    box_left_panel.append(&gtk_box_directory_users);
    /******************************start********************************** */
    box_who.append(&who_entry);
    box_who.append(&button_who);
    box_who.append(&button_whois);
    box_left_panel.append(&box_who);
    /*******************************end********************************* */

    box_left_panel
}

fn get_box_directory_users(
    model_arc: Arc<Mutex<Model>>,
    tx_update: Sender<String>,
    tx: Sender<String>,
    list_store_users: ListStore,
    user_window_arc: Rc<Mutex<UsersWindow>>,
    user_config_window: Rc<Mutex<UsersConfigWindow>>,
) -> gtk::Box {
    let box_container_directory_user = construct_v_box();

    let box_view_users = get_box_view_user(
        model_arc,
        tx,
        tx_update,
        list_store_users,
        user_window_arc,
        user_config_window,
    );

    box_container_directory_user.append(&box_view_users);

    box_container_directory_user
}

fn get_box_directory_channels(
    model_arc: Arc<Mutex<Model>>,
    tx_update: Sender<String>,
    tx: Sender<String>,
    list_store_channels: ListStore,
    list_store_users_in_channel: ListStore,
    channel_window_arc: Rc<Mutex<ChannelsWindow>>,
    channel_users_window_arc: Rc<RwLock<ChannelUserMenu>>,
) -> gtk::Box {
    let box_container_directory_channels = construct_v_box();

    let label_channels = Label::new(Some("Channels"));
    let box_add_channel = get_box_add_channel(model_arc.clone(), tx_update.clone(), tx.clone());
    let box_view_channels = get_box_view_channels(
        model_arc,
        tx_update,
        tx,
        list_store_channels,
        list_store_users_in_channel,
        channel_window_arc,
        channel_users_window_arc,
    );

    box_container_directory_channels.append(&label_channels);
    box_container_directory_channels.append(&box_add_channel);
    box_container_directory_channels.append(&box_view_channels);

    box_container_directory_channels
}

fn get_box_view_channels(
    model_arc: Arc<Mutex<Model>>,
    tx_update: Sender<String>,
    tx: Sender<String>,
    list_store_channels: ListStore,
    list_store_users_in_channel: ListStore,
    channel_window_arc: Rc<Mutex<ChannelsWindow>>,
    channel_users_window_arc: Rc<RwLock<ChannelUserMenu>>,
) -> gtk::Box {
    let box_container_directory_channels = construct_h_box();

    let scrolled_win_channels = get_scrolled_win_channels(
        model_arc,
        tx_update,
        tx,
        list_store_channels,
        channel_window_arc,
        channel_users_window_arc.clone(),
    );

    let scrolled_win_uses_in_channel =
        get_scrolled_win_users_in_channel(list_store_users_in_channel, channel_users_window_arc);

    box_container_directory_channels.append(&scrolled_win_channels);
    box_container_directory_channels.append(&scrolled_win_uses_in_channel);

    box_container_directory_channels
}

fn get_box_view_user(
    model_arc: Arc<Mutex<Model>>,
    tx: Sender<String>,
    tx_update: Sender<String>,
    _list_store_users: ListStore,
    user_window_arc: Rc<Mutex<UsersWindow>>,
    user_config_window: Rc<Mutex<UsersConfigWindow>>,
) -> gtk::Box {
    let box_container_directory_user = construct_h_box();

    let user_window_arc_copy = user_window_arc.clone();
    let user_window_copy = user_window_arc_copy.lock().unwrap();

    let user_menu = user_window_copy.get_menu();

    user_window_copy.add_model_to_popover(user_menu);

    let tx_copy = tx;
    let tx_update_copy = tx_update;
    let _user_menu_model_copy = user_window_copy.menu_model.clone();
    let user_window_arc_copy2 = user_window_arc;
    let model_arc2 = model_arc;
    let users_config_window_copy = user_config_window;
    let config_button = gtk::Button::builder().label("User Config").build();
    config_button.connect_clicked(move |_| {
        let model = model_arc2.lock().expect("cant lock model view");
        let mut users_window = user_window_arc_copy2.lock().unwrap();
        users_window.set_nick(model.nick());
        println!("El user que seleccioné: {}", model.nick());

        let msj_parser = format!("MODE {}", model.nick());
        // println!("View -> Controller - Sending: {}", msj_parser);
        tx_update_copy
            .send("true".to_string())
            .expect("cant send to update user");
        tx_copy
            .send(msj_parser)
            .expect("cant send to View-> controller from user");

        users_config_window_copy.lock().unwrap().show();
        users_window.unparent_popup();
    });
    box_container_directory_user.append(&config_button);

    box_container_directory_user
}

fn get_scrolled_win_users_in_channel(
    list_store_users_in_channel: ListStore,
    channel_users_window_arc: Rc<RwLock<ChannelUserMenu>>,
) -> ScrolledWindow {
    let win_users = construct_scrolled_window();
    let factory_list_view = construct_factory_list_view();

    let selection_model_channel = SingleSelection::new(Some(&list_store_users_in_channel));
    let list_view_channels =
        ListView::new(Some(&selection_model_channel), Some(&factory_list_view));

    /*******************START CONTEXT MENU*********************** */

    let channel_users_window_arc_copy = channel_users_window_arc.clone();
    let channel_users_window_copy = channel_users_window_arc_copy.write().unwrap();

    let channels_menu = channel_users_window_copy.get_menu();

    channel_users_window_copy.add_model_to_popover(channels_menu);

    let gesture = gtk::GestureClick::new();
    gesture.set_button(gtk::gdk::ffi::GDK_BUTTON_SECONDARY as u32);

    let list_view_copy = selection_model_channel;
    // let channel_users_window_copy = channel_users_window_arc;
    let channel_menu_model_copy = channel_users_window_copy.menu_model.clone();
    let channel_users_window_arc_copy2 = channel_users_window_arc;
    let list_view_channel_copy = list_view_channels.clone();
    gesture.connect_pressed(move |gesture, _, _x, _y| {
        gesture.set_state(gtk::EventSequenceState::Claimed);
        let user_idx = list_view_copy.selected();
        let mut contact_window_unref = channel_users_window_arc_copy2.write().unwrap();
        let user = list_view_copy
            .item(user_idx)
            .unwrap()
            .downcast::<StringObject>()
            .unwrap()
            .get_string();
        // contact_window_unref.set_channel(channel);
        contact_window_unref.set_contact(user);
        channel_menu_model_copy.set_parent(&list_view_channel_copy);
        channel_menu_model_copy.popup();
    });

    list_view_channels.add_controller(&gesture);

    /*******************END CONTEXT MENU*********************** */
    win_users.set_child(Some(&list_view_channels));

    win_users
}

fn get_scrolled_win_channels(
    model_arc: Arc<Mutex<Model>>,
    tx_update: Sender<String>,
    tx: Sender<String>,
    list_store_channels: ListStore,
    channel_window_arc: Rc<Mutex<ChannelsWindow>>,
    channel_users_window_arc: Rc<RwLock<ChannelUserMenu>>,
) -> ScrolledWindow {
    let win_channels = construct_scrolled_window();

    let factory_list_view = construct_factory_list_view();

    let selection_model_channel = SingleSelection::new(Some(&list_store_channels));
    let list_view_channels =
        ListView::new(Some(&selection_model_channel), Some(&factory_list_view));

    /*******************START CONTEXT MENU*********************** */

    let channel_users_window_arc_copy = channel_window_arc.clone();
    let channel_users_window_copy = channel_users_window_arc_copy.lock().unwrap();

    let channels_menu = channel_users_window_copy.get_menu();

    channel_users_window_copy.add_model_to_popover(channels_menu);

    let gesture = gtk::GestureClick::new();
    gesture.set_button(gtk::gdk::ffi::GDK_BUTTON_SECONDARY as u32);

    let list_view_copy = selection_model_channel;
    // let channel_users_window_copy = channel_window_arc;
    let channel_menu_model_copy = channel_users_window_copy.menu_model.clone();
    let channel_users_window_arc_copy2 = channel_window_arc;

    let list_view_channel_copy = list_view_channels.clone();
    gesture.connect_pressed(move |gesture, _, _x, _y| {
        gesture.set_state(gtk::EventSequenceState::Claimed);
        let channel_pos = list_view_copy.selected();
        let mut contact_window_unref = channel_users_window_arc_copy2.lock().unwrap();
        let channel = list_view_copy
            .item(channel_pos)
            .unwrap()
            .downcast::<StringObject>()
            .unwrap()
            .get_string();
        println!(
            "---------------channel pos: {}. channel: {}----------------",
            channel_pos, channel
        );
        contact_window_unref.set_channel(channel);
        channel_menu_model_copy.set_parent(&list_view_channel_copy);
        channel_menu_model_copy.popup();
    });

    list_view_channels.add_controller(&gesture);

    /*******************END CONTEXT MENU*********************** */

    update_selected_user_on_click(
        list_view_channels.clone(),
        model_arc,
        tx_update,
        tx,
        Some(channel_users_window_arc),
    );

    win_channels.set_child(Some(&list_view_channels));

    win_channels
}

fn get_box_add_channel(
    model_arc: Arc<Mutex<Model>>,
    tx_update: Sender<String>,
    tx: Sender<String>,
) -> gtk::Box {
    let box_channels = gtk::Box::builder()
        .halign(Align::Fill)
        .hexpand(true)
        .vexpand(false)
        .orientation(Orientation::Horizontal)
        .build();

    let box_buttons_handler_channel = gtk::Box::builder()
        .halign(Align::Fill)
        .hexpand(true)
        .vexpand(false)
        .orientation(Orientation::Vertical)
        .build();

    let entry_channel = construct_entry();
    let button_add_channel = construct_button_with_label("add: channel");
    let button_delete_channel = construct_button_with_label("delete: channel");

    let tx_update_share = tx_update.clone();
    let entry_copy = entry_channel.clone();
    let model_arc_share = model_arc.clone();
    let tx_controller = tx.clone();

    button_add_channel.connect_clicked(move |_| {
        let channel = entry_copy.buffer().text();
        if !channel.is_empty() {
            entry_copy.buffer().delete_text(0, Some(TAMANIO_CONTACTO));

            //TODO: Quitar si se encarga el CONTROLLER
            let mut model = model_arc_share.lock().expect("cant lock model view");
            let channel = format!("#{}", channel);
            model.add_channel(channel.clone());

            //TODO: enviar al controller -> server JOIN
            let chanel_command = format!("JOIN {}", channel);
            println!();
            println!("View -> Controller - sending:{}||", chanel_command);
            tx_controller
                .send(chanel_command)
                .expect("cant send for channel");

            tx_update_share
                .send("true".to_string())
                .expect("cant send for channel");
        }
    });

    let entry_copy = entry_channel.clone();

    button_delete_channel.connect_clicked(move |_| {
        let channel = entry_copy.buffer().text();
        if !channel.is_empty() {
            entry_copy.buffer().delete_text(0, Some(TAMANIO_CONTACTO));

            //TODO: Quitar si se encarga el CONTROLLER
            let mut model = model_arc.lock().expect("cant lock model view");
            let channel = format!("#{}", channel);
            model.delete_channel(&channel);

            //TODO: enviar al controller -> server JOIN
            let chanel_command = format!("PART {}", channel);
            println!();
            println!("View -> Controller - sending:{}||", chanel_command);
            tx.send(chanel_command)
                .expect("cant send view->controller channel");

            tx_update
                .send("true".to_string())
                .expect("cant send for update channel");
        }
    });

    box_buttons_handler_channel.append(&button_add_channel);
    box_buttons_handler_channel.append(&button_delete_channel);

    box_channels.append(&entry_channel);
    box_channels.append(&box_buttons_handler_channel);

    box_channels
}

fn get_box_directory_contacts(
    model_arc: Arc<Mutex<Model>>,
    tx_update: Sender<String>,
    tx: Sender<String>,
    list_store_contacts: ListStore,
    contacts_window_arc: Rc<Mutex<ContactsWindow>>,
) -> gtk::Box {
    let box_directory = construct_v_box();

    let label_contactos = Label::new(Some("Contacts"));

    let box_add_contact = get_box_add_contact(model_arc.clone(), tx_update.clone());
    let scrolled_win_contacts = get_scrolled_win_contacts(
        model_arc,
        tx_update,
        tx,
        list_store_contacts,
        contacts_window_arc,
    );

    box_directory.append(&label_contactos);
    box_directory.append(&box_add_contact);
    box_directory.append(&scrolled_win_contacts);

    box_directory
}

fn get_scrolled_win_contacts(
    model_arc: Arc<Mutex<Model>>,
    tx_update: Sender<String>,
    tx: Sender<String>,
    list_store_contacts: ListStore,
    contacts_window_arc: Rc<Mutex<ContactsWindow>>,
) -> ScrolledWindow {
    let win_contacts = construct_scrolled_window();

    let factory_list_view = construct_factory_list_view();

    let selection_model_channel = SingleSelection::new(Some(&list_store_contacts));
    let list_view_contacts =
        ListView::new(Some(&selection_model_channel), Some(&factory_list_view));

    /*******************START CONTEXT MENU*********************** */

    let contact_arc_copy = contacts_window_arc.clone();
    let contact_window = contact_arc_copy.lock().unwrap();

    let contact_menu = contact_window.get_menu();

    contact_window.add_model_to_popover(contact_menu);

    let gesture = gtk::GestureClick::new();
    gesture.set_button(gtk::gdk::ffi::GDK_BUTTON_SECONDARY as u32);

    let list_view_copy = selection_model_channel;
    let contacts_window_copy = contacts_window_arc;
    let contact_menu_model_copy = contact_window.menu_model.clone();
    let list_view_contacts_copy = list_view_contacts.clone();
    gesture.connect_pressed(move |gesture, _, _x, _y| {
        gesture.set_state(gtk::EventSequenceState::Claimed);
        let contact_pos = list_view_copy.selected();

        let mut contact_window_unref = contacts_window_copy.lock().unwrap();
        let contact = list_view_copy
            .item(contact_pos)
            .unwrap()
            .downcast::<StringObject>()
            .unwrap()
            .get_string();
        println!(
            "---------------Contact pos: {}. Contact: {}----------------",
            contact_pos, contact
        );
        contact_window_unref.set_contact(contact);
        contact_menu_model_copy.set_parent(&list_view_contacts_copy);
        contact_menu_model_copy.popup();
    });

    list_view_contacts.add_controller(&gesture);

    /*******************END CONTEXT MENU*********************** */
    update_selected_user_on_click(list_view_contacts.clone(), model_arc, tx_update, tx, None);

    win_contacts.set_child(Some(&list_view_contacts));

    win_contacts
}

fn get_box_add_contact(model_arc: Arc<Mutex<Model>>, tx_update: Sender<String>) -> gtk::Box {
    let gtk_box_add_contact = gtk::Box::builder()
        .vexpand(false)
        .orientation(Orientation::Horizontal)
        .build();

    let entry_contact = construct_entry();
    let button_add_contact = construct_button_with_label("add: contact");

    let entry_copy = entry_contact.clone();

    button_add_contact.connect_clicked(move |_| {
        let contact = entry_copy.buffer().text();
        if !contact.is_empty() {
            entry_copy.buffer().delete_text(0, Some(TAMANIO_CONTACTO));
            let mut model = model_arc.lock().expect("cant lock model view");
            model.add_contact(contact);
            tx_update
                .send("true".to_string())
                .expect("cant send to update channel");
        }
    });

    gtk_box_add_contact.append(&entry_contact);
    gtk_box_add_contact.append(&button_add_contact);

    gtk_box_add_contact
}

fn get_list_view_msgs(list_store_msgs: ListStore) -> ListView {
    let factory = construct_factory_list_view();

    let selection_model = NoSelection::new(Some(&list_store_msgs));
    let list_view = ListView::new(Some(&selection_model), Some(&factory));

    //TODO: Not used - behavior with double click in msgs
    list_view.connect_activate(move |list_view, position| {
        let model = list_view.model().expect("The model has to exist.");
        let _string_object = model
            .item(position)
            .expect("The item has to exist.")
            .downcast::<StringObject>()
            .expect("The item has to be an `StringObject`.");
    });

    list_view
}

fn get_box_msgs(
    tx: Sender<String>,
    tx_update: Sender<String>,
    model_arc: Arc<Mutex<Model>>,
    list_store_msgs: ListStore,
    dcc_handler: DccHandler<TcpStream>,
) -> gtk::Box {
    let label_info = Label::new(Some("***messages***"));

    let gtk_box_chat_interact = construct_v_box();

    let scrolled_window_msgs = get_scrolled_window_msgs(list_store_msgs);
    let writting_box = get_writting_box(tx, tx_update, model_arc, label_info.clone(), dcc_handler);

    gtk_box_chat_interact.append(&label_info);
    gtk_box_chat_interact.append(&scrolled_window_msgs);
    gtk_box_chat_interact.append(&writting_box);

    gtk_box_chat_interact
}

fn get_writting_box(
    tx: Sender<String>,
    tx_update: Sender<String>,
    model_arc: Arc<Mutex<Model>>,
    label_info: Label,
    dcc_handler: DccHandler<TcpStream>,
) -> gtk::Box {
    let writting_box = gtk::Box::builder()
        .hexpand(true)
        .vexpand(false)
        .halign(Align::Fill)
        .valign(Align::Fill)
        .orientation(Orientation::Horizontal)
        .build();

    let send_box = gtk::Box::builder()
        .hexpand(false)
        .vexpand(false)
        .halign(Align::Fill)
        .valign(Align::Fill)
        .orientation(Orientation::Vertical)
        .build();


    let zip_box = gtk::Box::builder()
        .hexpand(false)
        .vexpand(false)
        .halign(Align::Fill)
        .valign(Align::Fill)
        .orientation(Orientation::Horizontal)
        .build();

    let button_send_chat = construct_button_with_label("chat");
    let entry_msg_chat = Entry::builder()
        .max_length((TAMANIO_MSJ as i16 * 0.125 as i16) as i32)
        .max_width_chars(TAMANIO_MSJ as i32)
        .valign(Align::Center)
        .vexpand(false)
        .build();

    let model_arc2 = model_arc.clone();

    let model_arc3 = model_arc.clone();

    let model_arc4 = model_arc.clone();

    let tx_update_copy = tx_update.clone();
    let tx_update_copy2 = tx_update.clone();
    let tx_update_copy3 = tx_update.clone();
    //let ddc_chat_window_copy = ddc_chat_window_arc;

    let label_info_clone = label_info.clone();

    let label_info_clone2 = label_info.clone();

    let label_info_clone3 = label_info.clone();

    let entry_copy_ddc = entry_msg_chat.clone();
    let entry_copy_ddc2 = entry_msg_chat.clone();
    let entry_copy_ddc3 = entry_msg_chat.clone();
    let dcc_handle_clone = dcc_handler.clone();
    let dcc_handle_clone2 = dcc_handler.clone();

    let ddc_button = construct_button_with_label("DCC Start");

    ddc_button.connect_clicked(move |_| {
        /*let model = model_arc2.lock().expect("cant lock model view");
        let mut users_window = user_window_arc_copy2.lock().unwrap();
        users_window.set_nick(model.nick());
        println!("El user que seleccioné: {}", model.nick());

        let msj_parser = format!("DDC START {} : {}", selected, msj.trim());
        println!("View -> Controller - Sending: {}", msj_parser);
        tx_update_copy
            .send("true".to_string())
            .expect("cant send to update channel");
        tx.send(msj_parser)
            .expect("cant send to View-> controller from channel");*/

        let msj = entry_copy_ddc.buffer().text();
        entry_copy_ddc
            .buffer()
            .delete_text(0, Some(TAMANIO_MSJ as u16));

        let mut model = model_arc2.lock().expect("cant lock model view");

        let mut dcc_clone = dcc_handle_clone.clone();

        //TODO: to build if selected is channel or contact
        match model.selected() {
            None => label_info_clone.set_label("this is not selected contact/channel"),
            Some(selected) => {
                let msj_parser = format!(
                    "PRIVMSG {} dcc chat chat {}",
                    selected,
                    msj.trim().replace(':', " ")
                );
                let dcc = dcc_clone.handle_dcc_message_send(msj_parser.clone());
                model.add_dcc_connection(selected.clone(), msj.trim().to_owned());
                match dcc {
                    Ok(_) => {}
                    Err(_e) => {}
                }
                println!("View -> DCC Handler - Sending: {}", msj_parser);

                model.add_msg(
                    &selected,
                    "Started DCC Conection in: ".to_string() + msj.trim(),
                );
                tx_update_copy2
                    .send("true".to_string())
                    .expect("cant send to update channel");
            }
        }
    });

    let ddc_button_send = construct_button_with_label("chat dcc");

    ddc_button_send.connect_clicked(move |_| {
        let msj = entry_copy_ddc2.buffer().text();
        entry_copy_ddc2
            .buffer()
            .delete_text(0, Some(TAMANIO_MSJ as u16));

        let mut model = model_arc3.lock().expect("cant lock model view");

        let mut dcc_clone = dcc_handler.clone();

        //TODO: to build if selected is channel or contact
        match model.selected() {
            None => label_info_clone2.set_label("this is not selected contact/channel"),
            Some(selected) => {
                let conection = model.get_dcc_connection(selected.clone());
                match conection {
                    Some(ip_port) => {
                        let msj_parser = format!(":{} {} //END", ip_port, msj.trim());
                        let dcc = dcc_clone.handle_dcc_message_send(msj_parser.clone());
                        println!("DEBUG - dcc sending status = {:?}", dcc);
                        model.add_msg(&selected, msj.clone() + " (dcc)");
                        tx_update_copy
                            .send("true".to_string())
                            .expect("cant send to update channel");
                        println!("View -> DCC Handler - Sending: {}", msj_parser);
                    }
                    None => {
                        println!("No DCC Conection with these contact yet");
                    }
                }
            }
        }
    });

    let send_file_button = gtk::Button::builder().icon_name("mail-attachment").build();

    let checkbox_zip = gtk::Switch::builder().name("zip").build();

    let checkbox_duplicate = checkbox_zip.clone();

    let window_self = ApplicationWindow::builder()
        .title("DDC Conection")
        .default_width(300)
        .default_height(300)
        .build();

    send_file_button.connect_clicked(move |_| {
        let mut model = model_arc4.lock().expect("cant lock model view");

        let dcc_clone5 = Mutex::new(dcc_handle_clone2.clone());

        match model.selected() {
            None => label_info_clone3.set_label("this is not selected contact/channel "),
            Some(selected) => {
                let msj = entry_copy_ddc3.buffer().text();
                entry_copy_ddc3
                    .buffer()
                    .delete_text(0, Some(TAMANIO_MSJ as u16));

                let file_chooser = FileChooserDialog::new(
                    Some("Attach File"),
                    Some(&window_self), //Some(&window),
                    FileChooserAction::Open,
                    &[("Open", ResponseType::Ok), ("Cancel", ResponseType::Cancel)],
                );

                let user = selected.clone();
                let send_zip = checkbox_duplicate.is_active();
                let mut zip = "".to_string();
                if send_zip {
                    zip.push_str("zip");
                }
                file_chooser.show();
                file_chooser.connect_response(move |dialog, response_type| {
                    if response_type == ResponseType::Ok {
                        if let Some(filename) = dialog.file() {
                            println!("Selected file: {}", filename.parse_name());

                            let msj_parser = format!(
                                "PRIVMSG {} dcc send {} {} {}",
                                selected,
                                filename.parse_name(),
                                msj.trim().replace(':', " "),
                                zip
                            );
                            let mut dcc = dcc_clone5.lock().expect("Dcc send file not found ");
                            let _dcc = dcc.handle_dcc_message_send(msj_parser.clone());
                            println!("View -> DCC Handler - Sending: {} ", msj_parser);
                        }
                    }
                    dialog.close();
                });
                model.add_msg(&user, "file sending via dcc...".to_string());
                tx_update_copy3
                    .send("true".to_string())
                    .expect("cant send to update channel");
            }
        }
    });

    let entry_copy = entry_msg_chat.clone();
    let tx_update_copy = tx_update;
    button_send_chat.connect_clicked(move |_| {
        let msj = entry_copy.buffer().text();
        entry_copy.buffer().delete_text(0, Some(TAMANIO_MSJ as u16));

        let mut model = model_arc.lock().expect("cant lock model view");
        match model.selected() {
            None => label_info.set_label("this is not selected contact/channel"),
            Some(selected) => {
                label_info.set_label("***messages***");
                println!();
                if selected.contains('#') {
                    println!(
                        "Add model -user:{} ,-msj:{},-to channel:{}",
                        model.nick(),
                        msj,
                        selected
                    );
                    model.add_msg_to_channel(&selected, msj.clone());
                } else {
                    println!(
                        "Add model -user:{} ,-msj:{},-to contact:{}",
                        model.nick(),
                        msj,
                        selected
                    );
                    model.add_msg(&selected, msj.clone());
                }
                let msj_parser = format!("PRIVMSG {} : {}", selected, msj.trim());
                println!("View -> Controller - Sending: {}", msj_parser);
                tx_update_copy
                    .send("true".to_string())
                    .expect("cant send to update channel");
                tx.send(msj_parser)
                    .expect("cant send to View-> controller from channel");
            }
        }
    });

    let label_zip = gtk::Label::builder().label("zip").build();
    
    

    writting_box.append(&entry_msg_chat);
    writting_box.append(&ddc_button);
    writting_box.append(&button_send_chat);
    writting_box.append(&ddc_button_send);

    send_box.append(&send_file_button);

    zip_box.append(&label_zip);
    zip_box.append(&checkbox_zip);
    
    send_box.append(&zip_box);
    writting_box.append(&send_box);
    

    writting_box
}

fn get_scrolled_window_msgs(list_store_msgs: ListStore) -> ScrolledWindow {
    let list_view_msgs = get_list_view_msgs(list_store_msgs);
    let scrolled_window_msgs = construct_scrolled_window();

    scrolled_window_msgs.set_child(Some(&list_view_msgs));

    scrolled_window_msgs
}

fn get_box_login(
    tx: Sender<String>,
    model_arc: Arc<Mutex<Model>>,
    window: ApplicationWindow,
    box_chat: gtk4::Box,
) -> gtk::Box {
    let box_login = gtk::Box::builder()
        .halign(Align::Center)
        .valign(Align::Center)
        .orientation(Orientation::Vertical)
        .build();

    let box_h1_loggin = construct_v_box();
    let box_h2_loggin = construct_v_box();
    let box_h3_loggin = construct_v_box();
    let box_h4_loggin = construct_v_box();
    let box_h5_loggin = construct_v_box();
    let box_h6_loggin = construct_v_box();
    let entry_user = construct_entry();
    let entry_host = construct_entry();
    let entry_server = construct_entry();
    let entry_real = construct_entry();
    let entry_nick = construct_entry();
    let entry_pass = construct_entry();
    let button_login = construct_button_with_label("create & login");

    let label_error = Label::builder()
        .valign(Align::Center)
        .halign(Align::Start)
        // .hexpand(true)
        // .margin_end(50)
        .build();

    let login_view = LoginView::new(
        button_login.clone(),
        entry_user.clone(),
        entry_host.clone(),
        entry_server.clone(),
        entry_real.clone(),
        entry_nick.clone(),
        entry_pass.clone(),
    );

    button_create_user_login(
        login_view,
        label_error.clone(),
        window,
        box_chat,
        model_arc,
        tx,
    );

    let label_nick = gtk::Label::builder().label("Nickname").xalign(0.01).build();
    let label_pass = gtk::Label::builder().label("Password").xalign(0.01).build();
    let label_user = gtk::Label::builder().label("Username").xalign(0.01).build();
    let label_host = gtk::Label::builder().label("Hostname").xalign(0.01).build();
    let label_real = gtk::Label::builder().label("Realname").xalign(0.01).build();
    let label_server = gtk::Label::builder()
        .label("Servername")
        .xalign(0.01)
        .build();

    // box_h1_loggin.append(&Label::new(Some("name")));
    box_h1_loggin.append(&label_nick);
    box_h1_loggin.append(&entry_nick);
    //box_h2_loggin.append(&Label::new(Some("Nickname")).set_xalign(Align::Start));
    box_h2_loggin.append(&label_user);
    box_h2_loggin.append(&entry_user);

    box_h3_loggin.append(&label_host);
    box_h3_loggin.append(&entry_host);

    box_h4_loggin.append(&label_real);
    box_h4_loggin.append(&entry_real);

    box_h5_loggin.append(&label_server);
    box_h5_loggin.append(&entry_server);

    box_h6_loggin.append(&label_pass);
    box_h6_loggin.append(&entry_pass);

    box_login.append(&label_error);
    box_login.append(&box_h1_loggin);
    box_login.append(&box_h2_loggin);
    box_login.append(&box_h3_loggin);
    box_login.append(&box_h4_loggin);
    box_login.append(&box_h5_loggin);
    box_login.append(&box_h6_loggin);
    box_login.append(&button_login);

    box_login
}

fn button_create_user_login(
    login_view: LoginView,
    label_error: Label,
    window: ApplicationWindow,
    box_chat: gtk::Box,
    model_arc: Arc<Mutex<Model>>,
    tx_copy: Sender<String>,
) {
    login_view.get_button().connect_clicked(move |_| {
        let nick = login_view.get_entry_nick().buffer().text();
        login_view
            .get_entry_nick()
            .buffer()
            .delete_text(0, Some(TAMANIO_MSJ as u16));

        let username = login_view.get_entry_user().buffer().text();
        login_view
            .get_entry_user()
            .buffer()
            .delete_text(0, Some(TAMANIO_MSJ as u16));

        let host = login_view.get_entry_host().buffer().text();
        login_view
            .get_entry_user()
            .buffer()
            .delete_text(0, Some(TAMANIO_MSJ as u16));

        let server = login_view.get_entry_server().buffer().text();
        login_view
            .get_entry_user()
            .buffer()
            .delete_text(0, Some(TAMANIO_MSJ as u16));

        let real = login_view.get_entry_real().buffer().text();
        login_view
            .get_entry_user()
            .buffer()
            .delete_text(0, Some(TAMANIO_MSJ as u16));

        let pass = login_view.get_entry_pass().buffer().text();
        login_view
            .get_entry_user()
            .buffer()
            .delete_text(0, Some(TAMANIO_MSJ as u16));

        if !nick.is_empty()
            && !username.is_empty()
            && !host.is_empty()
            && !server.is_empty()
            && !real.is_empty()
        {
            let mut x = model_arc.lock().expect("cant lock model view");
            x.set_name(username.clone());
            x.set_nick(nick.clone());
            x.set_host(host.clone());
            x.set_server(server.clone());
            x.set_realname(real.clone());

            println!();
            println!("Model view username change to: {}", x.name());
            println!("Model view nick change to: {}", x.nick());

            if !pass.is_empty() {
                x.set_pass(pass.clone());
                let msg_pass_parsed = format!("PASS {}", pass.trim()).trim().to_string();
                println!("View -> Controller - Sending: {}", msg_pass_parsed);
                tx_copy
                    .send(msg_pass_parsed)
                    .expect("cant send View->Controller for channel");
            }

            let msg_user_parsed = format!(
                "USER {} {} {} :{}",
                username.trim(),
                host.trim(),
                server.trim(),
                real.trim()
            )
            .trim()
            .to_string();
            println!("View-> Controller - Sending: {}", msg_user_parsed);
            tx_copy
                .send(msg_user_parsed)
                .expect("cant send View->Controller for channel");

            //Esto lo voy a cambiar momentáneamente, para que no utilice la contraseña de la vista y solo use el
            let msg_nick_parsed = format!("NICK {}", nick.trim()).trim().to_string();
            println!("View -> Controller - Sending: {}", msg_nick_parsed);
            tx_copy
                .send(msg_nick_parsed)
                .expect("cant send View->Controller for channel");

            //cambiar a CHAT
            window.set_child(Some(&box_chat));
        } else {
            label_error.set_label("some entry is empty, retry");
        }
    });
}

fn construct_v_box() -> gtk::Box {
    gtk::Box::builder()
        .halign(Align::Fill)
        .hexpand(true)
        .vexpand(true)
        .valign(Align::Fill)
        .orientation(Orientation::Vertical)
        .build()
}

fn construct_h_box() -> gtk::Box {
    gtk::Box::builder()
        .halign(Align::Fill)
        .hexpand(true)
        .vexpand(true)
        .valign(Align::Fill)
        .orientation(Orientation::Horizontal)
        .build()
}

fn construct_factory_list_view() -> SignalListItemFactory {
    let factory = SignalListItemFactory::new();

    factory.connect_setup(move |_, list_item| {
        let label = Label::new(None);
        list_item.set_child(Some(&label));

        list_item
            .property_expression("item")
            .chain_property::<StringObject>("string")
            .bind(&label, "label", Widget::NONE);
    });
    factory
}

fn construct_scrolled_window() -> ScrolledWindow {
    ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never) // Disable horizontal scrolling
        .min_content_width(360)
        .halign(Align::Fill)
        .hexpand(true)
        .vexpand(true)
        .valign(Align::Fill)
        .build()
}

fn construct_entry() -> Entry {
    Entry::builder()
        .max_length(TAMANIO_CONTACTO as i32)
        .width_chars(TAMANIO_CONTACTO as i32 * 0.25 as i32)
        .margin_bottom(2)
        .margin_top(2)
        .margin_start(2)
        .margin_end(2)
        .vexpand(false)
        .build()
}

fn construct_button_with_label(label: &str) -> Button {
    Button::builder()
        .margin_top(2)
        .margin_bottom(2)
        .margin_start(2)
        .margin_end(2)
        .focus_on_click(true)
        .halign(Align::Fill)
        .valign(Align::Center)
        .label(label)
        .build()
}

fn update_selected_user_on_click(
    list_view: ListView,
    model_arc: Arc<Mutex<Model>>,
    tx_update: Sender<String>,
    tx: Sender<String>,
    channel_users_window_arc: Option<Rc<RwLock<ChannelUserMenu>>>,
) {
    list_view.connect_activate(move |list_view, position| {
        let model = list_view.model().expect("The model has to exist.");
        let string_object = model
            .item(position)
            .expect("The item has to exist.")
            .downcast::<StringObject>()
            .expect("The item has to be an `StringObject`.");
        if string_object.clone().get_string().starts_with('#')
            || string_object.clone().get_string().starts_with('&')
        {
            let msj = format!("NAMES {}", string_object.clone().get_string());
            println!("View -> Controller - Sending: {}", msj);
            let _x = tx.send(msj);
        }
        let mut model_view = model_arc.lock().expect("cant lock model view");
        model_view.set_selected(string_object.clone().get_string());

        tx_update
            .send("true".to_string())
            .expect("cant send to update channel");

        match &channel_users_window_arc {
            Some(channel_users_window_arc_some) => {
                println!(
                    "(1673) VOY A UPDATEAR EL CHANNEL COMO {}",
                    string_object.clone().get_string()
                );
                channel_users_window_arc_some
                    .write()
                    .unwrap()
                    .set_channel(string_object.get_string());
            }
            None => {}
        }
    });
}

fn show_error_box(error_msg: &str) {
    println!("ERROR MESSAGE SHOW BOX -{}-", error_msg);
    let error_dialog = gtk::MessageDialog::builder()
        .modal(true)
        .deletable(true)
        .default_height(100)
        .default_width(100)
        .build();
    // let window = window.lock().unwrap().window.clone().unwrap().lock().unwrap();
    println!("1");
    error_dialog.set_modal(true);
    println!("2");
    // error_dialog.set_parent(&window);
    error_dialog.set_text(Some(error_msg));
    error_dialog.set_hexpand(true);
    println!("3");
    error_dialog.show();
    println!("4");
}
