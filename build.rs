fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();

        if cfg!(debug_assertions) {
            res.set_manifest(include_str!("./Debug.Manifest.xml"));
        } else {
            res.set_manifest(include_str!("./Manifest.xml"));
        }
        res.set_icon("src/resources/assets/icon.ico");
        res.compile().unwrap();
    }
}
