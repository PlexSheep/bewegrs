use tracing::trace;

pub mod counters;
pub mod shapes;
pub mod ui;

pub fn setup() {
    // construct a subscriber that prints formatted traces to stdout
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(
            #[cfg(debug_assertions)]
            tracing::Level::TRACE,
            #[cfg(not(debug_assertions))]
            tracing::Level::INFO,
        )
        .without_time()
        .with_file(false)
        .with_target(false)
        .with_writer(std::io::stderr)
        .finish();
    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber).expect("could not setup logger");
    trace!("set up the logger");
}
