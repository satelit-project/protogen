use std::ffi::OsString;

#[derive(Debug)]
pub struct Compiler {
    path: OsString,
    include_paths: Vec<OsString>,
    plugin: Plugin,
    proto_paths: Vec<OsString>,
}

#[derive(Debug)]
pub struct Plugin {
    name: String,
    path: OsString,
    output: OsString,
    options: Vec<String>,
}

impl Compiler {
    pub fn new(path: OsString, plugin: Plugin) -> Self {
        let include_paths = vec![];
        let proto_paths = vec![];

        Self {
            path,
            include_paths,
            plugin,
            proto_paths,
        }
    }

    pub fn add_include(&mut self, path: OsString) {
        self.include_paths.push(path.into());
    }

    pub fn add_proto(&mut self, path: OsString) {
        self.proto_paths.push(path.into());
    }

    pub fn command(self) -> Vec<OsString> {
        let mut buf = vec![];
        buf.reserve(self.include_paths.len() * 2 + self.proto_paths.len());

        buf.push(self.path);
        for include in self.include_paths {
            buf.push("-I".into());
            buf.push(include);
        }

        buf.append(&mut self.plugin.args());
        for proto in self.proto_paths {
            buf.push(proto);
        }

        buf
    }
}

impl Plugin {
    pub fn new(name: String, path: OsString, output: OsString) -> Self {
        let options = vec![];
        Self {
            name,
            path,
            output,
            options,
        }
    }

    pub fn add_option(&mut self, option: String) {
        self.options.push(option)
    }

    pub fn args(self) -> Vec<OsString> {
        let options = self.options.join(",").into();
        let mut plugin = OsString::new();
        plugin.push(format!("protoc-gen-{}=", self.name));
        plugin.push(self.path);

        vec![
            format!("--{}_out", self.name).into(),
            self.output,
            format!("--{}_opt", self.name).into(),
            options,
            OsString::from("--plugin"),
            plugin
        ]
    }
}
