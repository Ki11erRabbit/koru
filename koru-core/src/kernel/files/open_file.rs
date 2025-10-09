use std::error::Error;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

pub struct OpenFile {
    buffer: String,
    file_name: String,
    absolute_path: PathBuf,
}

impl OpenFile {
    pub fn new(buffer: String, absolute_path: impl AsRef<Path>) -> Self {
        let file_name = absolute_path.as_ref().file_name().unwrap().to_str().unwrap().to_string();
        OpenFile {
            buffer,
            file_name,
            absolute_path: absolute_path.as_ref().to_path_buf(),
        }
    }
    
    pub async fn save(&self) -> Result<(), Box<dyn Error>> {
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.absolute_path).await?;
        
        file.write_all(self.buffer.as_bytes()).await?;
        file.flush().await?;
        Ok(())
    }
    
    pub fn absolute_path(&self) -> &Path {
        &self.absolute_path
    }
}