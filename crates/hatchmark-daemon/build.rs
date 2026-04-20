fn main() {
    #[cfg(windows)]
    {
        let icon = std::path::Path::new("resources/icon.ico");
        if icon.exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon("resources/icon.ico");
            if let Err(e) = res.compile() {
                eprintln!("winres failed to embed icon: {e}");
            }
        }
    }
}
