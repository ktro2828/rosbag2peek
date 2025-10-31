use app::App;
use backend::ReaderBackend;

pub mod app;
pub mod backend;

pub use backend::create_reader;
use rospeek_core::RosPeekResult;

pub fn spawn_app() -> RosPeekResult<()> {
    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "rospeek-app",
        native_options,
        Box::new(|cc| Ok(Box::new(App::<ReaderBackend>::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {e}"))
}
