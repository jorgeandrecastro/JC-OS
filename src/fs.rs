// fs.rs - Basic RAM File System for JC-OS
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use lazy_static::lazy_static;

/// Structure representing a single file in memory
pub struct File {
    pub name: String,
    pub data: Vec<u8>,
}

pub struct RamFileSystem {
    pub files: BTreeMap<String, File>,
}

lazy_static! {
    /// Global instance of the File System protected by a Mutex
    pub static ref FS: Mutex<RamFileSystem> = Mutex::new(RamFileSystem {
        files: BTreeMap::new(),
    });
}

impl RamFileSystem {
    /// Create or overwrite a file with string content
    pub fn write_file(&mut self, name: &str, content: &str) {
        let file = File {
            name: String::from(name),
            data: Vec::from(content.as_bytes()),
        };
        self.files.insert(String::from(name), file);
    }

    /// Read file content as a String
    pub fn read_file(&self, name: &str) -> Option<String> {
        self.files.get(name).map(|f| {
            String::from_utf8_lossy(&f.data).into_owned()
        })
    }

    /// Return a list of all filenames
    pub fn list_files(&self) -> Vec<String> {
        self.files.keys().cloned().collect()
    }
}