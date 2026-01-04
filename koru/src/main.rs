use std::error::Error;
use log::info;
use koru_core::{koru_main_ui, koru_main_ui_start_runtime, KoruArgs, KoruLogger};

mod iced_backend;
mod tuirealm_backend;
mod common;
mod ui_state;
mod crash_logs;

fn main() -> Result<(), Box<dyn Error>> {
    KoruArgs::parse_args();
    let logger_capacity = KoruArgs::get_log_capacity()?;
    KoruLogger::install_logger(logger_capacity);
    if KoruArgs::get_tui() {
        koru_main_ui_start_runtime(tuirealm_backend::real_main)
    } else {
        koru_main_ui(iced_backend::true_main)
    }
}