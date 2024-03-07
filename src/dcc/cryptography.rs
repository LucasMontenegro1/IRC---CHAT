extern crate crypto;

use crypto::buffer::{BufferResult, ReadBuffer, WriteBuffer};
use crypto::chacha20::ChaCha20;
use crypto::symmetriccipher::{Decryptor, Encryptor};

use crate::error::error_server::ErrorServer;

/// Encrypts a chunk of data using ChaCha20 stream cipher with the provided key.
///
/// # Arguments
///
/// * `data` - Data to be encrypted.
/// * `key` - Key used for encryption.
///
/// # Returns
///
/// A `Result` containing the encrypted data or an `ErrorServer` in case of failure.
pub fn encrypt_chunk(data: &[u8], key: &[u8]) -> Result<Vec<u8>, ErrorServer> {
    let mut encryptor = ChaCha20::new(key, &[0; 8]);

    let mut ciphertext = Vec::new();
    let mut buffer = [0; 4096];
    let mut read_buffer = crypto::buffer::RefReadBuffer::new(data);
    let mut write_buffer = crypto::buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = encryptor
            .encrypt(&mut read_buffer, &mut write_buffer, true)
            .map_err(|_| ErrorServer::DCCError)?;

        ciphertext.extend(write_buffer.take_read_buffer().take_remaining().iter());

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(ciphertext)
}

/// Decrypts a chunk of data previously encrypted with ChaCha20 using the given key.
///
/// # Arguments
///
/// * `data` - Data to be decrypted.
/// * `key` - Key used for decryption.
///
/// # Returns
///
/// A `Result` containing the decrypted data or an `ErrorServer` in case of failure.
pub fn decrypt_chunk(data: &[u8], key: &[u8]) -> Result<Vec<u8>, ErrorServer> {
    let mut decryptor = ChaCha20::new(key, &[0; 8]);

    let mut plaintext = Vec::new();
    let mut buffer = [0; 4096];
    let mut read_buffer = crypto::buffer::RefReadBuffer::new(data);
    let mut write_buffer = crypto::buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = decryptor
            .decrypt(&mut read_buffer, &mut write_buffer, true)
            .map_err(|_| ErrorServer::DCCError)?;

        plaintext.extend(write_buffer.take_read_buffer().take_remaining().iter());

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(plaintext)
}

pub const KEY: &[u8] = b"0123456789ABCDEF";

#[cfg(test)]
mod tests {
    use super::*;

    // Clave fija para propósito de prueba
    const TEST_KEY: &[u8] = b"0123456789ABCDEF";

    #[test]
    fn test_encryption_decryption() {
        // Datos de prueba
        let data = b"Hello, worldsafnlsdgsdopgsfogsfgdfkgnfdk!";

        // Encriptar los datos
        let encrypted_data = encrypt_chunk(data, TEST_KEY).unwrap();

        // Desencriptar los datos
        let decrypted_data = decrypt_chunk(&encrypted_data, TEST_KEY).unwrap();

        // Verificar que los datos desencriptados coincidan con los originales
        assert_eq!(decrypted_data, data);
    }

    #[test]
    fn test_empty_data() {
        // Encriptar y desencriptar datos vacíos
        let encrypted_data = encrypt_chunk(b"", TEST_KEY).unwrap();
        let decrypted_data = decrypt_chunk(&encrypted_data, TEST_KEY).unwrap();

        // Verificar que los datos desencriptados coincidan con los originales
        assert_eq!(decrypted_data, b"");
    }

    #[test]
    fn test_long_data() {
        // Datos de prueba más largos
        let long_data = vec![0u8; 1024 * 1024]; // 1 MB de datos

        // Encriptar los datos largos
        let encrypted_data = encrypt_chunk(&long_data, TEST_KEY).unwrap();

        // Desencriptar los datos largos
        let decrypted_data = decrypt_chunk(&encrypted_data, TEST_KEY).unwrap();

        // Verificar que los datos desencriptados coincidan con los originales
        assert_eq!(decrypted_data, long_data);
    }

    // Agrega más tests según los casos de uso específicos que desees cubrir
}
