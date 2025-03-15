use sfml::{
    cpp::FBox,
    graphics::{
        CircleShape, Color, Drawable, FloatRect, Font, RectangleShape, RenderTarget, RenderWindow,
        Shape, Transformable,
    },
    system::Vector2f,
    window::{Event, Key, Style, VideoMode},
    SfResult,
};
use tracing::info;

use bewegrs::{
    counters::Counters,
    setup,
    ui::{ComprehensiveElement, ComprehensiveUi},
};

const MAX_FPS: usize = 30;
const BG: Color = Color::rgb(30, 20, 20);
const STAR_AMOUNT: usize = 60000;

// Star configuration
const STAR_RADIUS: f32 = 50.0;
const FAR_PLANE: f32 = 800.0;
const NEAR_PLANE: f32 = 5.5;
const SPEED: f32 = 1.0;

fn main() -> SfResult<()> {
    setup();

    let video = VideoMode::fullscreen_modes()[0];
    info!("video mode: {video:?}");
    let mut window = RenderWindow::new(
        video,
        "Starfield",
        Style::DEFAULT | Style::FULLSCREEN,
        &Default::default(),
    )?;
    let mut counter = Counters::<MAX_FPS>::start()?;
    window.set_framerate_limit(MAX_FPS as u32);
    window.set_mouse_cursor_visible(false);

    let mut font = Font::new()?;
    font.load_from_memory_static(include_bytes!("../resources/sansation.ttf"))?;

    let mut gui = ComprehensiveUi::build(&window, &font, &video, &counter)?;

    let stars = Stars::new(&font, &video, &counter, STAR_AMOUNT);
    gui.info.set_custom_info("stars", STAR_AMOUNT);
    gui.add(Box::new(stars));

    let mut backdrop = RectangleShape::new();
    backdrop.set_size((video.width as f32, video.height as f32));
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
    distance: f32, // z-coordinate
    // World-space position (centered around 0,0), this is not for the display
    // but for calculations
    position: Vector2f,
    object: CircleShape<'s>,
}

impl Star<'_> {
    const DIST_FAR: f32 = FAR_PLANE;
    const DIST_NEAR: f32 = NEAR_PLANE;
    const R_MAX: f32 = STAR_RADIUS;
    const R_MIN: f32 = STAR_RADIUS * 0.8;
    const SPEED: f32 = SPEED;
    const SPREAD: f32 = Self::DIST_FAR * 40.0;

    fn new(width: u32, height: u32) -> Self {
        // Create circle shape with random radius
        let r = rand::random_range(Self::R_MIN..Self::R_MAX);
        let mut object = CircleShape::new(r, 8);
        object.set_origin((r, r)); // Set origin to center of circle

        // Create star with random world-space position and distance
        let mut star = Self {
            object,
            position: Vector2f::new(0.0, 0.0),
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
        // Generate position centered around origin in world space
        // Scale by FAR_PLANE to give stars enough space
        let aspect_ratio = width as f32 / height as f32;
        let star_free = FloatRect::new(
            width as f32 / -2.0,
            height as f32 / -2.0,
            width as f32 * 0.95,
            height as f32 * 0.95,
        );
        loop {
            self.position = Vector2f::new(
                rand::random_range(-Self::SPREAD..Self::SPREAD),
                rand::random_range(-Self::SPREAD..Self::SPREAD),
            ) * aspect_ratio;
            if !star_free.contains(self.position) {
                break;
            }
        }
    }

    fn update(&mut self, width: u32, height: u32) {
        // Decrease distance (move closer)
        self.distance -= Self::SPEED;

        // If star gets too close, reset it
        if self.distance <= Self::DIST_NEAR {
            self.rand_pos(width, height);
            self.rand_distance();
            return;
        }

        // Calculate perspective scale factor
        let scale = Self::DIST_NEAR / self.distance;

        // Project position to screen space with perspective
        let screen_x = self.position.x * scale + width as f32 / 2.0;
        let screen_y = self.position.y * scale + height as f32 / 2.0;
        self.object.set_position((screen_x, screen_y));

        // Scale size based on distance
        self.object.set_scale((scale, scale));

        // Adjust brightness based on distance
        let depth_ratio = (self.distance - Self::DIST_NEAR) / (Self::DIST_FAR - Self::DIST_NEAR);
        let brightness = ((1.0 - depth_ratio) * 255.0) as u8;
        self.object
            .set_fill_color(Color::rgb(brightness, brightness, brightness));
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
        stars.sort_unstable_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .expect("could not compare distances")
                .reverse()
        });

        Stars { stars, video }
    }
}

impl<'s, const N: usize> ComprehensiveElement<'s, N> for Stars<'s> {
    fn update(&mut self, counters: &Counters<N>) {
        for star in self.stars.iter_mut() {
            star.update(self.video.width, self.video.height);
        }
        self.stars.sort_unstable_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .expect("could not compare distances")
                .reverse()
        });
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
