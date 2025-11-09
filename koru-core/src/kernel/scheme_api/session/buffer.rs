use crate::kernel::scheme_api::session::SessionState;
use std::sync::{Arc};
use scheme_rs::exceptions::Condition;
use scheme_rs::gc::Gc;
use scheme_rs::lists::Pair;
use scheme_rs::num::Number;
use scheme_rs::registry::bridge;
use scheme_rs::value::{UnpackedValue, Value};
use crate::kernel::buffer::{BufferHandle, Cursor, CursorDirection, GridCursor};
use crate::kernel::scheme_api::major_mode::MajorMode;
use crate::styled_text::StyledFile;

#[derive(Clone)]
pub struct Buffer {
    major_mode: Value,
    handle: BufferHandle,
}

impl Buffer {
    pub(crate) fn new(handle: BufferHandle) -> Self {
        Buffer {
            major_mode: Value::undefined(),
            handle,
        }
    }

    pub fn set_major_mode(&mut self, major_mode: Value) {
        self.major_mode = major_mode;
    }

    pub fn get_handle(&self) -> BufferHandle {
        self.handle.clone()
    }

    pub async fn get_styled_text(&self, major_mode: Gc<MajorMode>, cursors: &[Cursor]) -> StyledFile {
        let text = self.handle.get_text().await;
        let file = StyledFile::from(text);
        file.place_cursors(cursors, major_mode).await
    }
    pub fn get_major_mode(&self) -> Value {
        self.major_mode.clone()
    }
}