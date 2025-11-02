use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};
use tokio::io::AsyncReadExt;
use tokio::sync::{RwLock, Mutex};
use crate::kernel::buffer::text_buffer::TextBuffer;
use crate::kernel::buffer::cursor::{Cursor, CursorDirection};

static OPEN_BUFFERS: LazyLock<RwLock<TextBufferTable>> = LazyLock::new(|| {
    RwLock::new(TextBufferTable::new())
});

pub struct TextBufferTable {
    table: Vec<Option<Arc<Mutex<TextBuffer>>>>,
    free_list: VecDeque<usize>,
    name_to_index: HashMap<String, usize>,
}

impl TextBufferTable {
    pub fn new() -> Self {
        Self {
            table: Vec::new(),
            free_list: VecDeque::new(),
            name_to_index: HashMap::new(),
        }
    }

    fn insert_internal(&mut self, name: String, buffer: TextBuffer) -> BufferHandle {
        if let Some(index) = self.free_list.pop_front() {
            let buffer = Arc::new(Mutex::new(buffer));
            self.table[index] = Some(buffer.clone());
            self.name_to_index.insert(name, index);
            return BufferHandle::new(buffer, index);
        }
        let index = self.table.len();
        let buffer = Arc::new(Mutex::new(buffer));
        self.table.push(Some(buffer.clone()));
        self.name_to_index.insert(name, index);
        BufferHandle::new(buffer, index)
    }

    async fn rename_internal(&mut self, old_name: &str, new_name: String) {
        if let Some(index) = self.name_to_index.remove(old_name) {
            self.name_to_index.insert(new_name.clone(), index);
            self.table[index].as_ref().unwrap().lock().await.rename(new_name)
        }
    }

    async fn open_internal(&mut self, path: String) -> Result<BufferHandle, Box<dyn Error>> {
        let path_buf = PathBuf::from(path);
        let path = path_buf.canonicalize()?;

        let contents = {
            let mut file = tokio::fs::File::open(&path).await?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).await?;
            String::from_utf8(contents)?
        };

        let name = path.into_os_string().into_string().expect("String is not convertable");
        let buffer = TextBuffer::new(contents, name.clone());
        
        Ok(self.insert_internal(name, buffer))
    }
    
    pub async fn open(path: String) -> Result<BufferHandle, Box<dyn Error>> {
        let mut table = OPEN_BUFFERS.write().await;
        table.open_internal(path).await
    }
}


#[derive(Clone)]
pub struct BufferHandle {
    handle: Arc<Mutex<TextBuffer>>,
    index: usize,
}

impl BufferHandle {
    pub fn new(handle: Arc<Mutex<TextBuffer>>, index: usize) -> Self {
        Self { handle, index }
    }
    
    pub async fn rename(&self, name: String) {
        self.handle.lock().await.rename(name);
    }
    
    pub async fn get_text(&self) -> String {
        self.handle.lock().await.get_buffer()
    }
    
    pub async fn move_cursors(&self, cursors: Vec<Cursor>, direction: CursorDirection) -> Vec<Cursor> {
        self.handle.lock().await.move_cursors(cursors, direction)
    }
    
    pub async fn place_marks(&self, cursors: Vec<Cursor>) -> Vec<Cursor> {
        self.handle.lock().await.place_marks(cursors)
    }
    
    pub async fn remove_marks(&self, cursors: Vec<Cursor>) -> Vec<Cursor> {
        self.handle.lock().await.remove_marks(cursors)
    }
}



