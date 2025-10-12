use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::{Arc, LazyLock};
use mlua::{AnyUserData, UserData, UserDataMethods};
use tokio::io::AsyncReadExt;
use tokio::sync::{Mutex, RwLock};
use crate::kernel::files::open_file::OpenFile;


static OPEN_FILES: LazyLock<RwLock<OpenFileTable>> = LazyLock::new(|| {
    RwLock::new(OpenFileTable::new())
});


pub struct OpenFileTable {
    table: Vec<Option<Arc<Mutex<OpenFile>>>>,
    free_list: VecDeque<usize>,
    /// A mapping from absolute path to index into table
    path_to_index: HashMap<String, usize>
}

impl OpenFileTable {
    pub fn new() -> Self {
        OpenFileTable {
            table: Vec::new(),
            free_list: VecDeque::new(),
            path_to_index: HashMap::new()
        }
    }

    fn insert(&mut self, buffer: String, path: impl AsRef<Path>) {
        let file = OpenFile::new(buffer, path.as_ref());
        let file = Arc::new(Mutex::new(file));
        let path = path.as_ref();
        let path = path.to_string_lossy();
        if let Some(index) = self.free_list.pop_front() {
            self.table[index] = Some(file);
            self.path_to_index.insert(path.to_string(), index);
            return;
        }
        let index = self.table.len();

        self.table.push(Some(file));
        self.path_to_index.insert(path.to_string(), index);
    }

    pub async fn open_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let path = path.canonicalize()?;
        let file_contents = {
            let mut contents = String::new();
            let mut file = tokio::fs::File::open(&path).await?;
            file.read_to_string(&mut contents).await?;
            contents
        };

        self.insert(file_contents, path);

        Ok(())
    }

    pub async fn close_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let path = path.canonicalize()?;
        let path = path.to_string_lossy().to_string();
        let Some(index) = self.path_to_index.get(&path) else {
            return Err(Box::from(String::from("File not found")));
        };
        self.close(*index).await?;
        Ok(())
    }

    pub async fn close(&mut self, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        let Some(file) = self.table[index].take() else {
            return Err(Box::from(String::from("File not found")));
        };
        let file = file.lock().await;
        let path = file.absolute_path();
        let path = path.to_string_lossy().to_string();
        self.path_to_index.remove(&path);
        self.free_list.push_back(index);

        Ok(())
    }

    pub async fn save_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let path = path.canonicalize()?;
        let path = path.to_string_lossy().to_string();
        let Some(index) = self.path_to_index.get(&path) else {
            return Err(Box::from(String::from("File not found")));
        };
        let Some(file) = &self.table[*index] else {
            panic!("file was closed without clearing out data")
        };
        file.lock().await.save().await?;
        Ok(())
    }

    pub fn get_handle<P: AsRef<Path>>(&self, path: P) -> Result<OpenFileHandle, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let path = path.canonicalize()?;
        let path = path.to_string_lossy().to_string();
        let Some(index) = self.path_to_index.get(&path) else {
            return Err(Box::from(String::from("File not found")));
        };
        let Some(file) = &self.table[*index] else {
            panic!("file was closed without clearing out data")
        };
        Ok(OpenFileHandle::new(file.clone(), *index))
    }

    pub async fn open_or_get_handle<P: AsRef<Path>>(&mut self, path: P) -> Result<OpenFileHandle, Box<dyn std::error::Error>> {
        match self.get_handle(path.as_ref()) {
            Ok(handle) => Ok(handle),
            Err(_) => {
                self.open_file(path.as_ref()).await?;
                self.get_handle(path)
            }
        }
    }
}


pub struct OpenFileHandle {
    file: Arc<Mutex<OpenFile>>,
    index: usize,
}

impl OpenFileHandle {
    fn new(file: Arc<Mutex<OpenFile>>, index: usize) -> Self {
        Self { file, index }
    }

    pub async fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.file.lock().await.save().await?;
        Ok(())
    }

    pub async fn close(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut open_files = OPEN_FILES.write().await;
        open_files.close(self.index).await?;
        Ok(())
    }
    
    pub async fn get_text(&self) -> String {
        self.file.lock().await.get_text()
    }
}

impl UserData for OpenFileHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_function(
            "save",
            async |_, (this,): (AnyUserData, )| {
                let this = this.borrow::<OpenFileHandle>()?;
                this.save().await.unwrap();
                Ok(())
            }
        );
        methods.add_async_function(
            "close",
            async |_, (this,): (AnyUserData, )| {
                let this = this.borrow::<OpenFileHandle>()?;
                this.close().await.unwrap();
                Ok(())
            }
        );
    }
}

pub async fn open_file<P: AsRef<Path>>(path: P) -> Result<OpenFileHandle, Box<dyn std::error::Error>> {
    let mut open_files = OPEN_FILES.write().await;
    open_files.open_file(path.as_ref()).await?;
    open_files.get_handle(path)
}

pub async fn get_handle<P: AsRef<Path>>(path: P) -> Result<OpenFileHandle, Box<dyn std::error::Error>> {
    let open_files = OPEN_FILES.read().await;
    open_files.get_handle(path)
}

pub async fn open_or_get_handle<P: AsRef<Path>>(path: P) -> Result<OpenFileHandle, Box<dyn std::error::Error>> {
    let mut open_files = OPEN_FILES.write().await;
    open_files.open_or_get_handle(path).await
}