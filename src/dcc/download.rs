use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use sha::{sha256::Sha256, utils::DigestExt};

use crate::{dcc::cryptography, error::error_server::ErrorServer};

use super::dcc_connection::Stream;

/// Represents a download instance for retrieving data from a stream.
#[derive(Debug)]
pub struct Download<T>
where
    T: Stream,
{
    /// Unique identifier for the download instance.
    id: String,
    /// Result of the stream, either containing the stream or an error.
    stream: Result<T, ErrorServer>,
    /// Name of the file being downloaded.
    filename: String,
    /// Size of the file being downloaded.
    size: usize,
    /// Shared counter for tracking the total bytes read during the download.
    total_bytes_read: Arc<Mutex<usize>>,
    /// SHA-256 hash of the file being downloaded.
    hash: String,
}

impl<T: Stream> Download<T> {
    /// Constructs a new `Download` instance.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the download.
    /// * `stream` - The stream from which the file is being downloaded.
    /// * `filename` - The name of the file being downloaded.
    /// * `size` - The size of the file being downloaded.
    /// * `hash` - The SHA-256 hash of the file being downloaded.
    pub fn new(id: String, stream: T, filename: String, size: usize, hash: String) -> Download<T> {
        Self {
            stream: Ok(stream),
            filename,
            id,
            size,
            total_bytes_read: Arc::new(Mutex::new(0)),
            hash,
        }
    }

    /// Initiates the download operation from the stream to a file.
    ///
    /// This function downloads the file from the stream to the local file system.
    ///
    /// # Errors
    ///
    /// Returns an `ErrorServer` if any error occurs during the download process.
    pub fn download_file(&mut self) -> Result<(), ErrorServer> {
        let home_dir = match std::env::var("HOME") {
            Ok(val) => PathBuf::from(val),
            Err(_) => {
                // Si no se puede obtener la variable HOME, utiliza el directorio actual como alternativa
                PathBuf::from(".")
            }
        };

        let dcc_downloads_dir = home_dir.join("dcc_downloads");
        if !dcc_downloads_dir.exists() && fs::create_dir_all(&dcc_downloads_dir).is_err() {
            println!("Error creating directory");
            return Err(ErrorServer::BadQuery);
        }
        let file_path = dcc_downloads_dir.join(self.filename.clone());
        let mut file = File::create(&file_path)?;

        let mut buffer = [0; 1024];
        while *self.total_bytes_read.lock().unwrap() < self.size {
            let bytes_read = self.read(&mut buffer)?;
            let result = cryptography::decrypt_chunk(&buffer[0..bytes_read], cryptography::KEY)?;
            // println!("{:?}", &result[0..bytes_read]);
            // println!("--------");
            //let result = cryptography::decrypt_chunk(&buffer[0..bytes_read], b"secret")?;
            //println!("{}", String::from_utf8_lossy(&result).to_string());

            if bytes_read == 0 {
                // Se alcanzó el final del stream antes de alcanzar el tamaño máximo
                break;
            }

            let bytes_to_write = std::cmp::min(
                bytes_read,
                self.size - *self.total_bytes_read.lock().unwrap(),
            );
            file.write_all(&result)?;
            let mut total_bytes_read = self.total_bytes_read.lock().unwrap();
            *total_bytes_read += bytes_to_write;
        }
        let mut file = File::open(&file_path)?;
        let mut hasher = Sha256::default();
        io::copy(&mut file, &mut hasher)?;
        hasher.flush()?;
        let bytes = hasher.to_hex();
        println!("{}", bytes);
        if self.hash == bytes {
            println!("Control de integridad correcto")
        } else {
            println!("Control de integridad incorrecto")
        }

        Ok(())
    }

    /// Checks if the stream is clonable and attempts to clone it for concurrent operations.
    fn see_if_clonable(&self) -> Result<T, ErrorServer> {
        match &self.stream {
            Ok(c) => match c.try_clone() {
                Ok(c) => Ok(c),
                Err(_) => Err(ErrorServer::TcpFail),
            },
            Err(_) => Err(ErrorServer::TcpFail),
        }
    }

    /// Retrieves the unique identifier of the download.
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    /// Retrieves the total bytes read during the download operation.
    pub fn total_bytes_read(&self) -> usize {
        *self.total_bytes_read.lock().unwrap()
    }
}

impl<T: Stream> Clone for Download<T> {
    fn clone(&self) -> Self {
        Self {
            stream: self.see_if_clonable(),
            filename: self.filename.clone(),
            id: self.id.clone(),
            size: self.size,
            total_bytes_read: self.total_bytes_read.clone(),
            hash: self.hash.clone(),
        }
    }
}

impl<T: Stream> Write for Download<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.stream {
            Ok(s) => s.write(buf),
            Err(_) => Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.stream {
            Ok(s) => s.flush(),
            Err(_) => Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match &mut self.stream {
            Ok(s) => s.write_all(buf),
            Err(_) => Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
        }
    }
}

impl<T: Stream> Read for Download<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.stream {
            Ok(s) => s.read(buf),
            Err(_) => Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dcc::command::send::DccSend;
    use crate::parser::dcc_message::DccMessage;

    use super::*;
    use std::io::{Cursor, Read, Result, Write};
    use std::str::FromStr;

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

    #[test]
    fn test_download_file() {
        // Create a sample DccSend
        let dcc_send = DccSend {
            id: "test_id".to_string(),
            ip: "127.0.0.1".to_string(),
            port: "12345".to_string(),
            filename: "test_file.txt".to_string(),
            zip: false,
            msg: DccMessage::from_str("privmsg user DCC SEND test_file.txt 127.0.0.1 12345")
                .unwrap(),
        };

        // Create a sample Download
        let mut download = Download::new(
            dcc_send.id.clone(),
            MockTcpStream::new(&[1, 2, 3, 4, 5]),
            dcc_send.filename.clone(),
            5,
            "9000609720e2990dd4241a2199a5a63eed186dc10ef6f71ba8cf9f87621e6478".to_string(),
        );

        // Test download_file
        let result = download.download_file();
        assert!(result.is_ok());
        assert_eq!(download.total_bytes_read(), 5);
    }
}
