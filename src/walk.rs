pub mod deep;
pub mod directory;

use std::collections::HashSet;
use std::fs::{self, ReadDir};
use std::io;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::rc::Rc;

// type Walker = impl Iterator is not supported yet
pub trait Walker: Iterator<Item = io::Result<PathBuf>> {}

#[derive(Debug)]
pub struct PagingProtoWalker<F, W> {
    path: PathBuf,
    make_walker: F,
    content: Option<ReadDir>,
    exclude: Option<Rc<HashSet<PathBuf>>>,
    _fret: PhantomData<W>,
}

impl<F, W> PagingProtoWalker<F, W> {
    pub fn new<P: Into<PathBuf>>(path: P, make_walker: F) -> Self {
        Self {
            path: path.into(),
            make_walker,
            content: None,
            exclude: None,
            _fret: PhantomData,
        }
    }

    pub fn set_exclude<I, P>(&mut self, iter: I) -> io::Result<()>
    where
        I: Iterator<Item = P>,
        P: Into<PathBuf>,
    {
        let mut exclude = HashSet::new();
        match iter.size_hint() {
            (x, Some(y)) => exclude.reserve(y - x),
            _ => (),
        }

        for path in iter {
            let path = path.into();
            if !path.is_relative() {
                let err = io::Error::new(io::ErrorKind::InvalidInput, "related path expected");
                return Err(err);
            }

            let mut absolute_path = self.path.to_owned();
            absolute_path.push(path);
            exclude.insert(absolute_path);
        }

        self.exclude = Some(Rc::new(exclude));
        Ok(())
    }
}

impl<F, W> Iterator for PagingProtoWalker<F, W>
where
    F: Fn(PathBuf, Rc<HashSet<PathBuf>>) -> W,
    W: Walker,
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

        loop {
            match self.content.as_mut().and_then(|d| d.next())? {
                Err(e) => return Some(Err(e)),
                Ok(entry) => {
                    let file_type = match entry.file_type() {
                        Err(e) => return Some(Err(e)),
                        Ok(t) => t,
                    };

                    if !file_type.is_dir() {
                        continue;
                    }

                    let make = &self.make_walker;
                    let exclude = self.exclude.get_or_insert_with(|| Rc::new(HashSet::new()));
                    let walker = make(entry.path(), Rc::clone(exclude));
                    return Some(Ok(walker))
                }
            }
        }
    }
}

impl<F, W> Clone for PagingProtoWalker<F, W>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        PagingProtoWalker {
            path: self.path.clone(),
            make_walker: self.make_walker.clone(),
            content: None,
            exclude: self.exclude.clone(),
            _fret: self._fret.clone(),
        }
    }
}
