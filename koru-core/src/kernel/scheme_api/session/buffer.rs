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
use crate::kernel::scheme_api::minor_mode::{MinorMode, MinorModeManager};
use crate::styled_text::StyledFile;

#[derive(Clone)]
pub struct Buffer {
    major_mode: Value,
    handle: BufferHandle,
    styled_text: StyledFile,
    minor_modes: MinorModeManager,
}

impl Buffer {
    pub(crate) fn new(handle: BufferHandle) -> Self {
        Buffer {
            major_mode: Value::undefined(),
            handle,
            styled_text: StyledFile::default(),
            minor_modes: MinorModeManager::new(),
        }
    }

    pub fn set_major_mode(&mut self, major_mode: Value) {
        self.major_mode = major_mode;
    }

    pub fn add_minor_mode(&mut self, minor_mode: Gc<MinorMode>) {
        self.minor_modes.add_minor_mode(minor_mode);
    }

    pub fn remove_minor_mode(&mut self, minor_mode: &str) -> Option<String> {
        self.minor_modes.remove_minor_mode(minor_mode)
    }

    pub fn get_handle(&self) -> BufferHandle {
        self.handle.clone()
    }

    pub async fn render_styled_text(&mut self) {
        let text = self.handle.get_text().await;
        self.styled_text = StyledFile::from(text);
    }

    pub fn get_styled_text(&self, cursors: &[Cursor]) -> StyledFile {
        let file = self.styled_text.clone();
        file.place_cursors(cursors)
    }
    pub fn get_major_mode(&self) -> Value {
        self.major_mode.clone()
    }

    pub async fn get_main_cursor(&self) -> Cursor {
        let major_mode: Gc<MajorMode> = self.major_mode.clone().try_into_rust_type().unwrap();
        let data = crate::kernel::scheme_api::major_mode::text_edit::get_data(&major_mode).await.unwrap();

        data.get_main_cursor().await
    }
}