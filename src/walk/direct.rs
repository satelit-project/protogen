use std::path::PathBuf;
use std::io;

use super::{Package, EntryType};

pub struct MultiPackageWalker {
    children: Vec<Package>,
}

impl MultiPackageWalker {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        let children = vec![Package::new(path)];
        Self { children }
    }
}

impl Iterator for MultiPackageWalker {
    type Item = io::Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.children.is_empty() {
            let package = &mut self.children[0];
            let mut push_package: Option<PathBuf> = None;
            let mut package_empty = false;

            match package.next() {
                Some(Ok(entry)) => {
                    match entry {
                        EntryType::Dir(path) => push_package = Some(path),
                        EntryType::Proto(path) => return Some(Ok(path)),
                        EntryType::Unknown(_) => continue,
                    }
                },
                Some(Err(e)) => return Some(Err(e)),
                None => package_empty = true,
            };

            if package_empty {
                self.children.pop();
            }

            if let Some(path) = push_package {
                self.children.push(Package::new(path));
            }
        }

        None
    }
}
