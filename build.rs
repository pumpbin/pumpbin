fn build_capnp() {
    capnpc::CompilerCommand::new()
        .src_prefix("capnp")
        .file("capnp/plugin.capnp")
        .output_path("capnp")
        .run()
        .expect("schema compiler command");
}

fn main() {
    #[cfg(debug_assertions)]
    build_capnp();

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
