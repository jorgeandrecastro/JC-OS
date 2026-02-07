use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use lazy_static::lazy_static;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NodeType {
    File,
    Directory,
}
#[allow(dead_code)]
pub struct Inode {
    pub uid: u32,
    pub permissions: u16,
    pub node_type: NodeType,
}

pub struct File {
    pub inode: Inode,
    pub data: Vec<u8>,
}

pub struct Directory {
    pub inode: Inode,
    pub entries: BTreeMap<String, FsNode>,
}

pub enum FsNode {
    File(File),
    Directory(Directory),
}

pub struct RamFileSystem {
    pub root: Directory,
    pub cwd: Vec<String>,
}

lazy_static! {
    pub static ref FS: Mutex<RamFileSystem> = Mutex::new(RamFileSystem::new());
}

impl RamFileSystem {
    pub fn new() -> Self {
        Self {
            root: Directory {
                inode: Inode { uid: 0, permissions: 0o755, node_type: NodeType::Directory },
                entries: BTreeMap::new(),
            },
            cwd: Vec::new(),
        }
    }

    fn get_current_dir(&self) -> &Directory {
        let mut curr = &self.root;
        for segment in &self.cwd {
            if let Some(FsNode::Directory(next_dir)) = curr.entries.get(segment) {
                curr = next_dir;
            }
        }
        curr
    }

    /// CORRECTION : Navigation mutable compatible avec le Borrow Checker Rust
    fn get_current_dir_mut(&mut self) -> &mut Directory {
        let mut curr = &mut self.root;
        for segment in self.cwd.iter() {
            // Cette astuce de "re-binding" permet de descendre sans bloquer les mutables
            let next = if let Some(FsNode::Directory(ref mut next_dir)) = curr.entries.get_mut(segment) {
                next_dir as *mut Directory
            } else {
                curr as *mut Directory
            };
            unsafe { curr = &mut *next; }
        }
        curr
    }

    pub fn ls(&self) -> Vec<(String, NodeType)> {
        let current_dir = self.get_current_dir();
        current_dir.entries.iter()
            .map(|(name, node)| {
                let t = match node {
                    FsNode::File(_) => NodeType::File,
                    FsNode::Directory(_) => NodeType::Directory,
                };
                (name.clone(), t)
            })
            .collect()
    }

    pub fn write_file(&mut self, name: &str, content: &str, uid: u32) -> Result<(), &str> {
        let current_dir = self.get_current_dir_mut();
        let data = Vec::from(content.as_bytes());
        
        let file_node = FsNode::File(File {
            inode: Inode { uid, permissions: 0o644, node_type: NodeType::File },
            data,
        });

        current_dir.entries.insert(name.to_string(), file_node);
        Ok(())
    }

    pub fn read_file(&self, name: &str) -> Option<String> {
        let current_dir = self.get_current_dir();
        if let Some(FsNode::File(f)) = current_dir.entries.get(name) {
            Some(String::from_utf8_lossy(&f.data).into_owned())
        } else {
            None
        }
    }

    pub fn remove_file(&mut self, name: &str) -> bool {
        let current_dir = self.get_current_dir_mut();
        current_dir.entries.remove(name).is_some()
    }

    pub fn mkdir(&mut self, name: &str, uid: u32) -> Result<(), &str> {
        let current_dir = self.get_current_dir_mut();
        if current_dir.entries.contains_key(name) {
            return Err("Le nom existe déjà");
        }
        let new_dir = FsNode::Directory(Directory {
            inode: Inode { uid, permissions: 0o755, node_type: NodeType::Directory },
            entries: BTreeMap::new(),
        });
        current_dir.entries.insert(name.to_string(), new_dir);
        Ok(())
    }

    pub fn cd(&mut self, path: &str) -> Result<(), &str> {
        match path {
            "/" => { self.cwd.clear(); Ok(()) },
            ".." => { self.cwd.pop(); Ok(()) },
            _ => {
                let current_dir = self.get_current_dir();
                if let Some(FsNode::Directory(_)) = current_dir.entries.get(path) {
                    self.cwd.push(path.to_string());
                    Ok(())
                } else {
                    Err("Dossier introuvable")
                }
            }
        }
    }

    pub fn get_stats(&self) -> (usize, usize) {
        fn traverse(dir: &Directory) -> (usize, usize) {
            let mut count = 0;
            let mut size = 0;
            for node in dir.entries.values() {
                match node {
                    FsNode::File(f) => { count += 1; size += f.data.len(); },
                    FsNode::Directory(d) => {
                        let (c, s) = traverse(d);
                        count += c; size += s;
                    }
                }
            }
            (count, size)
        }
        traverse(&self.root)
    }
}