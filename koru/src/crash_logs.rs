use tabled::Tabled;


#[derive(Tabled, Debug, Clone, Eq, PartialEq, Hash)]
pub struct CrashLog {
    level: String,
    timestamp: String,
    module_path: String,
    file: String,
    message: String,
}

impl CrashLog {
    pub fn new(level: String, timestamp: String, module_path: String, file: String, message: String) -> Self {
        CrashLog {
            level,
            timestamp,
            module_path,
            file,
            message,
        }
    }
}