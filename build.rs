use winres::WindowsResource;

fn main() {
    if cfg!(not(windows)) {
        panic!("This program only supports Windows.");
    }

    WindowsResource::new()
        .set_icon("icon.ico")
        .compile()
        .unwrap();
}
