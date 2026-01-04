use std::cell::OnceCell;
use std::sync::OnceLock;
use clap::Parser;

static COMMAND_LINE_ARGUMENTS: OnceLock<Args> = OnceLock::new();

#[derive(Parser, Debug, Default)]
pub struct Args {
    files: Vec<String>,
    /// Indicates to use the tui version rather than the iced frontend.
    #[clap(short, long)]
    tui: bool,
    /// A numeric value indicating how many of a particular kind of log should be stored.
    #[clap(short, long, default_value = "1000")]
    log_capacity: String,
}

impl Args {
    /// Parses the commandline arguments for Koru's runtime
    ///
    /// This should be called before starting the kernel
    pub fn parse_args() {
        let args = Args::parse();
        COMMAND_LINE_ARGUMENTS.set(args).expect("Args::parse_args() was called twice");
    }
    
    fn get_args() -> &'static Args {
        COMMAND_LINE_ARGUMENTS.get().expect("Args::parse_args() was not called yet")
    }
    
    pub fn get_files() -> Vec<String> {
        let args = Args::get_args();
        args.files.clone()
    }
    
    pub fn get_tui() -> bool {
        let args = Args::get_args();
        args.tui
    }
    
    pub fn get_log_capacity() -> Result<usize, Box<dyn std::error::Error>> {
        let args = Args::get_args();
        let capacity = args.log_capacity.parse()?;
        Ok(capacity)
    }
    
    
}
