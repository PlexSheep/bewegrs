use bewegrs::counter::Counter;
use bewegrs::graphic::ComprehensiveElement;
use bewegrs::graphic::elements::info::Info;
use bewegrs::sfml::graphics::Font;
use bewegrs::sfml::window::VideoMode;
use criterion::{Criterion, criterion_group, criterion_main};

use stars::{DEFAULT_STAR_RADIUS, Stars};

fn bench_stars_new(c: &mut Criterion) {
    let mut group = c.benchmark_group("stars_new");

    // Create test data
    let width = 1920;
    let height = 1080;
    let video = VideoMode::new(width, height, 24);

    group.bench_function("stars_new", |b| {
        b.iter(|| Stars::new(video, 100_000, None, 60, DEFAULT_STAR_RADIUS).unwrap());
    });

    group.finish();
}

fn bench_stars_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("stars_update");

    // Create test data
    let width = 1920;
    let height = 1080;
    let video = VideoMode::new(width, height, 24);

    let mut stars = Stars::new(video, 1_000_000, None, 60, DEFAULT_STAR_RADIUS).unwrap();
    stars.sort(0);

    let mut c = Counter::start(60).unwrap();
    let mut font = Font::new().unwrap();
    font.load_from_memory_static(include_bytes!("../../../resources/sansation.ttf"))
        .unwrap();
    let mut info = Info::new(&font, &video, &c);

    group.bench_function("stars_update", |b| {
        b.iter(|| {
            c.frame_start();
            stars.update(&c, &mut info);
            c.frame_prepare_display();
        })
    });

    group.finish();
}

criterion_group!(benches, bench_stars_new, bench_stars_update,);
criterion_main!(benches);
