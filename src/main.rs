use egui_sfml::SfEgui;
use sfml::{
    graphics::{
        glsl::Vec2, CircleShape, Color, CustomShape, Font, RectangleShape, RenderTarget,
        RenderWindow, Shape, Transformable,
    },
    system::{Vector2f, Vector2i},
    window::{Event, Key, Style, VideoMode},
    SfResult,
};

use self::{
    counters::{Counters, MAX_FPS},
    shapes::{hue_time, TriangleShape},
    ui::elements::{clickeable::Clickable, UIElement},
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
    let mut gui = SfEgui::new(&window);

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

    let mut message = String::new();
    let mut messages = Vec::new();

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

        counter.tick();
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

        // Step 3: Do an egui frame with the desired ui function
        let di = gui
            .run(&mut window, |_rw, ctx| {
                let win = egui::Window::new("Hello egui-sfml!");
                win.show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Message");
                        let te_re = ui.text_edit_singleline(&mut message);
                        if ui.button("Send").clicked()
                            || ui.input(|inp| inp.key_pressed(egui::Key::Enter))
                        {
                            messages.push(message.clone());
                            te_re.request_focus();
                            message.clear();
                        }
                    });
                    for msg in &messages {
                        ui.separator();
                        ui.label(msg);
                    }
                });
            })
            .unwrap();
        // Step 4: Draw

        window.draw(&backdrop);
        window.draw(&circle);
        window.draw(&triangle);
        window.draw(&clicker);
        gui.draw(di, &mut window, None);

        counter.tick_done();
        window.display();
    }
    Ok(())
}
