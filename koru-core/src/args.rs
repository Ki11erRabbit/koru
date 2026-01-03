use std::cell::OnceCell;
use std::sync::OnceLock;
use clap::Parser;

static COMMAND_LINE_ARGUMENTS: OnceLock<Args> = OnceLock::new();

#[derive(Parser, Debug, Default)]
pub struct Args {
    files: Vec<String>,
}

impl Args {
    pub fn parse_args() {
        let args = Args::parse();
        COMMAND_LINE_ARGUMENTS.set(args).expect("Args::parse_args() was called twice");
    }
    pub fn get_files(&self) -> Vec<String> {
        self.files.clone()
    }
}
