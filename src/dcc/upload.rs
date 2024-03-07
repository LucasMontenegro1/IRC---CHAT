use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    sync::{Arc, Mutex},
};

use crate::error::error_server::ErrorServer;

use super::{cryptography, dcc_connection::Stream};

/// Represents an upload instance for sending data through a stream.
#[derive(Debug)]
pub struct Upload<T>
where
    T: Stream,
{
    /// Unique identifier for the upload instance.
    id: String,
    /// Result of the stream, either containing the stream or an error.
    stream: Result<T, ErrorServer>,
    /// Name of the file being uploaded.
    filename: String,
    /// Mutex for controlling the pause state of the upload.
    pause: Arc<Mutex<bool>>,
}

impl<T: Stream> Upload<T> {
    /// Creates a new upload instance with the provided parameters.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the upload instance.
    /// * `stream` - The stream for sending data.
    /// * `filename` - Name of the file being uploaded.
    pub fn new(id: String, stream: T, filename: String) -> Upload<T> {
        Self {
            //user,
            stream: Ok(stream),
            filename,
            pause: Arc::new(Mutex::new(false)),
            id,
        }
    }

    /// Starts the upload process from a specified offset.
    ///
    /// # Arguments
    ///
    /// * `offset` - The starting offset for the upload.
    pub fn start(&mut self, offset: u64) -> Result<(), ErrorServer> {
        let paused = self.pause.clone();
        let mut stream = self.see_if_clonable()?;
        let path = Path::new(&self.filename);
        let mut file = File::open(path).unwrap();

        file.seek(SeekFrom::Start(offset))?;

        let mut buffer = [0; 1024];
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                return Ok(());
            }
            match paused.lock() {
                Ok(c) => {
                    if *c {
                        println!("Descarga pausada");
                        return Ok(());
                    }
                }
                Err(_) => return Err(ErrorServer::LockedResource),
            }
            let result = cryptography::encrypt_chunk(&buffer[0..bytes_read], cryptography::KEY)?;
            stream.write_all(&result)?;
        }
    }

    /// Pauses the upload process.
    pub fn pause(&self) -> Result<(), ErrorServer> {
        match self.pause.lock() {
            Ok(mut c) => {
                *c = true;
                Ok(())
            }
            Err(_) => Err(ErrorServer::LockedResource),
        }
    }

    /// Sets the upload to resume from a paused state.
    pub fn set_resume(&self) -> Result<(), ErrorServer> {
        match self.pause.lock() {
            Ok(mut c) => {
                *c = false;
                Ok(())
            }
            Err(_) => Err(ErrorServer::LockedResource),
        }
    }

    /// Resumes the upload process from a specified offset.
    ///
    /// # Arguments
    ///
    /// * `offset` - The starting offset for the upload.
    pub fn resume(&mut self, offset: u64) -> Result<(), ErrorServer> {
        self.start(offset)
    }

    /// Checks if the upload is currently paused.
    pub fn is_paused(&self) -> Result<bool, ErrorServer> {
        match self.pause.lock() {
            Ok(c) => {
                if *c {
                    return Ok(true);
                }
                Ok(false)
            }
            Err(_) => Err(ErrorServer::LockedResource),
        }
    }

    /// Clones the stream for clonable streams.
    fn see_if_clonable(&self) -> Result<T, ErrorServer> {
        match &self.stream {
            Ok(c) => match c.try_clone() {
                Ok(c) => Ok(c),
                Err(_) => Err(ErrorServer::TcpFail),
            },
            Err(_) => Err(ErrorServer::TcpFail),
        }
    }

    /// Gets the unique identifier of the upload.
    pub fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl<T: Stream> Clone for Upload<T> {
    fn clone(&self) -> Self {
        Self {
            //user: self.user.clone(),
            stream: self.see_if_clonable(),
            filename: self.filename.clone(),
            pause: self.pause.clone(),
            id: self.id.clone(),
        }
    }
}

impl<T: Stream> Write for Upload<T> {
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

impl<T: Stream> Read for Upload<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.stream {
            Ok(s) => s.read(buf),
            Err(_) => Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{Cursor, Read, Result, Write};

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

    // Estructura para crear y eliminar un archivo temporal en el ámbito
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
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            if let Err(err) = fs::remove_file(&self.path) {
                eprintln!("Failed to remove temp file: {}", err);
            }
        }
    }

    #[test]
    fn test_upload_start() {
        // Crear un archivo temporal para la prueba
        let temp_file_path = "test_file.txt";
        match File::create(temp_file_path) {
            Ok(mut file) => {
                // Escribir algunos datos en el archivo si es necesario
                file.write_all(b"Hello, world!")
                    .expect("Failed to write to file");

                // Mock Upload with a MockStream
                let mut mock_upload = Upload::new(
                    "test_id".to_string(),
                    MockTcpStream::new(&[]),
                    temp_file_path.to_string(),
                );

                // Test the start method
                let result = mock_upload.start(0);
                assert!(result.is_ok());
            }
            Err(e) => {
                eprintln!("Failed to create temp file: {}", e);
                panic!()
                // Mark the test as failed
            }
        }

        // Attempt to remove the temp file
        if let Err(e) = fs::remove_file(temp_file_path) {
            eprintln!("Failed to remove temp file: {}", e);
        }
    }

    #[test]
    fn test_upload_pause_resume() {
        // Crear un archivo temporal para la prueba
        let _temp_file = TempFile::new(&[]);

        // Mock Upload with a MockStream
        let mock_upload = Upload::new(
            "test_id".to_string(),
            MockTcpStream::new(&[]),
            "test_file.txt".to_string(),
        );

        // Test pause method
        let result = mock_upload.pause();
        assert!(result.is_ok());

        // Test set_resume method
        let result = mock_upload.set_resume();
        assert!(result.is_ok());
    }

    #[test]
    fn test_upload_resume() {
        // Crear un archivo temporal para la prueba
        let _temp_file = TempFile::new(&[]);

        // Mock Upload with a MockStream
        let mut mock_upload = Upload::new(
            "test_id".to_string(),
            MockTcpStream::new(&[]),
            "test_file.txt".to_string(),
        );

        // Test resume method
        let result = mock_upload.resume(0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_upload_is_paused() {
        // Crear un archivo temporal para la prueba
        let _temp_file = TempFile::new(&[]);

        // Mock Upload with a MockStream
        let mock_upload = Upload::new(
            "test_id".to_string(),
            MockTcpStream::new(&[]),
            "test_file.txt".to_string(),
        );

        // Test is_paused method
        let result = mock_upload.is_paused();
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Since we didn't pause it, it should be false
    }
}
