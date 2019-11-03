use std::fs::{self, DirEntry, ReadDir};
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Directory {
    path: PathBuf,
    content: Option<ReadDir>,
}

pub enum EntryType {
    Proto(PathBuf),
    Dir(PathBuf),
    Unknown(PathBuf),
}

impl Directory {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
            content: None,
        }
    }

    fn inspect_entry(&self, entry: &DirEntry) -> io::Result<EntryType> {
        let file_type = entry.file_type()?;
        let path = entry.path();

        if file_type.is_dir() {
            return Ok(EntryType::Dir(path));
        } else if !file_type.is_file() {
            return Ok(EntryType::Unknown(path));
        }

        let filename = entry.file_name();
        match filename.to_str().map_or(false, |n| n.ends_with(".proto")) {
            true => Ok(EntryType::Proto(path)),
            false => Ok(EntryType::Unknown(path)),
        }
    }
}

impl Iterator for Directory {
    type Item = io::Result<EntryType>;

    fn next(&mut self) -> Option<Self::Item> {
        if let None = self.content {
            match fs::read_dir(&self.path) {
                Err(e) => return Some(Err(e)),
                Ok(c) => self.content = Some(c),
            }
        }

        let entry = self.content.as_mut().and_then(|c| c.next())?;
        if let Err(e) = entry {
            return Some(Err(e));
        }

        let entry = entry.unwrap();
        Some(self.inspect_entry(&entry))
    }
}
