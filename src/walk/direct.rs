use std::path::{Path, PathBuf};
use std::fs::{self, ReadDir, DirEntry};
use std::io;

pub struct MultiPackageWalker<'p> {
    path: &'p Path,
    package: Option<MultiPackage<'p>>,
}

pub struct MultiPackage<'p> {
    path: &'p Path,
    children: Vec<ReadDir>,
}

enum EntryType {
    File(PathBuf),
    Dir(ReadDir),
    Unknown,
}

impl<'p> MultiPackageWalker<'p> {
    pub fn new(path: &'p Path) -> io::Result<Self> {
        let package = Some(MultiPackage::new(path)?);
        Ok(Self { path, package })
    }
}

impl<'p> Iterator for MultiPackageWalker<'p> {
    type Item = MultiPackage<'p>;
    
    fn next(&mut self) -> Option<Self::Item> { 
        self.package.take()
    }
}

impl<'p> MultiPackage<'p> {
    pub fn new(path: &'p Path) -> io::Result<Self> {
        if !path.is_dir() {
            let err = io::Error::from(io::ErrorKind::InvalidInput);
            return Err(err);
        }

        let dir = fs::read_dir(path)?;
        let children = vec![dir];
        Ok(Self { path, children })
    }
}

impl Iterator for MultiPackage<'_> {
    type Item = io::Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.children.is_empty() {
            let dir = &mut self.children[0];
            let mut push: Option<ReadDir> = None;
            let mut package_empty = false;

            match dir.next() {
                Some(item) => {
                    if let Err(e) = item {
                        return Some(Err(e));
                    }

                    let entry_type = inspect_entry(item.unwrap());
                    if let Err(e) = entry_type {
                        return Some(Err(e));
                    }

                    match entry_type.unwrap() {
                        EntryType::Dir(dir) => push = Some(dir),
                        EntryType::File(path) => return Some(Ok(path)),
                        EntryType::Unknown => continue,
                    }
                },
                None => package_empty = true,
            };

            if package_empty {
                self.children.pop();
            }

            if let Some(dir) = push {
                self.children.push(dir);
            }
        }

        None
    }
}

impl super::Package for MultiPackage<'_> { }

fn inspect_entry(entry: DirEntry) -> io::Result<EntryType> {
    let file_type = entry.file_type();
    if let Err(e) = file_type {
        return Err(e);
    }

    let file_type = file_type.unwrap();
    if file_type.is_dir() {
        let read_dir = fs::read_dir(entry.path());
        if let Err(e) = read_dir {
            return Err(e);
        }

        return Ok(EntryType::Dir(read_dir.unwrap()));
    } else if file_type.is_file() {
        return Ok(EntryType::File(entry.path()));
    }

    Ok(EntryType::Unknown)
}
