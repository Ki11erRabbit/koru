use std::error::Error;
use koru_core::{koru_main_ui, koru_main_ui_start_runtime, parse_cmdline_arguments};

mod iced_backend;
mod tuirealm_backend;
mod common;
mod ui_state;

fn main() -> Result<(), Box<dyn Error>> {
    parse_cmdline_arguments();
    koru_main_ui(iced_backend::true_main)
    //koru_main_ui_start_runtime(tuirealm_backend::real_main)
}