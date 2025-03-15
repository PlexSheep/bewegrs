use sfml::{
    graphics::{
        glsl::Vec2, CircleShape, Color, CustomShape, Font, RectangleShape, RenderTarget,
        RenderWindow, Shape, Transformable,
    },
    window::{Event, Key, Style, VideoMode},
    SfResult,
};

use self::{
    counters::{Counters, MAX_FPS},
    shapes::{hue_time, RectRoundShape, TriangleShape},
    ui::elements::clickeable::Clickable,
};

pub const WINDOW_WIDTH: u32 = 1000;
pub const WINDOW_HEIGHT: u32 = 600;

pub mod counters;
pub mod shapes;
pub mod ui;

fn main() -> SfResult<()> {
    let video = VideoMode::new(WINDOW_WIDTH, WINDOW_HEIGHT, 16);

    let mut window = RenderWindow::new(video, "Custom shape", Style::DEFAULT, &Default::default())?;
    let mut counter = Counters::start()?;
    window.set_framerate_limit(MAX_FPS);

    let mut font = Font::new()?;
    font.load_from_memory_static(include_bytes!("../resources/sansation.ttf"))?;

    let mut triangle = CustomShape::new(Box::new(TriangleShape));
    triangle.set_position((400., 300.));
    triangle.set_origin((400., 300.));
    triangle.set_outline_thickness(3.);

    let mut circle = CircleShape::new(100.0, 32);
    circle.set_position((400., 300.));
    circle.set_origin((400., 300.));
    circle.set_outline_thickness(8.);
    circle.set_outline_color(Color::RED);

    let mut backdrop = RectangleShape::new();
    backdrop.set_size(Vec2::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32));
    backdrop.set_fill_color(Color::rgb(30, 20, 20));

    let mut clicker = Clickable::new_rect_round(430.0, 240.0, 32.0);
    clicker.shape.set_position((200., 300.));
    clicker.shape.set_outline_thickness(3.);
    clicker = clicker.with_text("hello world", &font, 64);

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

        window.draw(&backdrop);
        window.draw(&circle);
        window.draw(&triangle);
        window.draw(&clicker);

        counter.tick_done();
        window.display();
    }
    Ok(())
}
