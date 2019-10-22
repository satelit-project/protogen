use std::path::PathBuf;

pub fn push_binary_path(path: &mut PathBuf) {
    path.push("bin");
    path.push(binary_name());
}

pub fn push_include_path(path: &mut PathBuf) {
    path.push("include")
}

pub fn target_platform() -> &'static str {
    platform()
}

#[cfg(target_os = "windows")]
fn binary_name() -> &'static str {
    "protoc.exe"
}

#[cfg(not(target_os = "windows"))]
fn binary_name() -> &'static str {
    "protoc"
}

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
fn platform() -> &'static str {
    "win64"
}

#[cfg(all(target_os = "windows", target_arch = "x86"))]
fn platform() -> &'static str {
    "win32"
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn platform() -> &'static str {
    "linux-x86_64"
}

#[cfg(all(target_os = "linux", target_arch = "x86"))]
fn platform() -> &'static str {
    "linux-x86_32"
}

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
fn platform() -> &'static str {
    "osx-x86_64"
}

#[cfg(all(target_os = "macos", target_arch = "x86"))]
fn platform() -> &'static str {
    "osx-x86_32"
}
