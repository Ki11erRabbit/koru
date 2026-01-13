use scheme_rs::exceptions::{Condition, Exception};
use scheme_rs::gc::Gc;
use scheme_rs::symbols::Symbol;
use scheme_rs::value::{Value};
use crate::kernel::buffer::{BufferHandle, Cursor};
use crate::kernel::scheme_api::major_mode::MajorMode;
use crate::kernel::scheme_api::minor_mode::{MinorModeManager};
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

    pub async fn set_major_mode(&mut self, major_mode: Value) -> Result<(), Condition>{
        {
            let mm: Gc<MajorMode> = major_mode.clone().try_into_rust_type()?;
            let gain_focus = mm.gain_focus();
            gain_focus.call(&[major_mode.clone()]).await?;
        }
        self.major_mode = major_mode;
        Ok(())
    }

    pub async fn add_minor_mode(&mut self, minor_mode: Value) -> Result<(), Condition> {
        self.minor_modes.add_minor_mode(minor_mode).await
    }

    pub async fn remove_minor_mode(&mut self, minor_mode: Symbol) -> Option<String> {
        self.minor_modes.remove_minor_mode(minor_mode).await
    }

    pub fn get_minor_modes(&self) -> Vec<Value> {
        self.minor_modes.get_minor_modes()
    }

    pub async fn get_minor_mode(&self, minor_mode: Symbol) -> Option<Value> {
        self.minor_modes.get_minor_mode(minor_mode).await.cloned()
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

    pub async fn get_main_cursor(&self) -> Result<Cursor, Exception> {
        let mm_value = self.major_mode.clone();
        let major_mode: Gc<MajorMode> = self.major_mode.clone()
            .try_into_rust_type()?;
        major_mode.get_main_cursor(mm_value).await
    }
}