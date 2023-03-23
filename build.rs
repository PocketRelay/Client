fn main() {
    if cfg!(target_os = "windows") && !cfg!(debug_assertions) {
        let mut res = winres::WindowsResource::new();
        res.set_manifest(include_str!("./Manifest.xml"));
        res.set_icon("src/resources/assets/icon.ico");
        res.compile().unwrap();
    }
}
