use mlua::{UserData, UserDataMethods};
use crate::kernel::buffer::BufferHandle;
use crate::kernel::cursor::{Cursor, CursorDirection};
use crate::styled_text::StyledFile;

pub enum BufferData {
    OpenFile(BufferHandle),
    Log(Vec<String>),
}


pub struct Buffer {
    buffer: BufferData,
    cursors: Vec<Cursor>,
}


impl Buffer {
    pub fn new_open_file(handle: BufferHandle) -> Self {
        let mut cursor = Cursor::default();
        cursor = Cursor::new_main(cursor.logical_cursor, cursor.byte_cursor, cursor.leading_edge);
        Buffer {
            buffer: BufferData::OpenFile(handle),
            cursors: vec![cursor]
        }
    }
    
    pub fn new_log() -> Self {
        Buffer {
            buffer: BufferData::Log(Vec::new()),
            cursors: vec![]
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
    
    pub fn manipulate<F>(&mut self, func: F)
    where
        F: FnOnce(&mut Vec<Cursor>, &mut BufferData),
    {
        func(&mut self.cursors, &mut self.buffer);
    }
}

impl UserData for Buffer {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method_mut(
            "cursor_up",
            async |_, mut this, _:()| {
                match &this.buffer {
                    BufferData::OpenFile(handle) => {
                        let cursors = handle.move_cursors(this.cursors.clone(), CursorDirection::Up).await;
                        this.cursors = cursors;
                    }
                    _ => {}
                }
                Ok(())
            }
        );
        methods.add_async_method_mut(
            "cursor_down",
            async |_, mut this, _:()| {
                match &this.buffer {
                    BufferData::OpenFile(handle) => {
                        let cursors = handle.move_cursors(this.cursors.clone(), CursorDirection::Down).await;
                        this.cursors = cursors;
                    }
                    _ => {}
                }
                Ok(())
            }
        );
        methods.add_async_method_mut(
            "cursor_left",
            async |_, mut this, _:()| {
                match &this.buffer {
                    BufferData::OpenFile(handle) => {
                        let cursors = handle.move_cursors(this.cursors.clone(), CursorDirection::Left { wrap: false }).await;
                        this.cursors = cursors;
                    }
                    _ => {}
                }
                Ok(())
            }
        );
        methods.add_async_method_mut(
            "cursor_right",
            async |_, mut this, _:()| {
                match &this.buffer {
                    BufferData::OpenFile(handle) => {
                        let cursors = handle.move_cursors(this.cursors.clone(), CursorDirection::Right { wrap: false }).await;
                        this.cursors = cursors;
                    }
                    _ => {}
                }
                Ok(())
            }
        );
    }
}