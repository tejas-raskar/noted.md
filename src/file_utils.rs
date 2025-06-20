use base64::{Engine, engine::general_purpose};
use std::{fs, path::Path};

pub struct FileData {
    pub encoded_data: String,
    pub mime_type: String,
}

pub fn process_file(file_path: &str) -> Result<FileData, std::io::Error> {
    let data = fs::read(file_path)?;
    let encoded_data: String = general_purpose::STANDARD.encode(&data);
    let mime_type = get_file_mime_type(file_path);

    Ok(FileData {
        encoded_data: encoded_data,
        mime_type: mime_type,
    })
}

fn get_file_mime_type(file_path: &str) -> String {
    let file_extension = Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str());
    let mime_type = match file_extension {
        Some("png") => "image/png".to_string(),
        Some("pdf") => "application/pdf".to_string(),
        Some("jpg") => "image/jpeg".to_string(),
        Some("jpeg") => "image/jpeg".to_string(),
        _ => {
            println!("Warning: Unknown file type, defaulting to application/octet-stream");
            "application/octet-stream".to_string()
        }
    };

    mime_type
}
