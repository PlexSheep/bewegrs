use bewegrs::tracing::error;
use stars::stars;

fn main() {
    if let Err(e) = stars(std::env::args().collect::<Vec<_>>()) {
        error!("{e}")
    }
}
