use std::error::Error;
use koru_core::koru_main_ui;
use crate::iced_backend::true_main;

mod iced_backend;



fn main() -> Result<(), Box<dyn Error>> {
    koru_main_ui(true_main)
}