use std::sync::{Arc, Mutex};
use scrollable_rich::rich::VisibleTextMetrics;

/// Stores the ui's state for individual buffers.
#[derive(Clone, Default)]
pub struct BufferState {
    /// The name of the buffer to load
    pub buffer_name: String,
    /// The amount of lines from the start of the buffer to request
    pub line_offset: usize,
    /// The amount of columns from the left side to load from
    pub column_offset: usize,
    /// The text metrics associated with the open buffer
    pub text_metrics: Arc<Mutex<VisibleTextMetrics>>,
}

impl BufferState {

    pub fn text_metrics_callback<'a>(&self) -> impl Fn(VisibleTextMetrics) + 'a {
        let text_metrics = self.text_metrics.clone();
        move |new: VisibleTextMetrics| {
            *text_metrics.lock().expect("lock poisoned") = new;
        }
    }
}