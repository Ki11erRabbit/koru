use std::sync::Arc;
use scheme_rs::exceptions::Exception;
use scheme_rs::gc::{Gc, Trace};
use scheme_rs::lists::Pair;
use scheme_rs::num::Number;
use scheme_rs::records::{rtd, Record, RecordTypeDescriptor, SchemeCompatible};
use scheme_rs::registry::bridge;
use scheme_rs::value::{UnpackedValue, Value};
use tokio::sync::Mutex;
use crate::kernel::buffer::{BufferHandle, Cursor, CursorDirection, Cursors, GridCursor};
use crate::kernel::input::{KeyPress, KeyValue, ModifierKey};
use crate::kernel::scheme_api::major_mode::{MajorMode};
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
    
    async fn get_buffer_handle(&self) -> Result<BufferHandle, Exception> {
        let state = SessionState::get_state();
        let guard = state.read().await;
        let buffer_guard=  guard.get_buffers().await;
        let Some(buffer) = buffer_guard.get(&self.internal.lock().await.buffer_name) else {
            return Err(Exception::error(String::from("Buffer not found")));
        };
        let handle = buffer.get_handle();
        Ok(handle)
    }
    pub async fn move_cursor(&self, index: usize, direction: CursorDirection) -> Result<(), Exception> {
        let handle = self.get_buffer_handle().await?;
        let new_cursor = handle.move_cursor(self.internal.lock().await.cursors[index], direction).await;
        self.internal.lock().await.cursors[index] = new_cursor;
        Ok(())
    }

    pub async fn place_mark(&self, index: usize) -> Result<(), Exception> {
        let handle = self.get_buffer_handle().await?;
        let new_cursor = handle.place_mark(self.internal.lock().await.cursors[index]).await;
        self.internal.lock().await.cursors[index] = new_cursor;
        Ok(())
    }

    pub async fn remove_mark(&self, index: usize) -> Result<(), Exception> {
        let handle = self.get_buffer_handle().await?;
        let new_cursor = handle.remove_mark(self.internal.lock().await.cursors[index]).await;
        self.internal.lock().await.cursors[index] = new_cursor;
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

    pub async fn change_main_cursor(&self, index: usize) {
        let mut guard = self.internal.lock().await;
        for cursor in guard.cursors.iter_mut() {
            cursor.unset_main()
        }
        guard.cursors[index].set_main()
    }

    pub async fn get_main_cursor(&self) -> Cursor{
        let mut guard = self.internal.lock().await;
        for cursor in guard.cursors.iter_mut() {
            if cursor.is_main() {
                return *cursor;
            }
        }
        unreachable!("There should always be a main cursor")
    }

    pub async fn get_cursor(&self, index: usize) -> Cursor {
        self.internal.lock().await.cursors[index].clone()
    }
    
    pub async fn get_cursors(&self) -> Vec<Cursor> {
        let guard = self.internal.lock().await;
        guard.cursors.clone()
    }
    
    pub async fn set_cursors(&self, cursors: Vec<Cursor>) {
        let mut guard = self.internal.lock().await;
        guard.cursors = cursors;
    }
}

impl SchemeCompatible for TextEditData {
    fn rtd() -> Arc<RecordTypeDescriptor>
    where
        Self: Sized
    {
        rtd!(name: "&TextEditData", sealed: true)
    }
}

pub async fn get_data(major_mode: &Gc<MajorMode>) -> Result<Gc<TextEditData>, Exception> {
    let data = major_mode.data.read().await.clone();
    let data: Gc<TextEditData> = data.try_to_rust_type()?;
    Ok(data)
}

#[bridge(name = "text-edit-data-create", lib = "(text-edit)")]
pub fn create_text_edit_data(buffer_name: &Value) -> Result<Vec<Value>, Exception> {
    let buffer_name: String = buffer_name.clone().try_into()?;
    let data = TextEditData::new(buffer_name);
    
    Ok(vec![Value::from(Record::from_rust_type(data))])
}

#[bridge(name = "text-edit-get-cursors", lib = "(text-edit)")]
pub async fn get_cursors(text_edit_data: &Value) -> Result<Vec<Value>, Exception> {
    let text_edit_data: Gc<TextEditData> = text_edit_data.clone().try_to_rust_type()?;
    let cursors = text_edit_data.internal.lock().await.cursors.clone();
    let cursors = Cursors { cursors };
    let value = Record::from_rust_type(cursors);
    Ok(vec![Value::from(value)])
}

#[bridge(name = "text-edit-get-buffer-name", lib = "(text-edit)")]
pub async fn get_buffer_name(text_edit_data: &Value) -> Result<Vec<Value>, Exception> {
    let text_edit_data: Gc<TextEditData> = text_edit_data.clone().try_to_rust_type()?;
    let buffer_name = text_edit_data.internal.lock().await.buffer_name.clone();
    Ok(vec![Value::from(buffer_name)])
}

#[bridge(name = "text-edit-draw", lib = "(text-edit)")]
pub async fn text_edit_draw(major_mode: &Value) -> Result<Vec<Value>, Exception> {
    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let data = get_data(&major_mode).await?;
    let buffer = {
        let buffer_name = data.internal.lock().await.buffer_name.clone();
        let state = SessionState::get_state();
        let mut guard = state.write().await;
        let mut buffers = guard.get_buffers_mut().await;
        let buffer = buffers.get_mut(&buffer_name).ok_or(Exception::error(String::from("Buffer does not exist")))?;
        buffer.render_styled_text().await;
        buffer.clone()
    };

    let styled_text = buffer.get_styled_text(&data.internal.lock().await.cursors);

    let value = Value::from(Record::from_rust_type(styled_text));
    Ok(vec![value])
}

#[bridge(name = "text-edit-move-cursor-up", lib = "(text-edit)")]
pub async fn move_cursor_up(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let Some((cursor_index, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let data = get_data(&major_mode).await?;
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index = cursor_index.as_ref().try_into()?;
    data.move_cursor(cursor_index, CursorDirection::Up).await?;
    Ok(Vec::new())
}

#[bridge(name = "text-edit-move-cursor-down", lib = "(text-edit)")]
pub async fn move_cursor_down(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let Some((cursor_index, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let data = get_data(&major_mode).await?;
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index = cursor_index.as_ref().try_into()?;
    data.move_cursor(cursor_index, CursorDirection::Down).await?;
    Ok(Vec::new())
}

#[bridge(name = "text-edit-move-cursor-left", lib = "(text-edit)")]
pub async fn move_cursor_left(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()))
    };
    let Some((cursor_index, rest)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()))
    };
    let Some((wrap, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let wrap: bool = wrap.clone().try_into()?;
    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let data = get_data(&major_mode).await?;
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index = cursor_index.as_ref().try_into()?;
    data.move_cursor(cursor_index, CursorDirection::Left { wrap }).await?;
    Ok(Vec::new())
}

#[bridge(name = "text-edit-move-cursor-right", lib = "(text-edit)")]
pub async fn move_cursor_right(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()))
    };
    let Some((cursor_index, rest)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()))
    };
    let Some((wrap, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let wrap: bool = wrap.clone().try_into()?;
    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let data = get_data(&major_mode).await?;
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index = cursor_index.as_ref().try_into()?;
    data.move_cursor(cursor_index, CursorDirection::Right { wrap }).await?;
    Ok(Vec::new())
}

#[bridge(name = "text-edit-place-point-mark-at-cursor", lib = "(text-edit)")]
pub async fn place_mark(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let Some((cursor_index, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index = cursor_index.as_ref().try_into()?;
    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let data = get_data(&major_mode).await?;
    data.place_mark(cursor_index).await?;
    Ok(Vec::new())
}

#[bridge(name = "text-edit-remove-mark-from-cursor", lib = "(text-edit)")]
pub async fn remove_mark(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let Some((cursor_index, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index = cursor_index.as_ref().try_into()?;
    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let data = get_data(&major_mode).await?;
    data.remove_mark(cursor_index).await?;
    Ok(Vec::new())
}


#[bridge(name = "text-edit-cursor-position", lib = "(text-edit)")]
pub async fn get_cursor_position(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let Some((cursor_index, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index = cursor_index.as_ref().try_into()?;
    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let data = get_data(&major_mode).await?;
    let (row, column): (usize, usize) = data.get_cursor_position(cursor_index);

    let pair = Value::new(UnpackedValue::Pair(Pair::new(Value::from(Number::from(row)), Value::from(Number::from(column)), false)));

    Ok(vec![pair])
}

#[bridge(name = "text-edit-cursor-create", lib = "(text-edit)")]
pub async fn create_cursor(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let Some((pair, rest)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()));
    };
    let (row, col) = match pair.clone().unpack() {
        UnpackedValue::Pair(pair) => {
            let left = pair.car().clone();
            let right = pair.cdr().clone();
            let row: Arc<Number> = left.try_into()?;
            let col: Arc<Number> = right.try_into()?;
            let col: usize = col.as_ref().try_into()?;
            let row: usize = row.as_ref().try_into()?;
            (row, col)
        }
        UnpackedValue::Number(row) => {
            let Some((col, _)) = rest.split_first() else {
                return Err(Exception::wrong_num_of_args(3, args.len()));
            };
            let col: Arc<Number> = col.clone().try_into()?;
            let row: usize = row.as_ref().try_into()?;
            let col: usize = col.as_ref().try_into()?;
            (row, col)
        }
        ty => return Err(Exception::type_error("Pair or Integer", ty.type_name()))
    };
    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let data = get_data(&major_mode).await?;
    data.add_cursor(row, col);

    Ok(Vec::new())
}

#[bridge(name = "text-edit-cursor-destroy", lib = "(text-edit)")]
pub async fn destroy_cursor(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let Some((index, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()));
    };
    let index: Arc<Number> = index.clone().try_into()?;
    let index: usize = index.as_ref().try_into()?;
    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let data = get_data(&major_mode).await?;
    data.remove_cursor(index);

    Ok(Vec::new())
}

#[bridge(name = "text-edit-cursor-count", lib = "(text-edit)")]
pub async fn get_cursor_count(major_mode: &Value) -> Result<Vec<Value>, Exception> {
    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let data: Gc<TextEditData> = major_mode.data.read().await.clone().try_to_rust_type()?;
    
    let cursor_count = data.num_cursors();
    
    let out = Value::from(Number::from(cursor_count));
    
    Ok(vec![out])
}

#[bridge(name = "text-edit-cursor-change-main", lib = "(text-edit)")]
pub async fn change_main_cursor(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let Some((index, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()));
    };
    let index: Arc<Number> = index.clone().try_into()?;
    let index: usize = index.as_ref().try_into()?;
    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let data = get_data(&major_mode).await?;
    data.change_main_cursor(index).await;
    Ok(Vec::new())
}

#[bridge(name = "text-edit-get-main-cursor", lib = "(text-edit)")]
pub async fn get_main_cursor(major_mode: &Value) -> Result<Vec<Value>, Exception> {
    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let data = get_data(&major_mode).await?;
    let cursor = data.get_main_cursor().await;
    let cursor = Value::from(Record::from_rust_type(cursor));
    Ok(vec![cursor])
}

pub async fn insert_text_at_cursor(
    major_mode: Gc<MajorMode>,
    cursor_index: usize,
    text: String
) -> Result<(), Exception> {
    let data = get_data(&major_mode).await?;
    let cursors = data.get_cursors().await;
    let handle: BufferHandle = data.get_buffer_handle().await?;
    let new_cursors = handle.insert(text, cursor_index, cursors).await;
    data.set_cursors(new_cursors).await;
    Ok(())
}

#[bridge(name = "text-edit-insert-at-cursor", lib = "(text-edit)")]
pub async fn insert_text(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()))
    };
    let Some((cursor_index, rest)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let Some((text, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };

    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index: usize = cursor_index.as_ref().try_into()?;
    match text.clone().try_into() {
        Ok(text) => {
            let text: String = text;
            insert_text_at_cursor(major_mode.clone(), cursor_index, text).await?;
            return Ok(Vec::new())
        }
        _ => {}
    }
    match text.clone().try_into() {
        Ok(letter) => {
            let letter: char = letter;
            insert_text_at_cursor(major_mode.clone(), cursor_index, letter.to_string()).await?;
            Ok(Vec::new())
        }
        _ => {
            Err(Exception::type_error("String or char", text.type_name()))
        }
    }
}

#[bridge(name = "text-edit-delete-before-cursor", lib = "(text-edit)")]
pub async fn delete_text_back(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let Some((cursor_index, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()));
    };

    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index: usize = cursor_index.as_ref().try_into()?;
    let data = get_data(&major_mode).await?;
    let cursors = data.get_cursors().await;
    let handle: BufferHandle = data.get_buffer_handle().await?;
    let new_cursors = handle.delete_back(cursor_index, cursors).await;
    data.set_cursors(new_cursors).await;
    Ok(Vec::new())
}

#[bridge(name = "text-edit-delete-after-cursor", lib = "(text-edit)")]
pub async fn delete_text_forward(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let Some((cursor_index, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()));
    };

    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index: usize = cursor_index.as_ref().try_into()?;
    let data = get_data(&major_mode).await?;
    let cursors = data.get_cursors().await;
    let handle: BufferHandle = data.get_buffer_handle().await?;
    let new_cursors = handle.delete_forward(cursor_index, cursors).await;
    data.set_cursors(new_cursors).await;
    Ok(Vec::new())
}

#[bridge(name = "text-edit-delete-region-cursor", lib = "(text-edit)")]
pub async fn delete_text_region(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let Some((cursor_index, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()));
    };

    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index: usize = cursor_index.as_ref().try_into()?;
    let data = get_data(&major_mode).await?;
    let cursors = data.get_cursors().await;
    let handle: BufferHandle = data.get_buffer_handle().await?;
    let new_cursors = handle.delete_region(cursor_index, cursors).await;
    data.set_cursors(new_cursors).await;
    Ok(Vec::new())
}

#[bridge(name = "text-edit-replace-text", lib = "(text-edit)")]
pub async fn replace_text(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()))
    };
    let Some((cursor_index, rest)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let Some((text, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };

    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index: usize = cursor_index.as_ref().try_into()?;
    match text.clone().try_into() {
        Ok(text) => {
            let text: String = text;
            let data = get_data(&major_mode).await?;
            let cursors = data.get_cursors().await;
            let handle: BufferHandle = data.get_buffer_handle().await?;
            let new_cursors = handle.replace(text, cursor_index, cursors).await;
            data.set_cursors(new_cursors).await;
            return Ok(Vec::new());
        }
        _ => {}
    }
    match text.clone().try_into() {
        Ok(letter) => {
            let letter: char = letter;
            let data = get_data(&major_mode).await?;
            let cursors = data.get_cursors().await;
            let handle: BufferHandle = data.get_buffer_handle().await?;
            let new_cursors = handle.replace(letter.to_string(), cursor_index, cursors).await;
            data.set_cursors(new_cursors).await;
            Ok(Vec::new())
        }
        _ => {
            Err(Exception::type_error("String or char", text.type_name()))
        }
    }
}

#[bridge(name = "text-edit-undo", lib = "(text-edit)")]
pub async fn undo(major_mode: &Value) -> Result<Vec<Value>, Exception> {
    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let data = get_data(&major_mode).await?;
    let handle: BufferHandle = data.get_buffer_handle().await?;
    handle.undo().await;
    Ok(Vec::new())
}

#[bridge(name = "text-edit-redo", lib = "(text-edit)")]
pub async fn redo(major_mode: &Value) -> Result<Vec<Value>, Exception> {
    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let data = get_data(&major_mode).await?;
    let handle: BufferHandle = data.get_buffer_handle().await?;
    handle.redo().await;
    Ok(Vec::new())
}

#[bridge(name = "text-edit-insert-keypress", lib = "(text-edit)")]
pub async fn insert_keypress(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((major_mode, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let Some((cursor_index, rest)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let Some((key_sequence, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let cursor_index: Arc<Number> = cursor_index.clone().try_into()?;
    let cursor_index: usize = cursor_index.as_ref().try_into()?;
    let key_press = {
        let key_sequence = key_sequence.clone().unpack();
        match key_sequence {
            UnpackedValue::Pair(pair) => {
                let cdr = pair.cdr().clone();
                if !cdr.is_null() {
                    // Skip if the key sequence is 2 or greater
                    return Ok(vec![Value::from(false)]);
                }
                let key = pair.car().clone();
                let key: Gc<KeyPress> = key.try_to_rust_type()?;
                (*key).clone()
            }
            _ => {
                return Err(Exception::type_error("List", key_sequence.type_name()))
            }
        }
    };

    if !key_press.modifiers.is_empty() {
        return Ok(vec![Value::from(false)]);
    }

    match &key_press.key {
        KeyValue::CharacterKey(c) => {
            insert_text_at_cursor(major_mode, cursor_index, c.to_string()).await?;
        }
        _ => {
            return Ok(vec![Value::from(false)]);
        }
    }

    Ok(vec![Value::from(true)])
}

#[bridge(name = "text-edit-start-transaction", lib = "(text-edit)")]
pub async fn start_transaction(major_mode: &Value) -> Result<Vec<Value>, Exception> {
    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let data = get_data(&major_mode).await?;
    let handle: BufferHandle = data.get_buffer_handle().await?;
    handle.start_transaction().await;
    Ok(Vec::new())
}

#[bridge(name = "text-edit-end-transaction", lib = "(text-edit)")]
pub async fn end_transaction(major_mode: &Value) -> Result<Vec<Value>, Exception> {
    let major_mode: Gc<MajorMode> = major_mode.clone().try_to_rust_type()?;
    let data = get_data(&major_mode).await?;
    let handle: BufferHandle = data.get_buffer_handle().await?;
    handle.end_transaction().await;
    Ok(Vec::new())
}