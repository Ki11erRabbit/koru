use std::error::Error;
use koru_core::{koru_main_ui, koru_main_ui_start_runtime};

mod iced_backend;
mod tuirealm_backend;
mod common;

fn main() -> Result<(), Box<dyn Error>> {
    koru_main_ui(iced_backend::true_main)
    //koru_main_ui_start_runtime(tuirealm_backend::real_main)
}