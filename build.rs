fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("logo/icon.ico");
        res.compile().unwrap();
    }
}
