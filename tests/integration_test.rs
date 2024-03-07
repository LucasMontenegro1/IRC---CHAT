// use crate::common::init_client;
// use crate::common::send_message;
// use irc_project::client::run_client_console;
// use irc_project::server::init_server;
// use std::fs::File;
// use std::io::{stdin, stdout, Read, Write};
// use std::path::Path;
// use std::process::{exit, Command};
// use std::sync::{mpsc, Arc, Mutex};
// use std::thread;

// mod common;

// #[test]
// fn test_initialize_server_connect_client() {
//     let address = format!("127.0.0.1:10000");
//     let server_name = "s1".to_string();

//     let address_share = address.clone();

//     thread::Builder::new()
//         .name("server".to_string())
//         .spawn(move || {
//             init_server(address_share, server_name).expect("error al iniciar server");
//         })
//         .expect("cant create thread");
//     assert!(init_client(address.clone()).is_ok());
// }

// #[ignore]
// #[test]
// fn test_server_stress() {
//     let cant_server = 7000;

//     let address = format!("127.0.0.1:10000");
//     let server_name = "s1".to_string();

//     let address_share = address.clone();

//     thread::Builder::new()
//         .name("server".to_string())
//         .spawn(move || {
//             init_server(address_share, server_name).expect("error al iniciar server");
//         })
//         .expect("cant create server thread");

//     for _ in 0..cant_server {
//         assert!(init_client(address.clone()).is_ok());
//     }
// }

// #[test]
// fn test_two_clients_comunicate_privmsg_on_server() {
//     // let address = format!("127.0.0.1:10000");
//     // let server_name = "s1".to_string();
//     //
//     // let address_share = address.clone();
//     //
//     // thread::Builder::new()
//     //     .name("server".to_string())
//     //     .spawn(move || {
//     //         init_server(address_share, server_name).expect("error al iniciar server");
//     //     })
//     //     .expect("cant create thread server");
//     //
//     // Command::new("sleep").arg("10").spawn().expect("");
//     //
//     // let mut tcp_stream = init_client(address.clone()).expect("cant obtain client tcp");
//     // Command::new("sleep").arg("10").spawn().expect("");
//     // let mut tcp_stream2 = init_client(address.clone()).expect("cant obtain client tcp");
//     // Command::new("sleep").arg("10").spawn().expect("");
//     //
//     // let path_to_read = Path::new("tests/files/client1.txt");
//     // let mut file = File::create(&path_to_read).expect("file open error");
//     // let mut box_file = Box::new(file);
//     //
//     // let (tx1, rx1) = mpsc::channel::<String>();
//     // let tx_clone1 = tx1.clone();
//     // thread::Builder::new()
//     //     .name("client1".to_string())
//     //     .spawn(move || {
//     //         assert!(
//     //             run_client_console(tcp_stream, Box::new(stdin()), box_file, tx_clone1, rx1)
//     //                 .is_err()
//     //         );
//     //     })
//     //     .expect("cant create thread");
//     // Command::new("sleep").arg("10").spawn().expect("");
//     // let path_to_read2 = Path::new("tests/files/client2.txt");
//     // let mut file2 = File::create(&path_to_read2).expect("file open error");
//     // let mut box_file2 = Box::new(file2);
//     //
//     // let (tx2, rx2) = mpsc::channel::<String>();
//     // let tx2_clone = tx2.clone();
//     // thread::Builder::new()
//     //     .name("client2".to_string())
//     //     .spawn(move || {
//     //         assert!(
//     //             run_client_console(tcp_stream2, Box::new(stdin()), box_file2, tx2_clone, rx2)
//     //                 .is_err()
//     //         );
//     //     })
//     //     .expect("cant creat thread");
//     //
//     // send_message(tx1.clone(), "USER x x x x".to_string());
//     // send_message(tx1.clone(), "NICK papa".to_string());
//     //
//     // send_message(tx2.clone(), "USER x x x x".to_string());
//     // send_message(tx2.clone(), "NICK pepe".to_string());
//     // send_message(tx2.clone(), "PRIVMSG papa : hola".to_string());
//     //
//     // send_message(tx1.clone(), "PRIVMSG pepe : hola papa,soy papa".to_string());
//     //
//     // send_message(tx1.clone(), "QUIT".to_string());
//     // send_message(tx2.clone(), "QUIT".to_string());
//     //
//     // Command::new("sleep").arg("100").spawn().expect("cant wait");
//     //
//     // let mut file_client1 = File::open(&path_to_read).expect("file open error");
//     // let mut buff: [u8; 512] = [b' '; 512];
//     // if let Ok(bytes_read) = file_client1.read(&mut buff) {
//     //     if bytes_read == 0 {
//     //         //error
//     //     }
//     // }
//     // let msg = String::from_utf8_lossy(&buff);
//     // let x = msg.as_ref();
//     // assert!(x.contains("pepe:  hola"));
//     //
//     // let mut file_client2 = File::open(&path_to_read2).expect("file open error");
//     // let mut buff2: [u8; 512] = [b' '; 512];
//     // if let Ok(bytes_read) = file_client2.read(&mut buff2) {
//     //     if bytes_read == 0 {
//     //         //error
//     //     }
//     // }
//     // let msg = String::from_utf8_lossy(&buff2);
//     // let x = msg.as_ref();
//     // assert!(x.contains("papa:  hola papa,soy papa"));
// }

// #[test]
// fn test_two_clients_comunicate_on_channel_on_server() {
//     // let address = format!("127.0.0.1:10000");
//     // let server_name = "s1".to_string();
//     //
//     // let address_share = address.clone();
//     //
//     // thread::Builder::new()
//     //     .name("server".to_string())
//     //     .spawn(move || {
//     //         init_server(address_share, server_name).expect("error al iniciar server");
//     //     })
//     //     .expect("cant create thread server");
//     //
//     // Command::new("sleep").arg("50").spawn().expect("");
//     //
//     // let mut tcp_stream = init_client(address.clone()).expect("cant obtain client tcp");
//     // Command::new("sleep").arg("50").spawn().expect("");
//     // let mut tcp_stream2 = init_client(address.clone()).expect("cant obtain client tcp");
//     // Command::new("sleep").arg("50").spawn().expect("");
//     //
//     // let path_to_read = Path::new("tests/files/client1.txt");
//     // let mut file = File::create(&path_to_read).expect("file open error");
//     // let mut box_file = Box::new(file);
//     //
//     // let (tx1, rx1) = mpsc::channel::<String>();
//     // let tx_clone1 = tx1.clone();
//     // thread::Builder::new()
//     //     .name("client1".to_string())
//     //     .spawn(move || {
//     //         assert!(
//     //             run_client_console(tcp_stream, Box::new(stdin()), box_file, tx_clone1, rx1)
//     //                 .is_err()
//     //         );
//     //     })
//     //     .expect("cant create thread");
//     // Command::new("sleep").arg("50").spawn().expect("");
//     // let path_to_read2 = Path::new("tests/files/client2.txt");
//     // let mut file2 = File::create(&path_to_read2).expect("file open error");
//     // let mut box_file2 = Box::new(file2);
//     //
//     // let (tx2, rx2) = mpsc::channel::<String>();
//     // let tx2_clone = tx2.clone();
//     // thread::Builder::new()
//     //     .name("client2".to_string())
//     //     .spawn(move || {
//     //         assert!(
//     //             run_client_console(tcp_stream2, Box::new(stdin()), box_file2, tx2_clone, rx2)
//     //                 .is_err()
//     //         );
//     //     })
//     //     .expect("cant creat thread");
//     //
//     // Command::new("sleep").arg("50").spawn().expect("");
//     //
//     // send_message(tx1.clone(), "USER x x x x".to_string());
//     // send_message(tx1.clone(), "NICK papa".to_string());
//     // send_message(tx1.clone(), "JOIN #ch1".to_string());
//     //
//     // send_message(tx2.clone(), "USER x x x x".to_string());
//     // send_message(tx2.clone(), "NICK pepe".to_string());
//     // send_message(tx2.clone(), "JOIN #ch1".to_string());
//     //
//     // Command::new("sleep").arg("100").spawn().expect("cant wait");
//     //
//     // send_message(tx1.clone(), " PRIVMSG #ch1 : hola ch2".to_string());
//     // send_message(tx2.clone(), " PRIVMSG #ch1 : hola ch2".to_string());
//     //
//     // Command::new("sleep")
//     //     .arg("100")
//     //     .spawn()
//     //     .expect("cant wait ");
//     // send_message(tx1.clone(), "QUIT".to_string());
//     // send_message(tx2.clone(), "QUIT".to_string());
//     //
//     // Command::new("sleep").arg("100").spawn().expect("cant wait");

//     // let mut file_client1 = File::open(&path_to_read).expect("file open error");
//     // let mut buff: [u8; 512] = [b' '; 512];
//     // if let Ok(bytes_read) = file_client1.read(&mut buff) {
//     //     if bytes_read == 0 {
//     //         //error
//     //     }
//     // }
//     // let msg = String::from_utf8_lossy(&buff);
//     // let x = msg.as_ref();
//     // assert!(x.contains("pepe:  hola"));
//     //
//     // let mut file_client2 = File::open(&path_to_read2).expect("file open error");
//     // let mut buff2: [u8; 512] = [b' '; 512];
//     // if let Ok(bytes_read) = file_client2.read(&mut buff2) {
//     //     if bytes_read == 0 {
//     //         //error
//     //     }
//     // }
//     // let msg = String::from_utf8_lossy(&buff2);
//     // let x = msg.as_ref();
//     // assert!(x.contains("papa:  hola papa,soy papa"));
//     assert!(true);
// }

use std::{
    fs::{self, File},
    io::{Cursor, Read, Result, Write},
    str::FromStr,
    sync::mpsc,
    thread,
    time::Duration,
};

use irc_project::{
    dcc::{
        command::chat::DccChat,
        dcc_connection::{DirectMessage, Stream},
        dcc_handler::DccHandler,
    },
    parser::dcc_message::DccMessage,
};

#[derive(Clone)]
struct MockTcpStream {
    inner: Cursor<Vec<u8>>,
}

impl MockTcpStream {
    fn new(data: &[u8]) -> Self {
        MockTcpStream {
            inner: Cursor::new(data.to_vec()),
        }
    }
}

impl Stream for MockTcpStream {
    fn try_clone(&self) -> Result<Self> {
        Ok(self.clone())
    }
}

impl Write for MockTcpStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.inner.write_all(buf)
    }
}

impl Read for MockTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }
}

struct TempFile {
    path: String,
}

impl TempFile {
    fn new(data: &[u8]) -> TempFile {
        let path = "test_file.txt".to_string();
        {
            let mut file = File::create(&path).expect("Failed to create temp file");
            file.write_all(data).expect("Failed to write to file");
        }
        TempFile { path }
    }
    fn create(data: &[u8]) -> TempFile {
        let path = "test_file2.txt".to_string();
        {
            let mut file = File::create(&path).expect("Failed to create temp file");
            file.write_all(data).expect("Failed to write to file");
        }
        TempFile { path }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if let Err(err) = fs::remove_file(&self.path) {
            eprintln!("Failed to remove temp file: {}", err);
        }
    }
}

#[test]
fn dcc_connection_sending_text_only() {
    let rs_clone = MockTcpStream::new(&[]);
    let (tx_dcc, rx_dcc): (mpsc::Sender<DirectMessage>, mpsc::Receiver<DirectMessage>) =
        mpsc::channel();
    let mut dcc_handler_sender = DccHandler::new(rs_clone.clone(), tx_dcc.clone());
    let mut dcc_handler_receiver = DccHandler::new(rs_clone.clone(), tx_dcc.clone());

    let msg = DccMessage::from_str(":pepe privmsg lucas dcc chat chat localhost 8081").unwrap();
    assert!(DccChat::new(msg.clone()).is_ok());
    assert!(dcc_handler_sender
        .handle_dcc_message_send(":pepe privmsg lucas dcc chat chat localhost 8081".to_string())
        .is_ok());
    assert!(dcc_handler_receiver
        .handle_dcc_message_reception(msg)
        .is_ok());
    thread::sleep(Duration::from_millis(100));
    dcc_handler_sender
        .handle_dcc_message_send(":localhost:8081 hello world //END".to_string())
        .unwrap();
    let result = rx_dcc.recv().unwrap();
    let expected = DirectMessage(
        "localhost:8081".to_owned(),
        "pepe".to_owned(),
        "hello world".to_owned(),
    );
    assert_eq!(result, expected);
    dcc_handler_receiver
        .handle_dcc_message_send(":localhost:8081 hello world //END".to_string())
        .unwrap();
    let result = rx_dcc.recv().unwrap();
    let expected = DirectMessage(
        "localhost:8081".to_owned(),
        "lucas".to_owned(),
        "hello world".to_owned(),
    );
    assert_eq!(result, expected);

    //close connection and expect response as an error
    dcc_handler_sender
        .handle_dcc_message_send("DCC CLOSE localhost 8081".to_owned())
        .unwrap();
    thread::sleep(Duration::from_millis(100));
    assert!(dcc_handler_receiver
        .handle_dcc_message_send(":localhost:8081 hello world //END".to_string())
        .is_err());

    //abro una conexion nueva
    let msg = DccMessage::from_str(":pepe privmsg lucas dcc chat chat localhost 8081").unwrap();
    assert!(DccChat::new(msg.clone()).is_ok());
    assert!(dcc_handler_sender
        .handle_dcc_message_send(":pepe privmsg lucas dcc chat chat localhost 8081".to_string())
        .is_ok());
    assert!(dcc_handler_receiver
        .handle_dcc_message_reception(msg)
        .is_ok());
    thread::sleep(Duration::from_millis(100));
    dcc_handler_sender
        .handle_dcc_message_send(":localhost:8081 hello world //END".to_string())
        .unwrap();
    let result = rx_dcc.recv().unwrap();
    let expected = DirectMessage(
        "localhost:8081".to_owned(),
        "pepe".to_owned(),
        "hello world".to_owned(),
    );
    assert_eq!(result, expected);
    dcc_handler_receiver
        .handle_dcc_message_send(":localhost:8081 hello world //END".to_string())
        .unwrap();
    let result = rx_dcc.recv().unwrap();
    let expected = DirectMessage(
        "localhost:8081".to_owned(),
        "lucas".to_owned(),
        "hello world".to_owned(),
    );
    assert_eq!(result, expected);
}

#[test]
fn dcc_connection_sending_files_only() {
    let _file = TempFile::new(&[]);
    let rs_clone = MockTcpStream::new(&[]);
    let (tx_dcc, _rx_dcc): (mpsc::Sender<DirectMessage>, mpsc::Receiver<DirectMessage>) =
        mpsc::channel();
    let mut dcc_handler_sender = DccHandler::new(rs_clone.clone(), tx_dcc.clone());
    let mut dcc_handler_receiver = DccHandler::new(rs_clone.clone(), tx_dcc.clone());
    let msg = DccMessage::from_str(":pepe privmsg lucas dcc send test_file.txt localhost 8082 123 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap();
    assert!(dcc_handler_sender
        .handle_dcc_message_send(
            ":pepe privmsg lucas dcc send test_file.txt localhost 8082".to_owned()
        )
        .is_ok());
    thread::sleep(Duration::from_millis(150));
    dcc_handler_receiver
        .handle_dcc_message_reception(msg)
        .unwrap();
    thread::sleep(Duration::from_millis(150));

    //sends the other way around
    let msg = DccMessage::from_str(":pepe privmsg lucas dcc send test_file.txt localhost 8082 123 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap();
    assert!(dcc_handler_receiver
        .handle_dcc_message_send(
            ":pepe privmsg lucas dcc send test_file.txt localhost 8082".to_owned()
        )
        .is_ok());
    thread::sleep(Duration::from_millis(150));
    dcc_handler_sender
        .handle_dcc_message_reception(msg)
        .unwrap();
    thread::sleep(Duration::from_millis(150));
}

#[test]
fn dcc_connection_sending_files_and_text() {
    let rs_clone = MockTcpStream::new(&[]);
    let (tx_dcc, rx_dcc): (mpsc::Sender<DirectMessage>, mpsc::Receiver<DirectMessage>) =
        mpsc::channel();
    let mut dcc_handler_sender = DccHandler::new(rs_clone.clone(), tx_dcc.clone());
    let mut dcc_handler_receiver = DccHandler::new(rs_clone.clone(), tx_dcc.clone());

    let msg = DccMessage::from_str(":pepe privmsg lucas dcc chat chat localhost 8080").unwrap();
    assert!(DccChat::new(msg.clone()).is_ok());
    assert!(dcc_handler_sender
        .handle_dcc_message_send(":pepe privmsg lucas dcc chat chat localhost 8080".to_string())
        .is_ok());
    assert!(dcc_handler_receiver
        .handle_dcc_message_reception(msg)
        .is_ok());
    thread::sleep(Duration::from_millis(100));
    dcc_handler_sender
        .handle_dcc_message_send(":localhost:8080 hello world //END".to_string())
        .unwrap();
    let result = rx_dcc.recv().unwrap();
    let expected = DirectMessage(
        "localhost:8080".to_owned(),
        "pepe".to_owned(),
        "hello world".to_owned(),
    );
    assert_eq!(result, expected);
    dcc_handler_receiver
        .handle_dcc_message_send(":localhost:8080 hello world //END".to_string())
        .unwrap();
    let result = rx_dcc.recv().unwrap();
    let expected = DirectMessage(
        "localhost:8080".to_owned(),
        "lucas".to_owned(),
        "hello world".to_owned(),
    );
    assert_eq!(result, expected);

    //close connection and expect response as an error
    dcc_handler_sender
        .handle_dcc_message_send("DCC CLOSE localhost 8080".to_owned())
        .unwrap();
    thread::sleep(Duration::from_millis(100));
    assert!(dcc_handler_receiver
        .handle_dcc_message_send(":localhost:8080 hello world //END".to_string())
        .is_err());

    //abro una conexion nueva
    let msg = DccMessage::from_str(":pepe privmsg lucas dcc chat chat localhost 8080").unwrap();
    assert!(DccChat::new(msg.clone()).is_ok());
    assert!(dcc_handler_sender
        .handle_dcc_message_send(":pepe privmsg lucas dcc chat chat localhost 8080".to_string())
        .is_ok());
    assert!(dcc_handler_receiver
        .handle_dcc_message_reception(msg)
        .is_ok());
    thread::sleep(Duration::from_millis(100));
    dcc_handler_sender
        .handle_dcc_message_send(":localhost:8080 hello world //END".to_string())
        .unwrap();
    let result = rx_dcc.recv().unwrap();
    let expected = DirectMessage(
        "localhost:8080".to_owned(),
        "pepe".to_owned(),
        "hello world".to_owned(),
    );
    assert_eq!(result, expected);
    dcc_handler_receiver
        .handle_dcc_message_send(":localhost:8080 hello world //END".to_string())
        .unwrap();
    let result = rx_dcc.recv().unwrap();
    let expected = DirectMessage(
        "localhost:8080".to_owned(),
        "lucas".to_owned(),
        "hello world".to_owned(),
    );
    assert_eq!(result, expected);

    dcc_handler_sender
        .handle_dcc_message_send("DCC CLOSE localhost 8080".to_owned())
        .unwrap();
    thread::sleep(Duration::from_millis(100));
    assert!(dcc_handler_receiver
        .handle_dcc_message_send(":localhost:8080 hello world //END".to_string())
        .is_err());

    let _file = TempFile::create(&[]);
    let msg = DccMessage::from_str(":pepe privmsg lucas dcc send test_file.txt localhost 8080 123 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap();
    assert!(dcc_handler_sender
        .handle_dcc_message_send(
            ":pepe privmsg lucas dcc send test_file.txt localhost 8080".to_owned()
        )
        .is_ok());
    thread::sleep(Duration::from_millis(150));
    dcc_handler_receiver
        .handle_dcc_message_reception(msg)
        .unwrap();
    thread::sleep(Duration::from_millis(150));

    //sends the other way around
    let msg = DccMessage::from_str(":pepe privmsg lucas dcc send test_file.txt localhost 8080 123 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap();
    assert!(dcc_handler_receiver
        .handle_dcc_message_send(
            ":pepe privmsg lucas dcc send test_file.txt localhost 8080".to_owned()
        )
        .is_ok());
    thread::sleep(Duration::from_millis(150));
    dcc_handler_sender
        .handle_dcc_message_reception(msg)
        .unwrap();
    thread::sleep(Duration::from_millis(150));
}
