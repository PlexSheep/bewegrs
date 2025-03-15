use sfml::{
    graphics::{
        glsl::Vec2, CircleShape, Color, CustomShape, Font, RectangleShape, RenderTarget,
        RenderWindow, Shape, Transformable,
    },
    system::{Vector2f, Vector2i},
    window::{Event, Key, Style, VideoMode},
    SfResult,
};
use tracing::trace;

use self::{
    counters::{Counters, MAX_FPS},
    shapes::{hue_time, TriangleShape},
    ui::{
        nativeui::elements::{clickeable::Clickable, NativeElement},
        ComprehensiveUi,
    },
};

pub mod counters;
pub mod shapes;
pub mod ui;

fn setup() {
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

fn main() -> SfResult<()> {
    setup();

    let video = VideoMode::fullscreen_modes()[0];
    let mut window = RenderWindow::new(
        video.clone(),
        "Custom shape",
        Style::DEFAULT | Style::FULLSCREEN,
        &Default::default(),
    )?;
    let mut counter = Counters::start()?;
    window.set_framerate_limit(MAX_FPS as u32);

    let mut font = Font::new()?;
    font.load_from_memory_static(include_bytes!("../resources/sansation.ttf"))?;

    let mut gui = ComprehensiveUi::build(&window, &font, video.clone())?;

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
    backdrop.set_size(Vec2::new(video.width as f32, video.height as f32));
    backdrop.set_fill_color(Color::rgb(30, 20, 20));

    let mut clicker = Clickable::new_rect_round(430.0, 240.0, 32.0);
    clicker.shape.set_position((200., 300.));
    clicker.shape.set_outline_thickness(3.);
    clicker = clicker.with_text("hello world", &font, 64);

    'mainloop: loop {
        while let Some(event) = window.poll_event() {
            gui.add_event(&event);
            match event {
                Event::Closed
                | Event::KeyPressed {
                    code: Key::Escape, ..
                } => break 'mainloop,
                Event::MouseButtonPressed { button: _, x, y } => {
                    let mouse_pos: Vector2i = (x, y).into();
                    let mouse_posf: Vector2f = (x as f32, y as f32).into();
                    if clicker.contains_point(mouse_posf) {
                        clicker.handle_event(&event, mouse_pos);
                    }
                }
                _ => (),
            }
        }

        counter.frame_start();

        if counter.frames % MAX_FPS == 1 {
            gui.update_slow(&counter)
        }

        let scale = counter.seconds.cos().abs();

        triangle.set_rotation(counter.seconds.sin().abs() * 360.0);
        triangle.set_scale(scale);
        triangle.set_fill_color(hue_time(counter.seconds));
        triangle.set_outline_color(hue_time(counter.seconds / 2.0));

        circle.set_scale(scale);
        circle.set_outline_color(Color::RED);

        if counter.seconds > 10.0 {
            clicker.set_position((300.0, 100.0));
        }

        window.clear(Color::BLACK);

        window.draw(&backdrop);
        window.draw(&circle);
        window.draw(&triangle);
        window.draw(&clicker);
        gui.draw_with(&mut window, &counter);

        counter.frame_prepare_display();
        window.display();
    }
    Ok(())
}
