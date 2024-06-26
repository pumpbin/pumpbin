fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("logo/icon.ico");
        res.compile().unwrap();
    }

    #[cfg(target_os = "macos")]
    {
        use std::fs;

        let version = env!("CARGO_PKG_VERSION");
        fs::write("VERSION", version).unwrap();
    }
}
