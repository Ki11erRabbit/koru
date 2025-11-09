use std::sync::Arc;
use scheme_rs::exceptions::Condition;
use scheme_rs::gc::{Gc, Trace};
use scheme_rs::lists::Pair;
use scheme_rs::num::Number;
use scheme_rs::records::{rtd, Record, RecordTypeDescriptor, SchemeCompatible};
use scheme_rs::registry::bridge;
use scheme_rs::value::{UnpackedValue, Value};
use tokio::sync::Mutex;
use crate::kernel::buffer::{BufferHandle, Cursor, CursorDirection, GridCursor};
use crate::kernel::scheme_api::major_mode::MajorMode;
use crate::kernel::scheme_api::session::SessionState;

#[derive(Debug, Trace)]
struct TextEditDataInternal {
    buffer_name: String,
    cursors: Vec<Cursor>,
}

#[derive(Debug, Clone, Trace)]
pub struct TextEditData {
    internal: Arc<Mutex<TextEditDataInternal>>
}

impl TextEditData {
    pub fn new(buffer_name: String) -> Self {
        let cursors = vec![Cursor::new_main(GridCursor::new(0,0))];
        let internal = TextEditDataInternal { buffer_name, cursors };
        TextEditData {
            internal: Arc::new(Mutex::new(internal)),
        }
    }
    
    async fn get_buffer_handle(&self) -> Result<BufferHandle, Condition> {
        let state = SessionState::get_state();
        let guard = state.read().await;
        let buffer_guard=  guard.get_buffers().await;
        let Some(buffer) = buffer_guard.get(&self.internal.lock().await.buffer_name) else {
            return Err(Condition::error(String::from("Buffer not found")));
        };
        let handle = buffer.get_handle();
        Ok(handle)
    }
    pub async fn move_cursor(&self, index: usize, direction: CursorDirection) -> Result<(), Condition> {
        let handle = self.get_buffer_handle().await?;
        let new_cursor = handle.move_cursor(self.internal.lock().await.cursors[index], direction).await;
        self.internal.lock().await.cursors[index] = new_cursor;
        Ok(())
    }

    pub async fn place_mark(&self, index: usize) -> Result<(), Condition> {
        let handle = self.get_buffer_handle().await?;
        self.internal.lock().await.cursors[index] = handle.place_mark(self.internal.lock().await.cursors[index]).await;
        Ok(())
    }

    pub async fn remove_mark(&self, index: usize) -> Result<(), Condition> {
        let handle = self.get_buffer_handle().await?;
        self.internal.lock().await.cursors[index] = handle.remove_mark(self.internal.lock().await.cursors[index]).await;
        Ok(())
    }

    pub fn num_cursors(&self) -> usize {
        self.internal.blocking_lock().cursors.len()
    }

    pub fn remove_cursor(&self, index: usize) {
        self.internal.blocking_lock().cursors.remove(index);
    }

    pub fn get_cursor_position(&self, index: usize) -> (usize, usize) {
        let cursor = &self.internal.blocking_lock().cursors[index];
        (cursor.line(), cursor.column())
    }

    pub fn add_cursor(&self, line: usize, column: usize) {
        let mut index = 0;
        'outer: for (i, cursor) in self.internal.blocking_lock().cursors.iter().enumerate() {
            if line < cursor.line() {
                index = i;
            } else if line == cursor.line() {
                for (i, cursor) in self.internal.blocking_lock().cursors.iter().enumerate() {
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

        self.internal.blocking_lock().cursors.insert(index, Cursor::new(GridCursor::new(line, column)))
    }
}

impl SchemeCompatible for TextEditData {
    fn rtd() -> Arc<RecordTypeDescriptor>
    where
        Self: Sized
    {
        rtd!(name: "&TextEditData")
    }
}

pub fn get_data(major_mode: &Gc<MajorMode>) -> Result<Gc<TextEditData>, Condition> {
    let data = major_mode.read().data.clone();
    let data: Gc<TextEditData> = data.try_into_rust_type()?;
    Ok(data)
}

#[bridge(name = "text-edit-data-create", lib = "(text-edit)")]
pub fn create_text_edit_data(buffer_name: &Value) -> Result<Vec<Value>, Condition> {
    let buffer_name: String = buffer_name.clone().try_into()?;
    let data = TextEditData::new(buffer_name);
    
    Ok(vec![Value::from(Record::from_rust_type(data))])
}

#[bridge(name = "text-edit-draw", lib = "(text-edit)")]
pub async fn text_edit_draw(major_mode: &Value) -> Result<Vec<Value>, Condition> {
    let major_mode: Gc<MajorMode> = major_mode.clone().try_into_rust_type()?;
    let data = get_data(&major_mode)?;
    let data = data.read().clone();
    let buffer = {
        let buffer_name = data.internal.lock().await.buffer_name.clone();
        let state = SessionState::get_state();
        let guard = state.read().await;
        guard.get_buffers().await.get(&buffer_name).unwrap().clone()
    };

    let styled_text = buffer.get_styled_text(major_mode.clone(), &data.internal.lock().await.cursors).await;

    let value = Value::from(Record::from_rust_type(styled_text));
    Ok(vec![value])
}

#[bridge(name = "text-edit-move-cursor-up", lib = "(text-edit)")]
pub async fn move_cursor_up(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let Some((cursor_index, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let major_mode: Gc<MajorMode> = major_mode.clone().try_into_rust_type()?;
    let data = get_data(&major_mode)?;
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
    let data = data.read().clone();
    data.move_cursor(cursor_index, CursorDirection::Up).await?;
    Ok(Vec::new())
}

#[bridge(name = "text-edit-move-cursor-down", lib = "(text-edit)")]
pub async fn move_cursor_down(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let Some((cursor_index, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let major_mode: Gc<MajorMode> = major_mode.clone().try_into_rust_type()?;
    let data = get_data(&major_mode)?;
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
    let data = data.read().clone();
    data.move_cursor(cursor_index, CursorDirection::Down).await?;
    Ok(Vec::new())
}

#[bridge(name = "text-edit-move-cursor-left", lib = "(text-edit)")]
pub async fn move_cursor_left(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()))
    };
    let Some((cursor_index, rest)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()))
    };
    let Some((wrap, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let wrap: bool = wrap.clone().try_into()?;
    let major_mode: Gc<MajorMode> = major_mode.clone().try_into_rust_type()?;
    let data = get_data(&major_mode)?;
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
    let data = data.read().clone();
    data.move_cursor(cursor_index, CursorDirection::Left { wrap }).await?;
    Ok(Vec::new())
}

#[bridge(name = "text-edit-move-cursor-right", lib = "(text-edit)")]
pub async fn move_cursor_right(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()))
    };
    let Some((cursor_index, rest)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()))
    };
    let Some((wrap, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let wrap: bool = wrap.clone().try_into()?;
    let major_mode: Gc<MajorMode> = major_mode.clone().try_into_rust_type()?;
    let data = get_data(&major_mode)?;
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
    let data = data.read().clone();
    data.move_cursor(cursor_index, CursorDirection::Right { wrap }).await?;
    Ok(Vec::new())
}

#[bridge(name = "text-edit-place-cursor-mark", lib = "(text-edit)")]
pub async fn place_mark(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let Some((cursor_index, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
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
    let major_mode: Gc<MajorMode> = major_mode.clone().try_into_rust_type()?;
    let data = get_data(&major_mode)?;
    let data = data.read().clone();
    data.place_mark(cursor_index).await?;
    Ok(Vec::new())
}

#[bridge(name = "text-edit-remove-cursor-mark", lib = "(text-edit)")]
pub async fn remove_mark(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let Some((cursor_index, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
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
    let major_mode: Gc<MajorMode> = major_mode.clone().try_into_rust_type()?;
    let data = get_data(&major_mode)?;
    let data = data.read().clone();
    data.remove_mark(cursor_index).await?;
    Ok(Vec::new())
}


#[bridge(name = "text-edit-cursor-position", lib = "(text-edit)")]
pub fn get_cursor_position(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let Some((cursor_index, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
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
    let major_mode: Gc<MajorMode> = major_mode.clone().try_into_rust_type()?;
    let data = get_data(&major_mode)?;
    let data = data.read().clone();
    let (row, column): (usize, usize) = data.get_cursor_position(cursor_index);

    let pair = Value::new(UnpackedValue::Pair(Gc::new(Pair::new(Value::from(Number::from(row)), Value::from(Number::from(column))))));

    Ok(vec![pair])
}

#[bridge(name = "text-edit-cursor-create", lib = "(text-edit)")]
pub fn create_cursor(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let Some((pair, rest)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()));
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
                return Err(Condition::wrong_num_of_args(3, args.len()));
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
    let major_mode: Gc<MajorMode> = major_mode.clone().try_into_rust_type()?;
    let data = get_data(&major_mode)?;
    let data = data.read().clone();
    data.add_cursor(row, col);

    Ok(Vec::new())
}