use std::error::Error;
use koru_core::{koru_main_ui, koru_main_ui_start_runtime};
use crate::iced_backend::true_main;

mod iced_backend;
mod tuirelm_backend;
mod common;

fn main() -> Result<(), Box<dyn Error>> {
    //koru_main_ui(true_main)
    koru_main_ui_start_runtime(tuirelm_backend::real_main)
}