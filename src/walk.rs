mod deep;

use std::fs::{self, DirEntry, ReadDir};
use std::io;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

// TODO: support excludes
pub struct PagingProtoWalker<'p, F, W> {
    path: &'p Path,
    make_walker: F,
    content: Option<ReadDir>,
    _fret: PhantomData<W>,
}

pub struct Directory {
    path: PathBuf,
    content: Option<ReadDir>,
}

pub enum EntryType {
    Proto(PathBuf),
    Dir(PathBuf),
    Unknown(PathBuf),
}

impl<'p, F, W> PagingProtoWalker<'p, F, W> {
    pub fn new(path: &'p Path, make_walker: F) -> Self {
        Self {
            path,
            make_walker,
            content: None,
            _fret: PhantomData,
        }
    }
}

impl<F, W> Iterator for PagingProtoWalker<'_, F, W>
where
    W: Iterator<Item = io::Result<PathBuf>>,
    F: Fn(PathBuf) -> W,
{
    type Item = io::Result<W>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.content.is_none() {
            match fs::read_dir(self.path) {
                Err(e) => return Some(Err(e)),
                Ok(c) => self.content = Some(c),
            };
        }

        match self.content.as_mut().and_then(|d| d.next())? {
            Err(e) => return Some(Err(e)),
            Ok(entry) => {
                let make = &self.make_walker;
                let walker = make(entry.path());
                Some(Ok(walker))
            }
        }
    }
}

impl Directory {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        let path = path.into();
        Self {
            path,
            content: None,
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
        Some(inspect_entry(&entry))
    }
}

fn inspect_entry(entry: &DirEntry) -> io::Result<EntryType> {
    let file_type = entry.file_type();
    if let Err(e) = file_type {
        return Err(e);
    }

    let file_type = file_type.unwrap();
    let path = entry.path();

    if file_type.is_dir() {
        return Ok(EntryType::Dir(path));
    }

    if !file_type.is_file() {
        return Ok(EntryType::Unknown(path));
    }

    let filename = entry.file_name();
    match filename.to_str().map_or(false, |n| n.ends_with(".proto")) {
        true => Ok(EntryType::Proto(path)),
        false => Ok(EntryType::Unknown(path)),
    }
}
