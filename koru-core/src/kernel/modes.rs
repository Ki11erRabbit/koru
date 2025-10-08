//! Modes are the Fundamental Units of Koru
//! 
//! Major modes define how the buffer will draw itself.
//! They also provide API that minor modes can consume.
//! 
//! Minor modes provide the input layer and issue commands to the major mode.
//! They can have their own commands and state.
mod major;
mod minor;

pub use major::MajorMode;
pub use minor::MinorMode;
use crate::key::KeyPress;

pub type KeyBuffer = Vec<KeyPress>;

/// A command is a function that takes in a `KeyBuffer` to process the keypress
pub type Command = mlua::Function;