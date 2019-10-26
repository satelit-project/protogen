mod deep;

use std::collections::HashSet;
use std::fs::{self, DirEntry, ReadDir};
use std::io;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

pub struct PagingProtoWalker<F, W> {
    path: PathBuf,
    make_walker: F,
    content: Option<ReadDir>,
    exclude: HashSet<PathBuf>,
    _fret: PhantomData<W>,
}

struct Directory {
    path: PathBuf,
    content: Option<ReadDir>,
}

enum EntryType {
    Proto(PathBuf),
    Dir(PathBuf),
    Unknown(PathBuf),
}

impl<F, W> PagingProtoWalker<F, W> {
    pub fn new<P: Into<PathBuf>>(path: P, make_walker: F) -> Self {
        Self {
            path: path.into(),
            make_walker,
            content: None,
            exclude: HashSet::new(),
            _fret: PhantomData,
        }
    }

    pub fn exclude<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        if !path.as_ref().is_relative() {
            let err = io::Error::new(io::ErrorKind::InvalidInput, "related path expected");
            return Err(err);
        }

        let mut absolute_path = self.path.to_owned();
        absolute_path.push(path);
        self.exclude.insert(absolute_path);

        Ok(())
    }
}

impl<F, W> Iterator for PagingProtoWalker<F, W>
where
    W: Iterator<Item = io::Result<PathBuf>>,
    F: for<'e> Fn(PathBuf, &'e HashSet<PathBuf>) -> W,
{
    type Item = io::Result<W>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.content.is_none() {
            let dir = self.path.canonicalize().and_then(|p| fs::read_dir(p));

            match dir {
                Err(e) => return Some(Err(e)),
                Ok(c) => self.content = Some(c),
            };
        }

        match self.content.as_mut().and_then(|d| d.next())? {
            Err(e) => return Some(Err(e)),
            Ok(entry) => {
                let make = &self.make_walker;
                let walker = make(entry.path(), &self.exclude);
                Some(Ok(walker))
            }
        }
    }
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
