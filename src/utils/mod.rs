use crate::error::error_server::ErrorServer;

use std::io::Read;
use std::io::Write;
use std::thread;
use std::time::Duration;

const MESSAGE_LENGHT: usize = 512;

pub fn read_message_from(client: &mut dyn Read) -> Result<String, ErrorServer> {
    // whitespaces are removed by the parser.
    let mut buff: [u8; 512] = [b' '; MESSAGE_LENGHT];
    //ADD: When it fails remove socket from clients?.
    let bytes_read = client.read(&mut buff)?;
    if bytes_read == 0 {
        Err(ErrorServer::UnreachableClient)
    } else {
        //Ok(n), 0 <= n <= buf.len() it's garanteed.
        let msg = String::from_utf8_lossy(&buff).trim().to_string();
        Ok(msg)
    }
}

pub fn write_message_to(message: &dyn ToString, client: &mut dyn Write) -> Result<(), ErrorServer> {
    let mut buff: [u8; 512] = [b' '; MESSAGE_LENGHT];
    if !message.to_string().is_empty() {
        println!("Sending  \"{}\"  to client", message.to_string());
        buff[..message.to_string().len()].copy_from_slice(message.to_string().as_bytes());
        client.write_all(&buff)?;
    }
    Ok(())
}

pub fn write_messages_to(
    messages: &mut dyn Iterator<Item = &String>,
    client: &mut dyn Write,
) -> Result<(), ErrorServer> {
    //Manda el mensaje para darse conocer a quien se hizo conocer.
    //println!("----------------------------------------------");
    //println!("START SENDING MESSAGES: write_messages_to()");
    messages.for_each(|msg| {
        thread::sleep(Duration::from_millis(500));
        //println!("send_info(): {}", msg);
        if let Err(err) = write_message_to(&msg, client) {
            println!("cant send info {}: {:?}", msg, err);
        }
    });
    //println!("END SENDING MESSAGES: write_messages_to()");
    //println!("----------------------------------------------");
    Ok(())
}
