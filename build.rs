#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    let rc_dir = r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64";
    if std::path::Path::new(rc_dir).exists() {
        res.set_toolkit_path(rc_dir);
    }
    res.set_icon("tide.ico");
    let _ = res.compile();
}

#[cfg(not(windows))]
fn main() {}
