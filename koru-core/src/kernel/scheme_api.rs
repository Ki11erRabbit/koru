use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::LazyLock;
use futures::future::BoxFuture;
use log::info;
use scheme_rs::ast::DefinitionBody;
use scheme_rs::env::Environment;
use scheme_rs::cps::Compile;
use scheme_rs::registry::Library;
use scheme_rs::runtime::Runtime;
use scheme_rs::syntax::{Span, Syntax};
use tokio::sync::{Mutex, RwLock};

pub mod major_mode;
pub mod command;
pub mod session;
mod minor_mode;
mod modal;

pub static SCHEME_RUNTIME: LazyLock<Mutex<Option<Runtime>>> = LazyLock::new(|| {
    Mutex::new(Some(Runtime::new()))
});

static SCHEME_ENV: LazyLock<RwLock<SchemeEnvs>> = LazyLock::new(|| {
    RwLock::new(SchemeEnvs::new())
});

pub struct SchemeEnvs {
    envs: HashMap<String, Environment>,
}

impl SchemeEnvs {
    pub fn new() -> Self {
        SchemeEnvs {
            envs: HashMap::new(),
        }
    }

    pub async fn get_environment(path: &str) -> Option<Environment> {
        SCHEME_ENV.read().await.envs.get(path).map(Clone::clone)
    }

    pub async fn put_environment<P: AsRef<Path>>(path: P, environment: Environment) {
        let path = path.as_ref().as_os_str().to_string_lossy().to_string();
        SCHEME_ENV.write().await.envs.insert(path, environment);
    }
}

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
            
            let prog = Library::new_program(runtime, &path);
            let env = Environment::Top(prog);

            let file = std::fs::read_to_string(&path)?;

            let file_name = entry.file_name().into_string().unwrap();

            let sexprs = Syntax::from_str(file.as_str(), Some(file_name.as_str()))?;
            let span = Span::default();
            let future = Box::pin(async move {
                let env = env;
                let base = DefinitionBody::parse_lib_body(
                    &runtime,
                    &sexprs,
                    &env,
                    &span,
                ).await.map_err(|e| format!("{:?}", e))?;

                SchemeEnvs::put_environment(path, env).await;

                let compiled = base.compile_top_level();
                let proc = runtime.compile_expr(compiled).await;
                proc.call(&[]).await?;
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