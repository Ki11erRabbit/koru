use log::info;
use koru_core::styled_text::StyledFile;

/// Stores the ui's state for individual buffers.
#[derive(Clone, Default)]
pub struct BufferState {
    /// The name of the buffer to load
    pub buffer_name: String,
    /// The amount of lines from the start of the buffer to request
    pub line_offset: usize,
    /// The amount of columns from the left side to load from
    pub column_offset: usize,
    pub line_count: usize,
    pub column_count: usize,
    /// The styled text of the buffer
    pub text: StyledFile,
    /// The column of the main cursor
    pub col: usize,
    /// The line of the main cursor
    pub line: usize,
}

impl BufferState {


    pub fn scroll_view(&mut self) {
        self.scroll_horizontal();
        self.scroll_vertical();
    }

    fn scroll_vertical(&mut self) {
        let line_count = self.line_count;
        while self.line < self.line_offset {
            self.line_offset -= 1;
        }
        while self.line > self.line_offset + line_count - 1 {
            self.line_offset += 1;
        }
    }

    fn scroll_horizontal(&mut self) {
        let column_count = self.column_count;
        info!("before:\n\tcolumn_count: {} column_offset: {}", column_count, self.column_offset);
        while self.col < self.column_offset {
            self.column_offset -= 1;
        }
        while self.col > self.column_offset + column_count - 1 {
            self.column_offset += 1;
        }
        info!("after:\n\tcolumn_count: {} column_offset: {}", column_count, self.column_offset);
    }
}