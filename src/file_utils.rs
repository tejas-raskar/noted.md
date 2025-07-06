use crate::error::NotedError;
use base64::{Engine, engine::general_purpose};
use std::{fs, path::Path};

pub struct FileData {
    pub encoded_data: String,
    pub mime_type: String,
}

pub fn process_file(file_path: &str) -> Result<FileData, NotedError> {
    let data = fs::read(file_path)?;
    let encoded_data: String = general_purpose::STANDARD.encode(&data);
    let mime_type = get_file_mime_type(file_path)?;

    Ok(FileData {
        encoded_data,
        mime_type,
    })
}

pub fn get_file_mime_type(file_path: &str) -> Result<String, NotedError> {
    let file_extension = Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str());

    match file_extension {
        Some("png") => Ok("image/png".to_string()),
        Some("pdf") => Ok("application/pdf".to_string()),
        Some("jpg") => Ok("image/jpeg".to_string()),
        Some("jpeg") => Ok("image/jpeg".to_string()),
        Some(ext) => Err(NotedError::UnsupportedFileType(ext.to_string())),
        None => Err(NotedError::UnsupportedFileType("No extension".to_string())),
    }
}
