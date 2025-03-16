use bewegrs::sfml::graphics::Font;
use bewegrs::ui::ComprehensiveElement;
use bewegrs::ui::elements::info::Info;
use bewegrs::{counters::Counters, sfml::window::VideoMode};
use criterion::{Criterion, criterion_group, criterion_main};

use stars::Stars;

fn bench_stars_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("stars_update");

    // Create test data
    let width = 1920;
    let height = 1080;
    let video = VideoMode::new(width, height, 24);

    let mut stars = Stars::new(video, 500_000, None).unwrap();
    stars.sort();

    let c = Counters::start(60).unwrap();
    let mut font = Font::new().unwrap();
    font.load_from_memory_static(include_bytes!("../../../resources/sansation.ttf"))
        .unwrap();
    let mut info = Info::new(&font, &video, &c);

    group.bench_function("stars_update", |b| {
        b.iter(|| stars.update(&c, &mut info));
    });

    group.finish();
}

criterion_group!(benches, bench_stars_update,);
criterion_main!(benches);
