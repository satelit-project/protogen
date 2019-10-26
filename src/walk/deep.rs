use std::collections::HashSet;
use std::io;
use std::path::PathBuf;

use super::{Directory, EntryType};

pub struct DeepProtoWalker<'e> {
    children: Vec<Directory>,
    exclude: &'e HashSet<PathBuf>,
}

impl<'e> DeepProtoWalker<'e> {
    pub fn new<P: Into<PathBuf>>(path: P, exclude: &'e HashSet<PathBuf>) -> Self {
        let children = vec![Directory::new(path)];
        Self { children, exclude }
    }

    fn should_skip(&self, path: &PathBuf) -> bool {
        self.exclude.contains(path)
    }
}

impl Iterator for DeepProtoWalker<'_> {
    type Item = io::Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.children.is_empty() {
            let package = &mut self.children[0];
            let mut push_package: Option<PathBuf> = None;
            let mut package_empty = false;

            match package.next() {
                Some(Ok(entry)) => match entry {
                    EntryType::Dir(path) => {
                        if !self.should_skip(&path) {
                            push_package = Some(path)
                        }
                    }
                    EntryType::Proto(path) => {
                        if !self.should_skip(&path) {
                            return Some(Ok(path));
                        }
                    }
                    _ => continue,
                },
                Some(Err(e)) => return Some(Err(e)),
                None => package_empty = true,
            };

            if package_empty {
                self.children.pop();
            }

            if let Some(path) = push_package {
                self.children.push(Directory::new(path));
            }
        }

        None
    }
}
