use crate::error::NotedError;
use crate::file_utils::FileData;
use base64::Engine;
use pdf2image::{PDF2ImageError, PDF, RenderOptionsBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use image::ImageFormat;
use base64;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingProgress {
    pub last_processed_page: u32,
    pub total_pages: u32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProgressTracker {
    files: HashMap<String, ProcessingProgress>,
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    pub fn load() -> Result<Self, NotedError> {
        let progress_file = Self::get_progress_file_path()?;
        if progress_file.exists() {
            let content = fs::read_to_string(&progress_file)?;
            serde_json::from_str(&content).map_err(NotedError::JsonError)
        } else {
            Ok(Self::new())
        }
    }

    pub fn save(&self) -> Result<(), NotedError> {
        let progress_file = Self::get_progress_file_path()?;
        let content = serde_json::to_string_pretty(self).map_err(NotedError::JsonError)?;
        fs::write(&progress_file, content)?;
        Ok(())
    }

    fn get_progress_file_path() -> Result<PathBuf, NotedError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| NotedError::ConfigDirError("Could not find config directory".into()))?;
        let progress_dir = config_dir.join("notedmd");
        fs::create_dir_all(&progress_dir)?;
        Ok(progress_dir.join("progress.json"))
    }

    pub fn get_progress(&self, file_path: &str) -> Option<&ProcessingProgress> {
        self.files.get(file_path)
    }

    pub fn update_progress(&mut self, file_path: String, progress: ProcessingProgress) {
        self.files.insert(file_path, progress);
    }

    pub fn mark_completed(&mut self, file_path: &str) {
        self.files.remove(file_path);
    }
}

pub fn process_pdf(pdf_path: &str) -> Result<(PDF, u32), NotedError> {
    let pdf = PDF::from_file(pdf_path).unwrap();
        // .map_err(|e| NotedError::PdfError(e.to_string()))?;
    let options = RenderOptionsBuilder::default()
        .build()
        .map_err(|e| NotedError::PdfError(e.to_string()))?;

    // Count total pages by trying to render all pages
    let total_pages = pdf.page_count();
        // .map_err(|e| NotedError::PdfError(e.to_string()))?;

    Ok((pdf, total_pages))
}

pub fn extract_page_as_image(pdf: &PDF, page_num: u32) -> Result<FileData, PDF2ImageError> {
    let temp_file = NamedTempFile::new()?;

    // Setup render options (example: high DPI for better quality)
    let options = RenderOptionsBuilder::default()
        // .dpi(300)
        .build()?;

    // Render the specific page (page numbers are 1-based)
    let images = pdf.render(
        pdf2image::Pages::Single(page_num + 1),
        options,
    )?;

    // Get the rendered page image, or return an I/O error if not found
    let image = images.get(0).ok_or_else(|| PDF2ImageError::Io(
        std::io::Error::new(std::io::ErrorKind::Other, "Failed to render the requested page")))?;

    // Save to PNG with maximum quality
    image.save_with_format(temp_file.path(), ImageFormat::Png)?;

    // Read and encode
    let image_data = fs::read(temp_file.path())?;
    let encoded_data = base64::engine::general_purpose::STANDARD.encode(&image_data);

    Ok(FileData {
        encoded_data,
        mime_type: "image/png".to_string(),
    })
}