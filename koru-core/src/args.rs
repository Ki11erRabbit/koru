use std::error::Error;
use std::sync::{OnceLock, RwLock};
use clap::Parser;

static COMMAND_LINE_ARGUMENTS: OnceLock<Args> = OnceLock::new();

#[derive(Parser, Debug, Default)]
pub struct ArgsInternal {
    files: Vec<String>,
    /// Indicates to use the tui version rather than the iced frontend.
    #[clap(short, long)]
    tui: bool,
    /// Indicates to use the iced frontend rather than the tui frontend. This will override the tui option.
    #[clap(short, long)]
    gui: bool,
    /// A numeric value indicating how many of a particular kind of log should be stored.
    #[clap(short, long, default_value = "1000")]
    log_capacity: String,
}


#[derive(Debug, Default)]
pub struct Args {
    files: RwLock<Option<Vec<String>>>,
    /// Indicates to use the tui version rather than the iced frontend.
    tui: bool,
    /// A numeric value indicating how many of a particular kind of log should be stored.
    log_capacity: usize,
}

impl Args {
    /// Parses the commandline arguments for Koru's runtime
    ///
    /// This should be called before starting the kernel
    pub fn parse_args() -> Result<(), Box<dyn Error>> {
        let args = ArgsInternal::parse();

        let tui = if args.gui {
            false
        } else {
            args.tui
        };

        let args = Args {
            files: RwLock::new(Some(args.files)),
            tui,
            log_capacity: args.log_capacity.parse()?,
        };
        COMMAND_LINE_ARGUMENTS.set(args).expect("Args::parse_args() was called twice");
        Ok(())
    }
    
    fn get_args() -> &'static Args {
        COMMAND_LINE_ARGUMENTS.get().expect("Args::parse_args() was not called yet")
    }
    
    pub fn get_files() -> Option<Vec<String>> {
        let args = Args::get_args();
        args.files.write()
            .expect("lock poisoned")
            .take()
    }
    
    pub fn get_tui() -> bool {
        let args = Args::get_args();
        args.tui
    }
    
    pub fn get_log_capacity() -> usize {
        let args = Args::get_args();
        let capacity = args.log_capacity;
        capacity
    }
    
    
}
