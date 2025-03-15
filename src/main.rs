use sfml::{
    graphics::{CircleShape, Color, CustomShape, RenderTarget, RenderWindow, Shape, Transformable},
    system::{sleep, Time},
    window::{Event, Key, Style, VideoMode},
    SfResult,
};

use self::{
    counters::{Counters, MAX_FPS},
    shapes::{hue_time, TriangleShape},
};

pub mod counters;
pub mod shapes;

fn main() -> SfResult<()> {
    let mut window = RenderWindow::new(
        VideoMode::new(1000, 600, 1),
        "Custom shape",
        Style::DEFAULT,
        &Default::default(),
    )?;
    let mut counter = Counters::start()?;
    window.set_framerate_limit(MAX_FPS);

    let mut triangle = CustomShape::new(Box::new(TriangleShape));
    triangle.set_position((400., 300.));
    triangle.set_origin((400., 300.));
    triangle.set_outline_thickness(3.);

    let mut circle = CircleShape::new(100.0, 32);
    circle.set_position((400., 300.));
    circle.set_origin((400., 300.));
    circle.set_outline_thickness(8.);
    circle.set_outline_color(Color::RED);

    'mainloop: loop {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed
                | Event::KeyPressed {
                    code: Key::Escape, ..
                } => break 'mainloop,
                _ => {}
            }
        }

        counter.tick();
        let scale = counter.seconds.cos().abs();

        triangle.set_rotation(counter.seconds.sin().abs() * 360.0);
        triangle.set_scale(scale);
        triangle.set_fill_color(hue_time(counter.seconds));
        triangle.set_outline_color(hue_time(counter.seconds / 2.0));

        circle.set_scale(scale);
        circle.set_outline_color(Color::RED);

        window.clear(Color::BLACK);

        window.draw(&circle);
        window.draw(&triangle);

        counter.tick_done();
        window.display();
    }
    Ok(())
}
