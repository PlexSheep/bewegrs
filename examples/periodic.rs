use sfml::{
    SfResult,
    graphics::{
        CircleShape, Color, CustomShape, Font, RectangleShape, RenderTarget, RenderWindow, Shape,
        Transformable, glsl::Vec2,
    },
    window::{Event, Key, Style, VideoMode},
};
use tracing::info;

use bewegrs::{
    counters::Counters,
    setup,
    shapes::{TriangleShape, hue_time},
    ui::ComprehensiveUi,
};

const MAX_FPS: u64 = 60;

fn main() -> SfResult<()> {
    setup();

    let video = VideoMode::fullscreen_modes()[0];
    info!("video mode: {video:?}");
    let mut window = RenderWindow::new(
        video,
        "Custom shape",
        Style::DEFAULT | Style::FULLSCREEN,
        &Default::default(),
    )?;
    let mut counter = Counters::start(MAX_FPS)?;
    window.set_framerate_limit(MAX_FPS as u32);

    let mut font = Font::new()?;
    font.load_from_memory_static(include_bytes!("../resources/sansation.ttf"))?;

    let mut gui = ComprehensiveUi::build(&window, &font, &video, &counter)?;

    let mut triangle = CustomShape::new(Box::new(TriangleShape));
    triangle.set_position((400., 300.));
    triangle.set_origin((400., 300.));
    triangle.set_outline_thickness(3.);

    let mut circle = CircleShape::new(100.0, 32);
    circle.set_position((800., 900.));
    circle.set_origin((400., 300.));
    circle.set_outline_thickness(8.);
    circle.set_outline_color(Color::RED);

    let mut backdrop = RectangleShape::new();
    backdrop.set_size(Vec2::new(video.width as f32, video.height as f32));
    backdrop.set_fill_color(Color::rgb(30, 20, 20));

    'mainloop: loop {
        while let Some(event) = window.poll_event() {
            gui.add_event(&event);
            match event {
                Event::Closed
                | Event::KeyPressed {
                    code: Key::Escape, ..
                } => break 'mainloop,
                _ => (),
            }
        }

        counter.frame_start();

        gui.update(&counter);
        if counter.frames % MAX_FPS as u64 == 1 {
            gui.update_slow(&counter)
        }

        let scale = counter.seconds.cos().abs();

        triangle.set_rotation(counter.seconds.sin().abs() * 360.0);
        triangle.set_scale(scale);
        triangle.set_fill_color(hue_time(counter.seconds));
        triangle.set_outline_color(hue_time(counter.seconds / 2.0));

        circle.set_scale(scale);
        circle.set_outline_color(Color::RED);

        window.clear(Color::BLACK);

        window.draw(&backdrop);
        window.draw(&circle);
        window.draw(&triangle);
        gui.draw_with(&mut window, &counter);

        counter.frame_prepare_display();
        window.display();
    }
    Ok(())
}
