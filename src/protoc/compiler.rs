use std::cell::{Ref, RefCell};
use std::ffi::OsStr;

#[derive(Debug)]
pub struct Compiler<'p, 'n> {
    path: &'p str,
    include_paths: Vec<&'p str>,
    plugin: Plugin<'n>,
    proto_paths: Vec<&'p str>,

    cached_options: RefCell<Option<String>>,
    cached_output: RefCell<Option<String>>,
}

#[derive(Debug)]
pub struct Plugin<'n> {
    name: &'n str,
    output: &'n str,
    options: Vec<&'n str>,
}

#[derive(Debug)]
pub enum Arg<'r, 's> {
    Ref(Ref<'r, str>),
    Str(&'s str),
}

impl<'p, 'n> Compiler<'p, 'n> {
    pub fn new(path: &'p str, plugin: Plugin<'n>) -> Self {
        let include_paths = vec![];
        let proto_paths = vec![];

        Self {
            path,
            include_paths,
            plugin,
            proto_paths,
            cached_options: RefCell::new(None),
            cached_output: RefCell::new(None),
        }
    }

    pub fn add_include(&mut self, path: &'p str) {
        self.include_paths.push(path);
    }

    pub fn add_proto(&mut self, path: &'p str) {
        self.proto_paths.push(path);
    }

    pub fn command(&self) -> Vec<Arg> {
        let mut buf = vec![];
        buf.reserve(self.include_paths.len() * 2 + self.proto_paths.len());

        buf.push(Arg::Str(self.path));
        for include in &self.include_paths {
            buf.push(Arg::Str("-I"));
            buf.push(Arg::Str(include));
        }

        buf.push(Arg::Ref(self.options()));
        buf.push(Arg::Ref(self.output()));
        for proto in &self.proto_paths {
            buf.push(Arg::Str(proto));
        }

        buf
    }

    fn options(&self) -> Ref<str> {
        if let Some(options) = self.take_ref(&self.cached_options) {
            return options;
        }

        self.put_into_cell(&self.cached_options, self.plugin.options_arg());
        self.take_ref(&self.cached_options).unwrap()
    }

    fn output(&self) -> Ref<str> {
        if let Some(output) = self.take_ref(&self.cached_output) {
            return output;
        }

        self.put_into_cell(&self.cached_output, self.plugin.output_arg());
        self.take_ref(&self.cached_output).unwrap()
    }

    fn take_ref<'a>(&'a self, cell: &'a RefCell<Option<String>>) -> Option<Ref<str>> {
        let borrow = cell.borrow();
        if borrow.is_none() {
            return None;
        }

        let taken = Ref::map(borrow, |b| b.as_ref().unwrap().as_str());
        Some(taken)
    }

    fn put_into_cell(&self, cell: &RefCell<Option<String>>, value: String) {
        let mut borrow = cell.borrow_mut();
        *borrow = Some(value);
    }
}

impl<'n> Plugin<'n> {
    pub fn new(name: &'n str, output: &'n str) -> Self {
        let options = vec![];
        Self {
            name,
            output,
            options,
        }
    }

    pub fn add_option(&mut self, option: &'n str) {
        self.options.push(option)
    }

    pub fn output_arg(&self) -> String {
        format!("--{}_out={}", self.name, self.output)
    }

    pub fn options_arg(&self) -> String {
        let options = self.options.join(",");
        format!("--{}_opt={}", self.name, options)
    }
}

impl<'r, 's> AsRef<OsStr> for Arg<'r, 's> {
    fn as_ref(&self) -> &OsStr {
        match self {
            Arg::Ref(r) => r.as_ref(),
            Arg::Str(s) => OsStr::new(s),
        }
    }
}
