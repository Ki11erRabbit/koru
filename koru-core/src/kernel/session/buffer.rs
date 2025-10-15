use mlua::UserData;
use crate::kernel::cursor::Cursor;
use crate::kernel::files::OpenFileHandle;
use crate::styled_text::StyledFile;

pub enum BufferData {
    OpenFile(OpenFileHandle),
    Log(Vec<String>),
}


pub struct Buffer {
    buffer: BufferData,
    cursors: Vec<Cursor>,
}


impl Buffer {
    pub fn new_open_file(handle: OpenFileHandle) -> Self {
        Buffer {
            buffer: BufferData::OpenFile(handle),
            cursors: vec![Cursor::new(0, 1)]
        }
    }
    
    pub fn new_log() -> Self {
        Buffer {
            buffer: BufferData::Log(Vec::new()),
            cursors: vec![Cursor::new(0, 1)]
        }
    }
    
    pub async fn styled_file(&self) -> StyledFile {
        match &self.buffer {
            BufferData::OpenFile(handle) => {
                let text = handle.get_text().await;
                let file = StyledFile::from(text);
                file.place_cursors(&self.cursors)
            },
            BufferData::Log(lines) => {
                let joined = lines.join("\n");
                StyledFile::from(joined)
            }
        }
    }
    
    pub fn manipulate_data<F>(&mut self, func: F)
    where
        F: FnOnce(&mut BufferData),
    {
        func(&mut self.buffer);
    }
}

impl UserData for Buffer {
    
}