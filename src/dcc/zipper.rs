use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use zip::write::FileOptions;

/// Compresses a file into a ZIP archive.
///
/// This function takes the path to an input file, creates a ZIP archive containing
/// the file, and returns the path to the resulting ZIP archive. The compression
/// method used is Deflated.
///
/// # Arguments
///
/// * `input_path` - The path to the input file.
///
/// # Returns
///
/// Returns a `Result` containing the path to the created ZIP archive or an `io::Error`.
pub fn file_to_zip(input_path: &str) -> Result<PathBuf, io::Error> {
    let dcc_uploads_dir = match std::env::var("HOME") {
        Ok(val) => PathBuf::from(val).join("dcc_uploads"),
        Err(_) => PathBuf::from(".").join("dcc_uploads"),
    };

    if !dcc_uploads_dir.exists() && fs::create_dir_all(&dcc_uploads_dir).is_err() {
        eprintln!("Error creating directory");
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Error creating directory for uploads",
        ));
    }

    let file_name = Path::new(input_path)
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Couldn't find specified file."))?;
    let output_path = dcc_uploads_dir.join(file_name).with_extension("zip");

    let input_file = File::open(input_path)?;

    let output_file = File::create(&output_path)?;

    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut zip_writer = zip::ZipWriter::new(output_file);

    zip_writer.start_file(file_name.to_str().unwrap(), options)?;

    let mut buffer = Vec::new();
    let mut input_reader = io::BufReader::new(input_file);
    input_reader.read_to_end(&mut buffer)?;

    zip_writer.write_all(&buffer)?;

    zip_writer.finish()?;

    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::{Read, Write};
    use std::path::PathBuf;

    fn create_temp_dir() -> PathBuf {
        let temp_dir = std::env::temp_dir().join("test_temp_dir");
        fs::create_dir_all(&temp_dir).expect("Failed to create temporary directory");
        temp_dir
    }

    #[test]
    fn test_file_to_zip() {
        // Crear un directorio temporal para la prueba
        let temp_dir_path = create_temp_dir();

        // Crear un archivo temporal dentro del directorio
        let input_file_path = temp_dir_path.join("test_file.txt");
        let mut input_file = File::create(&input_file_path).expect("Failed to create input file");
        input_file
            .write_all(b"Hello, world!")
            .expect("Failed to write to input file");

        // Llamar a la función file_to_zip
        let result = file_to_zip(input_file_path.to_str().unwrap());

        // Verificar el resultado
        assert!(result.is_ok());
        let zip_file_path = result.unwrap();

        // Verificar si el archivo ZIP existe y tiene contenido
        assert!(zip_file_path.exists());

        let mut zip_file = File::open(&zip_file_path).expect("Failed to open ZIP file");
        let mut zip_contents = Vec::new();
        zip_file
            .read_to_end(&mut zip_contents)
            .expect("Failed to read ZIP file contents");

        assert!(!zip_contents.is_empty());

        // Limpiar: Eliminar el directorio temporal y su contenido
        fs::remove_dir_all(&temp_dir_path).expect("Failed to remove temporary directory");
    }
}
