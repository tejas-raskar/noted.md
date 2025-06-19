use std::{fs, path::Path};

use base64::{Engine, engine::general_purpose};

pub fn process_image(file_path: &str) -> Result<String, std::io::Error> {
    let path = Path::new(file_path);

    let data: Vec<u8> = fs::read(path)?;
    let encoded_data: String = general_purpose::STANDARD.encode(data);
    Ok(encoded_data)
}
