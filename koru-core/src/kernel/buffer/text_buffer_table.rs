use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};
use crop::Rope;
use scheme_rs::exceptions::Exception;
use scheme_rs::gc::Gc;
use scheme_rs::records::Record;
use scheme_rs::registry::bridge;
use scheme_rs::value::Value;
use tokio::io::AsyncReadExt;
use tokio::sync::{RwLock, Mutex};
use crate::kernel::buffer::text_buffer::TextBuffer;
use crate::kernel::buffer::cursor::{Cursor, CursorDirection};
use crate::kernel::buffer::Cursors;
use crate::kernel::scheme_api::session::SessionState;

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

        let mut contents = {
            let mut file = tokio::fs::File::open(&path).await?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).await?;
            String::from_utf8(contents)?
        };

        if !contents.ends_with("\n") {
            contents.push('\n');
        }

        let name = path.into_os_string().into_string().expect("String is not convertable");
        let buffer = TextBuffer::new(contents, name.clone());
        
        Ok(self.insert_internal(name, buffer))
    }
    
    pub async fn open(path: String) -> Result<BufferHandle, Box<dyn Error>> {
        let mut table = OPEN_BUFFERS.write().await;
        table.open_internal(path).await
    }

    pub async fn create<S: AsRef<str>>(name: String, contents: S) -> Result<BufferHandle, Box<dyn Error>> {
        let mut table = OPEN_BUFFERS.write().await;
        let text_buffer = TextBuffer::new(contents.as_ref(), &name.clone());
        Ok(table.insert_internal(name, text_buffer))
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
    
    pub async fn get_text(&self) -> Rope {
        self.handle.lock().await.get_buffer()
    }

    pub async fn get_name(&self) -> String {
        self.handle.lock().await.get_name()
    }
    
    pub async fn move_cursors(&self, cursors: Vec<Cursor>, direction: CursorDirection) -> Vec<Cursor> {
        self.handle.lock().await.move_cursors(cursors, direction)
    }

    pub async fn move_cursor(&self, cursor: Cursor, direction: CursorDirection) -> Cursor {
        self.handle.lock().await.move_cursor(cursor, direction)
    }
    
    pub async fn place_point_marks(&self, cursors: Vec<Cursor>) -> Vec<Cursor> {
        self.handle.lock().await.place_point_marks(cursors)
    }
    pub async fn place_point_mark(&self, cursor: Cursor) -> Cursor {
        self.handle.lock().await.place_point_mark(cursor)
    }

    pub async fn place_line_marks(&self, cursors: Vec<Cursor>) -> Vec<Cursor> {
        self.handle.lock().await.place_line_marks(cursors)
    }
    pub async fn place_line_mark(&self, cursor: Cursor) -> Cursor {
        self.handle.lock().await.place_line_mark(cursor)
    }

    pub async fn place_box_marks(&self, cursors: Vec<Cursor>) -> Vec<Cursor> {
        self.handle.lock().await.place_box_marks(cursors)
    }
    pub async fn place_box_mark(&self, cursor: Cursor) -> Cursor {
        self.handle.lock().await.place_box_mark(cursor)
    }
    
    pub async fn place_file_marks(&self, cursors: Vec<Cursor>) -> Vec<Cursor> {
        self.handle.lock().await.place_file_marks(cursors)
    }
    pub async fn place_file_mark(&self, cursor: Cursor) -> Cursor {
        self.handle.lock().await.place_file_mark(cursor)
    }
    
    pub async fn remove_marks(&self, cursors: Vec<Cursor>) -> Vec<Cursor> {
        self.handle.lock().await.remove_marks(cursors)
    }
    pub async fn remove_mark(&self, cursor: Cursor) -> Cursor {
        self.handle.lock().await.remove_mark(cursor)
    }

    pub async fn insert(&self, text: String, cursor_index: usize, cursors: Vec<Cursor>) -> Vec<Cursor> {
        self.handle.lock().await.insert(text, cursor_index, cursors).await
    }

    pub async fn delete_back(&self, cursor_index: usize, cursors: Vec<Cursor>) -> Vec<Cursor> {
        self.handle.lock().await.delete_back(cursor_index, cursors).await
    }

    pub async fn delete_forward(&self, cursor_index: usize, cursors: Vec<Cursor>) -> Vec<Cursor> {
        self.handle.lock().await.delete_forward(cursor_index, cursors).await
    }

    pub async fn delete_region(&self, cursor_index: usize, cursors: Vec<Cursor>) -> Vec<Cursor> {
        self.handle.lock().await.delete_region(cursor_index, cursors).await
    }

    pub async fn replace(&self, text: String, cursor_index: usize, cursors: Vec<Cursor>) -> Vec<Cursor> {
        self.handle.lock().await.replace(text, cursor_index, cursors).await
    }
    
    pub async fn start_transaction(&self) {
        self.handle.lock().await.start_transaction().await;
    }
    
    pub async fn end_transaction(&self) {
        self.handle.lock().await.end_transaction().await;
    }

    pub async fn undo(&self) {
        self.handle.lock().await.undo().await;
    }

    pub async fn redo(&self) {
        self.handle.lock().await.redo().await;
    }

    pub async fn save(&self) -> Result<(), Exception> {
        self.handle.lock().await.save().await
    }

    pub async fn save_as(&self, path: &str) -> Result<(), Exception> {
        self.handle.lock().await.save_as(path).await
    }

    pub async fn get_path(&self) -> Option<String> {
        self.handle.lock().await.get_path()
    }
}


#[bridge(name = "buffer-from-path", lib = "(koru-buffer)")]
pub async fn buffer_from_file(path: &Value) -> Result<Vec<Value>, Exception> {
    let path: String = path.clone().try_into()?;

    let handle = TextBufferTable::open(path).await
        .map_err(|e| Exception::error(e))?;
    let name = handle.get_name().await;

    Ok(vec![Value::from(name)])
}

#[bridge(name = "buffer-create", lib = "(koru-buffer)")]
pub async fn buffer_create(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((path, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(1, args.len()));
    };
    let contents = if let Some((contents, _)) = rest.split_first() {
        let contents = contents.clone().try_into()?;
        contents
    } else {
        String::new()
    };

    let buffer_name: String = path.clone().try_into()?;

    let handle = TextBufferTable::create(buffer_name, contents).await
        .map_err(|e| Exception::error(e))?;

    let name = handle.get_name().await;
    Ok(vec![Value::from(name)])
}

#[bridge(name = "buffer-save", lib = "(koru-buffer)")]
pub async fn save_buffer(buffer_name: &Value) -> Result<Vec<Value>, Exception> {
    let buffer_name: String = buffer_name.clone().try_into()?;
    let buffer = {
        let state = SessionState::get_state();
        let guard = state.read().await;
        let buffers = guard.get_buffers().await;
        buffers.get(buffer_name.as_str()).cloned()
    };
    let Some(buffer) = buffer else {
        return Err(Exception::error(String::from("Buffer not found")))
    };

    let handle = buffer.get_handle();
    handle.save().await.map_err(|e| Exception::error(e))?;
    Ok(vec![])
}

#[bridge(name = "buffer-save-as", lib = "(koru-buffer)")]
pub async fn save_buffer_as(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((buffer_name, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()));
    };
    let Some((new_name, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()));
    };
    let buffer_name: String = buffer_name.clone().try_into()?;
    let new_name: String = new_name.clone().try_into()?;
    let buffer = {
        let state = SessionState::get_state();
        let guard = state.read().await;
        let buffers = guard.get_buffers().await;
        buffers.get(buffer_name.as_str()).cloned()
    };
    let Some(buffer) = buffer else {
        return Err(Exception::error(String::from("Buffer not found")))
    };

    let handle = buffer.get_handle();
    handle.save_as(&new_name).await.map_err(|e| Exception::error(e))?;
    Ok(vec![])
}

#[bridge(name = "buffer-get-path", lib = "(koru-buffer)")]
pub async fn get_path(buffer_name: &Value) -> Result<Vec<Value>, Exception> {
    let buffer_name: String = buffer_name.clone().try_into()?;
    let buffer = {
        let state = SessionState::get_state();
        let guard = state.read().await;
        let buffers = guard.get_buffers().await;
        buffers.get(buffer_name.as_str()).cloned()
    };
    let Some(buffer) = buffer else {
        return Err(Exception::error(String::from("Buffer not found")))
    };

    let handle = buffer.get_handle();
    let value = handle.get_path().await.map(|path| Value::from(path)).unwrap_or(Value::null());
    Ok(vec![value])
}

#[bridge(name = "plain-draw", lib = "(koru-buffer)")]
pub async fn text_edit_draw(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((buffer_name, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()));
    };
    let Some((cursors, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()));
    };
    let buffer = {
        let buffer_name: String = buffer_name.clone().try_into()?;
        let state = SessionState::get_state();
        let mut guard = state.write().await;
        let mut buffers = guard.get_buffers_mut().await;
        let buffer = buffers.get_mut(&buffer_name).unwrap();
        buffer.render_styled_text().await;
        buffer.clone()
    };
    let cursors: Gc<Cursors> = cursors.try_to_rust_type()?;

    let styled_text = buffer.get_styled_text(&cursors.cursors);

    let value = Value::from(Record::from_rust_type(styled_text));
    Ok(vec![value])
}
