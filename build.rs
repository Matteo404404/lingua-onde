// The Windows exe carries the app icon, so Explorer, the Start menu and
// shortcuts show it before the window opens.
fn main() {
    println!("cargo:rerun-if-changed=packaging/marea.ico");
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("packaging/marea.ico");
        res.compile().expect("the Windows icon resource");
    }
}
