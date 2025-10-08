//! The Buffer is a window into the editor.
//!
//! The buffer provides an opaque space to allow for the editor to work with
//!

use crate::kernel::modes::{KeyBuffer, MajorMode, MinorMode};


pub struct Buffer {
    width: usize,
    height: usize,
    major_mode: MajorMode,
    minor_modes: Vec<MinorMode>,
    key_buffer: KeyBuffer,
}