use tracing::trace;

pub use egui;
pub use egui_sfml;
pub use sfml;
pub use tracing;

pub mod counter;
pub mod errors;
pub mod graphic;
pub mod physics;
pub mod shapes;

pub fn setup(verbose: bool) {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if verbose {
            tracing::Level::TRACE
        } else {
            tracing::Level::INFO
        })
        .without_time()
        .with_file(false)
        .with_target(false)
        .with_writer(std::io::stderr)
        .finish();
    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber).expect("could not setup logger");
    trace!("set up the logger");
}
