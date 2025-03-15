use sfml::{
    cpp::FBox,
    graphics::{
        Color, CustomShape, CustomShapePoints, RenderTarget, RenderWindow, Shape, Transformable,
    },
    system::{Clock, Vector2f},
    window::{Event, Key, Style, VideoMode},
    SfResult,
};

const MAX_FPS: u32 = 60;

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

fn hue_time(t: f32) -> Color {
    const fn lerp(from: f32, to: f32, amount: f32) -> f32 {
        from + amount * (to - from)
    }

    let frac = t.fract();

    let [r, g, b] = match (t % 6.0).floor() {
        0.0 => [255., lerp(0., 255., frac), 0.],
        1.0 => [lerp(255., 0., frac), 255., 0.],
        2.0 => [0., 255., lerp(0., 255., frac)],
        3.0 => [0., lerp(255., 0., frac), 255.],
        4.0 => [lerp(0., 255., frac), 0., 255.],
        _ => [255., 0., lerp(255., 0., frac)],
    };
    Color::rgb(r as u8, g as u8, b as u8)
}

/// lazy fields get updated every [FRAME_MAX] frames
#[derive(Debug)]
struct Counters {
    /// frame counter
    pub frames: u64,
    /// frame counter lazy
    pub lframes: u64,
    /// seconds counter
    pub seconds: f32,
    /// seconds counter lazy
    pub lseconds: f32,

    /// actually keeps track of time
    pub clock: FBox<Clock>,
}

impl Counters {
    fn start() -> SfResult<Self> {
        Ok(Counters {
            clock: Clock::start()?,
            lframes: 0,
            frames: 0,
            seconds: 0.0,
            lseconds: 0.0,
        })
    }
    fn tick(&mut self) {
        self.seconds = self.clock.elapsed_time().as_seconds();
        self.frames += 1;

        if self.frames % MAX_FPS as u64 == 0 {
            println!("seconds: {:.2}", self.seconds);
            println!("frames: {}", self.frames);
            println!("FPS: {}", self.fps());
            self.lseconds = self.seconds;
            self.lframes = self.frames;
        }
    }
    fn dframes(&self) -> u64 {
        self.frames - self.lframes
    }
    fn dseconds(&self) -> f32 {
        self.seconds - self.lseconds
    }
    fn fps(&self) -> f32 {
        self.dframes() as f32 / self.dseconds()
    }
}

fn main() -> SfResult<()> {
    let mut window = RenderWindow::new(
        VideoMode::new(1000, 600, 1),
        "Custom shape",
        Style::DEFAULT,
        &Default::default(),
    )?;
    let mut counter = Counters::start()?;
    window.set_framerate_limit(MAX_FPS);

    let mut shape = CustomShape::new(Box::new(TriangleShape));
    shape.set_position((400., 300.));
    shape.set_origin((400., 300.));
    shape.set_outline_thickness(3.);

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

        shape.set_rotation(counter.seconds.sin().abs() * 360.0);
        let scale = counter.seconds.cos().abs();
        shape.set_scale(scale);
        shape.set_fill_color(hue_time(counter.seconds));
        shape.set_outline_color(hue_time(counter.seconds / 2.0));
        window.clear(Color::BLACK);
        window.draw(&shape);
        window.display();
    }
    Ok(())
}
