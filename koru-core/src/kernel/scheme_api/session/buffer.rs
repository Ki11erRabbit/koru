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
    cursors: Vec<Cursor>,
}

impl Buffer {
    pub(crate) fn new(handle: BufferHandle) -> Self {
        Buffer {
            major_mode: Value::undefined(),
            handle,
            cursors: vec![Cursor::new_main(GridCursor::default())]
        }
    }

    pub fn set_major_mode(&mut self, major_mode: Value) {
        self.major_mode = major_mode;
    }

    pub fn get_handle(&self) -> BufferHandle {
        self.handle.clone()
    }

    pub async fn get_styled_text(&self) -> StyledFile {
        let text = self.handle.get_text().await;
        let file = StyledFile::from(text);
        let major_mode: Gc<MajorMode> = self.major_mode.clone().try_into_rust_type().unwrap();
        file.place_cursors(&self.cursors, major_mode).await
    }
    pub fn get_major_mode(&self) -> Value {
        self.major_mode.clone()
    }

    pub async fn move_cursor(&mut self, index: usize, direction: CursorDirection) {
        let handle = self.handle.clone();
        self.cursors[index] = handle.move_cursor(self.cursors[index], direction).await
    }

    pub async fn place_mark(&mut self, index: usize) {
        let handle = self.handle.clone();
        self.cursors[index] = handle.place_mark(self.cursors[index]).await
    }

    pub async fn remove_mark(&mut self, index: usize) {
        let handle = self.handle.clone();
        self.cursors[index] = handle.remove_mark(self.cursors[index]).await
    }

    pub fn num_cursors(&self) -> usize {
        self.cursors.len()
    }

    pub fn remove_cursor(&mut self, index: usize) {
        self.cursors.remove(index);
    }
    
    pub fn get_cursor_position(&self, index: usize) -> (usize, usize) {
        let cursor = &self.cursors[index];
        (cursor.line(), cursor.column())
    }
    
    pub fn add_cursor(&mut self, line: usize, column: usize) {
        let mut index = 0;
        'outer: for (i, cursor) in self.cursors.iter().enumerate() {
            if line < cursor.line() {
                index = i;
            } else if line == cursor.line() {
                for (i, cursor) in self.cursors.iter().enumerate() {
                    if line == cursor.line() {
                        if column >= cursor.column() {
                            if column == cursor.column() {
                                index = i;
                            }
                            break 'outer;
                        }
                    } 
                    index = i;
                }
            } else if line > cursor.line() {
                break 'outer;
            }
        }
        
        self.cursors.insert(index, Cursor::new(GridCursor::new(line, column)))
    }
}

#[bridge(name = "move-cursor-up", lib = "(koru-session)")]
pub async fn move_cursors_up(cursor_index: &Value) -> Result<Vec<Value>, Condition> {
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index = match cursor_index.as_ref() {
        Number::FixedInteger(fixed) => *fixed as usize,
        Number::BigInteger(big) => {
            let index: usize = big.try_into().unwrap();
            index
        }
        Number::Complex(..) => return Err(Condition::type_error("Integer", "Complex")),
        Number::Rational(..) => return Err(Condition::type_error("BigInteger", "Rational")),
        Number::Real(..) => return Err(Condition::type_error("BigInteger", "Real")),
    };
    let state = SessionState::get_state();
    let mut guard = state.lock().await;
    let Some(buffer) = guard.get_current_buffer_mut() else {
        return Err(Condition::error(String::from("Buffer not found")));
    };
    buffer.move_cursor(cursor_index, CursorDirection::Up).await;
    Ok(Vec::new())
}

#[bridge(name = "move-cursor-down", lib = "(koru-session)")]
pub async fn move_cursors_down(cursor_index: &Value) -> Result<Vec<Value>, Condition> {
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index = match cursor_index.as_ref() {
        Number::FixedInteger(fixed) => *fixed as usize,
        Number::BigInteger(big) => {
            let index: usize = big.try_into().unwrap();
            index
        }
        Number::Complex(..) => return Err(Condition::type_error("Integer", "Complex")),
        Number::Rational(..) => return Err(Condition::type_error("BigInteger", "Rational")),
        Number::Real(..) => return Err(Condition::type_error("BigInteger", "Real")),
    };
    let state = SessionState::get_state();
    let mut guard = state.lock().await;
    let Some(buffer) = guard.get_current_buffer_mut() else {
        return Err(Condition::error(String::from("Buffer not found")));
    };
    buffer.move_cursor(cursor_index, CursorDirection::Down).await;
    Ok(Vec::new())
}

#[bridge(name = "move-cursor-left", lib = "(koru-session)")]
pub async fn move_cursors_left(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((cursor_index, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()));
    };
    let Some((wrap, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()));
    };
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index = match cursor_index.as_ref() {
        Number::FixedInteger(fixed) => *fixed as usize,
        Number::BigInteger(big) => {
            let index: usize = big.try_into().unwrap();
            index
        }
        Number::Complex(..) => return Err(Condition::type_error("Integer", "Complex")),
        Number::Rational(..) => return Err(Condition::type_error("BigInteger", "Rational")),
        Number::Real(..) => return Err(Condition::type_error("BigInteger", "Real")),
    };
    let wrap: bool = wrap.clone().try_into()?;
    let state = SessionState::get_state();
    let mut guard = state.lock().await;
    let Some(buffer) = guard.get_current_buffer_mut() else {
        return Err(Condition::error(String::from("Buffer not found")));
    };
    buffer.move_cursor(cursor_index, CursorDirection::Left { wrap }).await;
    Ok(Vec::new())
}

#[bridge(name = "move-cursor-right", lib = "(koru-session)")]
pub async fn move_cursors_right(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((cursor_index, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()));
    };
    let Some((wrap, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()));
    };
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index = match cursor_index.as_ref() {
        Number::FixedInteger(fixed) => *fixed as usize,
        Number::BigInteger(big) => {
            let index: usize = big.try_into().unwrap();
            index
        }
        Number::Complex(..) => return Err(Condition::type_error("Integer", "Complex")),
        Number::Rational(..) => return Err(Condition::type_error("BigInteger", "Rational")),
        Number::Real(..) => return Err(Condition::type_error("BigInteger", "Real")),
    };
    let wrap: bool = wrap.clone().try_into()?;
    let state = SessionState::get_state();
    let mut guard = state.lock().await;
    let Some(buffer) = guard.get_current_buffer_mut() else {
        return Err(Condition::error(String::from("Buffer not found")));
    };
    buffer.move_cursor(cursor_index, CursorDirection::Right { wrap }).await;
    Ok(Vec::new())
}

#[bridge(name = "place-cursor-mark", lib = "(koru-session)")]
pub async fn place_marks(cursor_index: &Value) -> Result<Vec<Value>, Condition> {
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index = match cursor_index.as_ref() {
        Number::FixedInteger(fixed) => *fixed as usize,
        Number::BigInteger(big) => {
            let index: usize = big.try_into().unwrap();
            index
        }
        Number::Complex(..) => return Err(Condition::type_error("Integer", "Complex")),
        Number::Rational(..) => return Err(Condition::type_error("BigInteger", "Rational")),
        Number::Real(..) => return Err(Condition::type_error("BigInteger", "Real")),
    };
    let state = SessionState::get_state();
    let mut guard = state.lock().await;
    let Some(buffer) = guard.get_current_buffer_mut() else {
        return Err(Condition::error(String::from("Buffer not found")));
    };
    buffer.place_mark(cursor_index).await;
    Ok(Vec::new())
}

#[bridge(name = "remove-cursor-mark", lib = "(koru-session)")]
pub async fn remove_marks(cursor_index: &Value) -> Result<Vec<Value>, Condition> {
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index = match cursor_index.as_ref() {
        Number::FixedInteger(fixed) => *fixed as usize,
        Number::BigInteger(big) => {
            let index: usize = big.try_into().unwrap();
            index
        }
        Number::Complex(..) => return Err(Condition::type_error("Integer", "Complex")),
        Number::Rational(..) => return Err(Condition::type_error("BigInteger", "Rational")),
        Number::Real(..) => return Err(Condition::type_error("BigInteger", "Real")),
    };
    let state = SessionState::get_state();
    let mut guard = state.lock().await;
    let Some(buffer) = guard.get_current_buffer_mut() else {
        return Err(Condition::error(String::from("Buffer not found")));
    };
    buffer.remove_mark(cursor_index).await;
    Ok(Vec::new())
}


#[bridge(name = "cursor-position", lib = "(koru-session)")]
pub async fn get_cursor_position(cursor_index: &Value) -> Result<Vec<Value>, Condition> {
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index = match cursor_index.as_ref() {
        Number::FixedInteger(fixed) => *fixed as usize,
        Number::BigInteger(big) => {
            let index: usize = big.try_into().unwrap();
            index
        }
        Number::Complex(..) => return Err(Condition::type_error("Integer", "Complex")),
        Number::Rational(..) => return Err(Condition::type_error("BigInteger", "Rational")),
        Number::Real(..) => return Err(Condition::type_error("BigInteger", "Real")),
    };
    let state = SessionState::get_state();
    let mut guard = state.lock().await;
    let Some(buffer) = guard.get_current_buffer_mut() else {
        return Err(Condition::error(String::from("Buffer not found")));
    };
    let (row, column): (usize, usize) = buffer.get_cursor_position(cursor_index);
    
    let pair = Value::new(UnpackedValue::Pair(Gc::new(Pair::new(Value::from(Number::from(row)), Value::from(Number::from(column))))));
    
    Ok(vec![pair])
}

#[bridge(name = "cursor-create", lib = "(koru-session)")]
pub async fn create_cursor(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((pair, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(1, args.len()));
    };
    let (row, col) = match pair.clone().unpack() {
        UnpackedValue::Pair(pair) => {
            let left = pair.read().0.clone();
            let right = pair.read().1.clone();
            let row: Arc<Number> = left.try_into()?;
            let col: Arc<Number> = right.try_into()?;
            let row: usize = match row.as_ref() {
                Number::FixedInteger(fixed) => *fixed as usize,
                Number::BigInteger(big) => {
                    big.try_into().unwrap()
                }
                Number::Complex(..) => return Err(Condition::type_error("BigInteger", "Complex")),
                Number::Rational(..) => return Err(Condition::type_error("BigInteger", "Rational")),
                Number::Real(..) => return Err(Condition::type_error("BigInteger", "Real")),
            };
            let col: usize = match col.as_ref() {
                Number::FixedInteger(fixed) => *fixed as usize,
                Number::BigInteger(big) => {
                    big.try_into().unwrap()
                }
                Number::Complex(..) => return Err(Condition::type_error("BigInteger", "Complex")),
                Number::Rational(..) => return Err(Condition::type_error("BigInteger", "Rational")),
                Number::Real(..) => return Err(Condition::type_error("BigInteger", "Real")),
            };
            (row, col)
        }
        UnpackedValue::Number(row) => {
            let Some((col, _)) = rest.split_first() else {
                return Err(Condition::wrong_num_of_args(2, args.len()));
            };
            let col: Arc<Number> = col.clone().try_into()?;
            let row: usize = match row.as_ref() {
                Number::FixedInteger(fixed) => *fixed as usize,
                Number::BigInteger(big) => {
                    big.try_into().unwrap()
                }
                Number::Complex(..) => return Err(Condition::type_error("BigInteger", "Complex")),
                Number::Rational(..) => return Err(Condition::type_error("BigInteger", "Rational")),
                Number::Real(..) => return Err(Condition::type_error("BigInteger", "Real")),
            };
            let col: usize = match col.as_ref() {
                Number::FixedInteger(fixed) => *fixed as usize,
                Number::BigInteger(big) => {
                    big.try_into().unwrap()
                }
                Number::Complex(..) => return Err(Condition::type_error("BigInteger", "Complex")),
                Number::Rational(..) => return Err(Condition::type_error("BigInteger", "Rational")),
                Number::Real(..) => return Err(Condition::type_error("BigInteger", "Real")),
            };
            (row, col)
        }
        ty => return Err(Condition::type_error("Pair or Integer", ty.type_name()))
    };
    let state = SessionState::get_state();
    let mut guard = state.lock().await;
    let Some(buffer) = guard.get_current_buffer_mut() else {
        return Err(Condition::error(String::from("Buffer not found")));
    };
    buffer.add_cursor(row, col);

    Ok(Vec::new())
}