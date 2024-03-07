use crate::dcc::dcc_connection::DirectMessage;
use crate::dcc::dcc_handler::DccHandler;
use crate::error::error_client::ErrorClient;
use crate::parser::dcc_message::DccMessage;
use crate::reply::{reply_maker, Reply};
use std::io::{BufRead, BufReader};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::FromStr;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

const WHITESPACE_U8: u8 = b' ';

pub fn run_client_console(
    read_stream: Arc<TcpStream>,
    buf: Box<dyn Read + Send>,
    buff_out: Box<dyn Write + Send>,
    tx: Sender<String>,
    rx: Receiver<String>,
) -> Result<(), ErrorClient> {
    let rs_clone = read_stream.clone().try_clone()?;
    let (tx_dcc, rx_dcc): (mpsc::Sender<DirectMessage>, mpsc::Receiver<DirectMessage>) =
        mpsc::channel();
    let dcc_handler = DccHandler::new(rs_clone, tx_dcc.clone());
    let dcc_handler_2 = dcc_handler.clone();

    //recepcion de dcc
    let th_dcc = thread::spawn(move || -> Result<(), ErrorClient> {
        loop {
            if let Ok(message) = rx_dcc.try_recv() {
                println!("{:?}", message);
            }
        }
    });

    // recepcion de msjs
    let rs1 = Arc::clone(&read_stream);
    let th1 = thread::spawn(move || -> Result<(), ErrorClient> {
        handle_message_reception(&rs1, buff_out, dcc_handler_2.clone())
    });

    // transmision de msj
    let rs2 = Arc::clone(&read_stream);
    let th2 = thread::spawn(move || -> Result<(), ErrorClient> {
        handle_message_trasmission(&rs2, rx, dcc_handler.clone())
    });

    let transmitter = tx;
    let th3 = thread::spawn(move || -> Result<(), ErrorClient> {
        read_message(&mut BufReader::new(buf), transmitter)
    });

    run_cli_app(th1, th2, th3, th_dcc)
}

fn run_cli_app(
    th1: JoinHandle<Result<(), ErrorClient>>,
    th2: JoinHandle<Result<(), ErrorClient>>,
    th3: JoinHandle<Result<(), ErrorClient>>,
    th_dcc: JoinHandle<Result<(), ErrorClient>>,
) -> Result<(), ErrorClient> {
    while !th1.is_finished() & !th2.is_finished() & !th3.is_finished() & !th_dcc.is_finished() {}
    handle_thread(th1)?;
    handle_thread(th2)?;
    handle_thread(th3)?;
    handle_thread(th_dcc)?;

    Ok(())
}

fn read_message(
    reader: &mut BufReader<Box<dyn Read>>,
    tx: Sender<String>,
) -> Result<(), ErrorClient> {
    //Looks for messages written to stdin. Those are in turn sent to the client's thread to manage.
    while let Some(Ok(s)) = reader.lines().next() {
        let buff = s.clone();
        tx.send(buff.clone())?;
        if buff.trim().len() >= 4 && &buff.trim()[..4].to_uppercase() == "QUIT" {
            return Ok(());
        }
    }
    Ok(())
}

fn handle_thread(thread: JoinHandle<Result<(), ErrorClient>>) -> Result<(), ErrorClient> {
    if thread.is_finished() {
        if let Ok(r) = thread.join() {
            return r;
        };
    }
    Ok(())
}

fn handle_message_reception(
    server_stream: &TcpStream,
    buff_out: Box<dyn Write + Send>,
    mut dcc_handler: DccHandler<TcpStream>,
) -> Result<(), ErrorClient> {
    let mut read_stream = server_stream.try_clone()?;
    loop {
        let mut buff: [u8; 512] = [WHITESPACE_U8; 512];
        if let Ok(bytes_read) = read_stream.read(&mut buff) {
            //Reads messages stored in the server-side socket.
            if bytes_read == 0 {
                return Err(ErrorClient::ServerClosed);
            }
            let msg = String::from_utf8_lossy(&buff);
            let x = msg.as_ref();

            if let Ok(c) = DccMessage::from_str(x) {
                println!("DCC MESSAGE: {:?}", c);
                match dcc_handler.handle_dcc_message_reception(c) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error DCC MESSAGE: {:?}", e)
                    }
                }
                continue;
            }

            match Reply::from_str(x) {
                Ok(c) => {
                    // let x = format!("REPLY: \n{}\n", reply_maker::make_reply_format(c));
                    // if buff_out.write(x.trim().as_bytes()).is_ok() {
                    //     buff_out.flush()?;
                    // };

                    println!("REPLY: \n{}", reply_maker::make_reply_format(c));
                }
                Err(_) => {
                    // if buff_out
                    //     .write(format!("MSG: \n{}\n", msg.trim()).as_bytes())
                    //     .is_err()
                    // {
                    //     buff_out.flush()?;
                    //     println!("ERROR");
                    // }
                    println!("MSG: \n{}", msg.trim());
                    if msg.trim().len() >= 4 && &msg.trim()[..4].to_uppercase() == "QUIT" {
                        drop(buff_out);
                        return Ok(());
                    }
                }
            }
        };
    }
}

fn handle_message_trasmission(
    server_stream: &TcpStream,
    rx: Receiver<String>,
    mut dcc_handler: DccHandler<TcpStream>,
) -> Result<(), ErrorClient> {
    loop {
        let mut write_stream = server_stream.try_clone()?;
        let mut is_dcc = false;
        if let Ok(mut buff) = rx.try_recv() {
            buff = buff.trim().to_string();
            let dcc = dcc_handler.handle_dcc_message_send(buff.clone());
            match dcc {
                Ok(_) => is_dcc = true,
                Err(e) => match e {
                    crate::error::error_msg::ErrorMsg::EmptyMsg => {
                        println!("error en el mensaje dcc")
                    }
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
                        is_dcc = true;
                        println!("Server error dcc :{}", e)
                    }
                },
            }

            if !is_dcc {
                let _bytes_written = write_stream.write(buff.as_bytes());
            }
        }
    }
}

// fn end_irc_connection(tx: &Sender<String>) -> Result<(), ErrorClient> {
//     tx.send("QUIT :Closing Desktop App".to_string())?;
//     Ok(())
// }
