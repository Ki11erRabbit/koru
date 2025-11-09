use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::LazyLock;
use futures::future::BoxFuture;
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

pub static KORU_BUILTIN_PATH: &str = "scheme";
pub static KORU_USER_PATH: &str = ".";

pub async fn load_builtins() {
    let path = Path::new(KORU_BUILTIN_PATH);
    let runtime = SCHEME_RUNTIME.lock().await.take().unwrap();
    let path = path.canonicalize().unwrap();

    let mut futures = Vec::new();

    load_directory(path, &runtime, &mut futures);

    for f in futures {
        f.await;
    }

    *SCHEME_RUNTIME.lock().await = Some(runtime);
}

fn load_directory<'load, P: AsRef<Path>>(path: P, runtime: &'load Runtime, futures: &mut Vec<BoxFuture<'load, ()>>) -> bool {
    let mut loaded_file = false;
    for entry in path.as_ref().read_dir().unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_type = entry.file_type().unwrap();
        if file_type.is_dir() {
            loaded_file = loaded_file || load_directory(path, runtime, futures);
        } else if file_type.is_file() {
            if let Some(extension) = path.extension() {
                if extension != OsStr::new("scm") {
                    continue;
                }
            } else {
                continue;
            }
            
            let prog = Library::new_program(runtime, &path);
            let env = Environment::Top(prog);

            let file = std::fs::read_to_string(&path).unwrap();

            let file_name = entry.file_name().into_string().unwrap();

            let sexprs = Syntax::from_str(file.as_str(), Some(file_name.as_str())).unwrap();
            let span = Span::default();
            let future = Box::pin(async move {
                let env = env;
                let base = DefinitionBody::parse_lib_body(
                    &runtime,
                    &sexprs,
                    &env,
                    &span,
                ).await.unwrap();

                SchemeEnvs::put_environment(path, env).await;

                let compiled = base.compile_top_level();
                let proc = runtime.compile_expr(compiled).await;
                proc.call(&[]).await.unwrap();
            });

            futures.push(future);
            loaded_file = true;
        }
    }
    loaded_file
}

pub async fn load_user_config() -> bool {
    let path = Path::new(KORU_USER_PATH);
    if path.exists() {
        return false;
    }
    let runtime = SCHEME_RUNTIME.lock().await.take().unwrap();
    let path = path.canonicalize().unwrap();

    let mut futures = Vec::new();

    if !load_directory(path, &runtime, &mut futures) {
        drop(futures);
        *SCHEME_RUNTIME.lock().await = Some(runtime);
        return false;
    }

    for f in futures {
        f.await;
    }

    true
}