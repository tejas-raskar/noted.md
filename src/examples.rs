use crate::{error::NotedError, file_utils::FileData};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub id: String,
    pub image_path: String,
    pub markdown_content: String,
    pub tags: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct ExampleContext {
    pub examples: Vec<ProcessedExample>,
}

#[derive(Debug, Clone)]
pub struct ProcessedExample {
    pub image_data: FileData,
    pub markdown_content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExampleDatabase {
    examples: Vec<Example>,
}

impl ExampleContext {
    pub fn new() -> Self {
        Self {
            examples: Vec::new(),
        }
    }

    pub fn add_example(&mut self, example: ProcessedExample) {
        self.examples.push(example);
    }

    pub fn is_empty(&self) -> bool {
        self.examples.is_empty()
    }

    pub fn len(&self) -> usize {
        self.examples.len()
    }
}

pub struct ExampleManager {
    database_path: String,
    examples_dir: String,
}

impl ExampleManager {
    pub fn new(database_path: String, examples_dir: String) -> Self {
        Self {
            database_path,
            examples_dir,
        }
    }

    pub fn initialize(&self) -> Result<(), NotedError> {
        // Create examples directory if it doesn't exist
        fs::create_dir_all(&self.examples_dir)?;
        
        // Create database file if it doesn't exist
        if !Path::new(&self.database_path).exists() {
            let empty_db = ExampleDatabase {
                examples: Vec::new(),
            };
            self.save_database(&empty_db)?;
        }
        
        Ok(())
    }

    pub fn add_example(
        &self,
        image_path: &str,
        markdown_content: &str,
        tags: Vec<String>,
    ) -> Result<String, NotedError> {
        let mut db = self.load_database()?;
        
        // Generate unique ID
        let id = format!("example_{}", chrono::Utc::now().timestamp());
        
        // Copy image to examples directory
        let image_filename = format!("{}.png", id);
        let dest_image_path = Path::new(&self.examples_dir).join(&image_filename);
        fs::copy(image_path, &dest_image_path)?;
        
        let example = Example {
            id: id.clone(),
            image_path: dest_image_path.to_string_lossy().to_string(),
            markdown_content: markdown_content.to_string(),
            tags,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        db.examples.push(example);
        self.save_database(&db)?;
        
        Ok(id)
    }

    pub fn remove_example(&self, id: &str) -> Result<(), NotedError> {
        let mut db = self.load_database()?;
        
        if let Some(pos) = db.examples.iter().position(|e| e.id == id) {
            let example = &db.examples[pos];
            
            // Remove image file
            if Path::new(&example.image_path).exists() {
                fs::remove_file(&example.image_path)?;
            }
            
            // Remove from database
            db.examples.remove(pos);
            self.save_database(&db)?;
        }
        
        Ok(())
    }

    pub fn list_examples(&self, tag_filter: Option<&str>) -> Result<Vec<Example>, NotedError> {
        let db = self.load_database()?;
        
        match tag_filter {
            Some(tag) => Ok(db
                .examples
                .into_iter()
                .filter(|e| e.tags.contains(&tag.to_string()))
                .collect()),
            None => Ok(db.examples),
        }
    }

    pub fn get_example(&self, id: &str) -> Result<Option<Example>, NotedError> {
        let db = self.load_database()?;
        Ok(db.examples.into_iter().find(|e| e.id == id))
    }

    pub fn load_examples_with_tags(&self, tags: &[String], limit: Option<usize>) -> Result<ExampleContext, NotedError> {
        let db = self.load_database()?;
        let mut context = ExampleContext::new();
        
        let filtered_examples: Vec<_> = db
            .examples
            .into_iter()
            .filter(|e| {
                if tags.is_empty() {
                    true
                } else {
                    tags.iter().any(|tag| e.tags.contains(tag))
                }
            })
            .take(limit.unwrap_or(usize::MAX))
            .collect();
        
        for example in filtered_examples {
            if !Path::new(&example.image_path).exists() {
                continue; // Skip missing files
            }
            
            let image_data = crate::file_utils::process_file(&example.image_path)?;
            let processed_example = ProcessedExample {
                image_data,
                markdown_content: example.markdown_content,
            };
            
            context.add_example(processed_example);
        }
        
        Ok(context)
    }

    fn load_database(&self) -> Result<ExampleDatabase, NotedError> {
        let content = fs::read_to_string(&self.database_path)?;
        let db: ExampleDatabase = serde_json::from_str(&content)
            .map_err(|e| NotedError::ResponseDecodeError(format!("Failed to parse examples database: {}", e)))?;
        Ok(db)
    }

    fn save_database(&self, db: &ExampleDatabase) -> Result<(), NotedError> {
        let content = serde_json::to_string_pretty(db)
            .map_err(|e| NotedError::ResponseDecodeError(format!("Failed to serialize examples database: {}", e)))?;
        fs::write(&self.database_path, content)?;
        Ok(())
    }
}