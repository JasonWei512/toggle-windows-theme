use winres::WindowsResource;

fn main() {
    WindowsResource::new()
        .set_icon("icon.ico")
        .compile()
        .unwrap();
}
