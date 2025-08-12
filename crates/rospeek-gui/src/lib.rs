use app::App;
use backend::ReaderBackend;

pub mod app;
pub mod backend;

pub fn spawn_app() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "rospeek-app",
        native_options,
        Box::new(|cc| Ok(Box::new(App::<ReaderBackend>::new(cc)))),
    )
}
