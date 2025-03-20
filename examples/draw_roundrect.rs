use bewegrs::{setup, shapes::RectRoundShape};
use sfml::{
    SfResult,
    graphics::{
        CircleShape, Color, CustomShape, CustomShapePoints, RenderTarget, RenderWindow, Shape,
        Transformable,
    },
    system::{Time, Vector2f, sleep},
    window::{Event, Key, Style, VideoMode},
};
const R: f32 = 200.0;

#[derive(Clone, Copy)]
pub struct TriangleShape;

impl CustomShapePoints for TriangleShape {
    fn point_count(&self) -> usize {
        3
    }

    fn point(&self, point: usize) -> Vector2f {
        match point {
            0 => Vector2f { x: 20., y: 580. },
            1 => Vector2f { x: 400., y: 20. },
            2 => Vector2f { x: 780., y: 580. },
            p => panic!("Non-existent point: {p}"),
        }
    }
}

fn main() -> SfResult<()> {
    setup(true);
    let video = VideoMode::desktop_mode();
    let mut window = RenderWindow::new(video, "Custom shape", Style::DEFAULT, &Default::default())?;

    let center: Vector2f = (video.width as f32 / 2.0, video.height as f32 / 2.0).into();

    let mut shape = RectRoundShape::new(400.0, 200.0, 20.0);
    shape.set_position(center);
    shape.set_origin((400. / 2.0, 200. / 2.0));
    shape.set_fill_color(Color::RED);

    'mainloop: loop {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed
                | Event::KeyPressed {
                    code: Key::Escape, ..
                } => break 'mainloop,
                _ => (),
            }
        }

        window.clear(Color::BLACK);
        window.draw(&shape);
        window.display();
    }

    Ok(())
}
