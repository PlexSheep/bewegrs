use sfml::{
    SfResult,
    cpp::FBox,
    graphics::{
        CircleShape, Color, Drawable, Font, RectangleShape, RenderTarget, RenderWindow, Shape,
        Transformable, glsl::Vec2,
    },
    system::Vector2f,
    window::{Event, Key, Style, VideoMode},
};
use tracing::info;

use bewegrs::{
    counters::Counters,
    setup,
    ui::{ComprehensiveElement, ComprehensiveUi},
};

const MAX_FPS: usize = 60;
const BG: Color = Color::rgb(30, 20, 20);
const STAR_AMOUNT: usize = 1000;

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
    let mut counter = Counters::<MAX_FPS>::start()?;
    window.set_framerate_limit(MAX_FPS as u32);

    let mut font = Font::new()?;
    font.load_from_memory_static(include_bytes!("../resources/sansation.ttf"))?;

    let mut gui = ComprehensiveUi::build(&window, &font, &video, &counter)?;

    let stars = Stars::new(&font, &video, &counter, STAR_AMOUNT);
    gui.info.set_custom_info("stars", STAR_AMOUNT);
    gui.add(Box::new(stars));

    let mut backdrop = RectangleShape::new();
    backdrop.set_size(Vec2::new(video.width as f32, video.height as f32));
    backdrop.set_fill_color(BG);

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

        window.clear(BG);

        window.draw(&backdrop);
        gui.draw_with(&mut window, &counter);

        counter.frame_prepare_display();
        window.display();
    }
    Ok(())
}

struct Star<'s> {
    distance: f32,
    object: CircleShape<'s>,
}

impl Star<'_> {
    const DIST_FAR: f32 = 10.0;
    const DIST_NEAR: f32 = 1.0;
    const R_MAX: f32 = 17.3;
    const R_MIN: f32 = 16.8;
    const SPEED: f32 = 0.01;

    fn new(width: u32, height: u32) -> Self {
        let r = rand::random_range(Self::R_MIN..Self::R_MAX);
        let object = CircleShape::new(r, 16);
        let mut star = Self {
            object,
            distance: 0.0,
        };
        star.rand_pos(width, height);
        star.rand_distance();
        star
    }

    fn rand_distance(&mut self) {
        self.distance = rand::random_range(Self::DIST_NEAR..Self::DIST_FAR);
    }

    fn rand_pos(&mut self, width: u32, height: u32) {
        self.object.set_position(rand_point(width, height));
        // self.object.set_position((0.0, 0.0));
    }

    fn update(&mut self, width: u32, height: u32) {
        if self.distance <= Self::DIST_NEAR {
            self.rand_pos(width, height);
            self.rand_distance();
            return;
        }
        self.distance -= Self::SPEED;
        let scale = 1.0 / self.distance;
        self.object.set_position({
            let p = self.object.position();

            (
                p.x * scale + width as f32 / 2.0,
                p.y * scale + height as f32 / 2.0,
            )
        });
        self.object.set_scale(scale);
    }
}

impl Drawable for Star<'_> {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn RenderTarget,
        states: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        self.object.draw(target, states);
    }
}

struct Stars<'s> {
    stars: Vec<Star<'s>>,
    video: &'s VideoMode,
}

impl<'s> Stars<'s> {
    pub fn new<const N: usize>(
        font: &'s FBox<Font>,
        video: &'s VideoMode,
        counters: &Counters<N>,
        amount: usize,
    ) -> Self {
        let mut stars: Vec<Star> = Vec::with_capacity(amount);
        for _i in 0..amount {
            stars.push(Star::new(video.width, video.height));
        }

        Stars { stars, video }
    }
}

impl<'s, const N: usize> ComprehensiveElement<'s, N> for Stars<'s> {
    fn update(&mut self, counters: &Counters<N>) {
        for star in self.stars.iter_mut() {
            star.update(self.video.width, self.video.height);
        }
    }
    fn draw_with(
        &mut self,
        sfml_w: &mut FBox<RenderWindow>,
        egui_w: &mut egui_sfml::SfEgui,
        counters: &Counters<N>,
    ) {
        for star in self.stars.iter() {
            sfml_w.draw(star);
        }
    }
    fn z_level(&self) -> u16 {
        0
    }
}

fn rand_point(width: u32, height: u32) -> Vector2f {
    (
        rand::random_range(0f32..width as f32),
        rand::random_range(0f32..height as f32),
    )
        .into()
}
