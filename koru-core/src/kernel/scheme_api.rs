use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::LazyLock;
use futures::future::BoxFuture;
use log::info;
use scheme_rs::env::TopLevelEnvironment;
use scheme_rs::runtime::Runtime;
use tokio::sync::Mutex;

pub mod major_mode;
pub mod command;
pub mod session;
mod minor_mode;
mod modal;
pub mod theme;

pub static SCHEME_RUNTIME: LazyLock<Mutex<Option<Runtime>>> = LazyLock::new(|| {
    Mutex::new(Some(Runtime::new()))
});
pub static KORU_USER_PATH: &str = ".";


fn load_directory<'load, P: AsRef<Path>>(
    path: P, 
    runtime: &'load Runtime, 
    futures: &mut Vec<BoxFuture<'load, Result<(), Box<dyn Error>>>>,
    no_recurse: bool
) -> Result<bool, Box<dyn Error>> {
    let mut loaded_file = false;
    for entry in path.as_ref().read_dir()? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if no_recurse {
                continue;
            }
            loaded_file = loaded_file || load_directory(path, runtime, futures, no_recurse)?;
        } else if file_type.is_file() {
            if let Some(extension) = path.extension() {
                if extension != OsStr::new("scm") {
                    continue;
                }
            } else {
                continue;
            }
            
            info!("Loading {:?}", path);

            let future = Box::pin(async move {
                let env = TopLevelEnvironment::new_repl(runtime);
                env.import("(library (rnrs))".parse().unwrap()).await.unwrap();
                let contents = tokio::fs::read_to_string(path).await?;
                env.eval(true, &contents).await.unwrap();
                Ok(())
            });

            futures.push(future);
            loaded_file = true;
        }
    }
    Ok(loaded_file)
}

pub async fn load_user_config() -> Result<bool, Box<dyn Error>> {
    let path = Path::new(KORU_USER_PATH);
    let path = path.canonicalize()?;
    if !path.exists() {
        return Ok(false);
    }
    let runtime = SCHEME_RUNTIME.lock().await.take().unwrap();

    let mut futures = Vec::new();

    if !load_directory(path, &runtime, &mut futures, true)? {
        drop(futures);
        *SCHEME_RUNTIME.lock().await = Some(runtime);
        return Ok(false);
    }

    for f in futures {
        f.await?;
    }

    Ok(true)
}