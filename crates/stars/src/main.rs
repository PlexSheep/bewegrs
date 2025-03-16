use std::path::PathBuf;

use bewegrs::sfml;
use bewegrs::tracing;

use getopts::Options;
use sfml::{
    SfResult,
    cpp::FBox,
    graphics::{
        Color, FloatRect, Font, Image, IntRect, PrimitiveType, RectangleShape, RenderTarget,
        RenderWindow, Texture, Transformable, Vertex, VertexBuffer, VertexBufferUsage,
    },
    system::{Vector2f, Vector2u},
    window::{Event, Key, Style, VideoMode},
};
use tracing::{debug, error, info};

use bewegrs::{
    counters::Counters,
    setup,
    ui::{ComprehensiveElement, ComprehensiveUi, elements::info::Info},
};

const MAX_FPS: usize = 60;
const BG: Color = Color::rgb(30, 20, 20);
const STAR_AMOUNT: usize = 100_000;
const DEFAULT_SPEED: f32 = 0.8;

// Star configuration
const STAR_RADIUS: f32 = 150.0;
const FAR_PLANE: f32 = 2200.0;
const NEAR_PLANE: f32 = 5.5;
const BEHIND_CAMERA: f32 = 60.5;
const SPREAD: f32 = FAR_PLANE * 40.0;

// Performance configuration
const FAR_THRESH: f32 = FAR_PLANE / 1.7;
const LAZY_STAR_UPDATE_INTERVAL: u64 = MAX_FPS as u64 / 15;

fn main() -> SfResult<()> {
    setup();
    let args: Vec<_> = std::env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("s", "stars", "amount of stars", "STARS");
    opts.optopt("i", "sprite", "sprite texture to use for stars", "IMAGE");
    opts.optflag("h", "help", "print help menu");
    opts.optflag("l", "hide-logo", "hide the logo");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };
    if matches.opt_present("help") {
        print_usage(&program, opts);
        return Ok(());
    }
    let sprite_path: Option<PathBuf> = matches.opt_get("sprite").expect("boom");
    if let Some(path) = &sprite_path {
        info!("using sprite: {}", path.to_string_lossy());
    }

    let stars_amount: usize = matches
        .opt_get("stars")
        .expect("could not get stars option")
        .unwrap_or(STAR_AMOUNT);

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

    let mut font = Font::new()?;
    font.load_from_memory_static(include_bytes!("../../../resources/sansation.ttf"))?;

    let profile_image = &*Image::from_memory(include_bytes!("../../../resources/profile.png"))?;
    let mut texture = Texture::from_image(profile_image, IntRect::default())?;
    texture.set_smooth(true);

    let mut gui = ComprehensiveUi::build(&window, &font, &video, &counter)?;
    gui.set_no_cursor(&mut window, true);

    if !matches.opt_present("hide-logo") {
        gui.info
            .set_logo(&texture, "Christoph J. Scherr\nsoftware@cscherr.de")?;
    }

    let stars = Stars::new(video, stars_amount, sprite_path)?;
    gui.info.set_custom_info("stars", stars.stars.len());
    gui.info
        .set_custom_info("lazy_update_interval", LAZY_STAR_UPDATE_INTERVAL);
    gui.add(Box::new(stars));

    let mut logo = RectangleShape::new();

    logo.set_position((400.0, 400.0));
    debug!("{logo:?}");

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
        gui.draw_with(&mut window, &counter);

        window.draw(&logo);

        counter.frame_prepare_display();
        window.display();
    }
    Ok(())
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum StarLodLevel {
    Detail,
    Far,
}

struct StarRenderCtx<'render> {
    vertices: &'render mut [Vertex],
    texture_size: &'render Vector2u,
    color: &'render Color,
    i: usize,
    screen_x: f32,
    screen_y: f32,
    radius: f32,
}

// Simple star data without the SFML object
struct Star {
    position: Vector2f, // World-space position (centered around 0,0)
    distance: f32,      // z-coordinate
    active: bool,       // Whether star is active/visible
    lod_level: StarLodLevel,
}

impl Star {
    fn new(width: u32, height: u32) -> Self {
        let mut star = Star {
            position: Vector2f::new(0.0, 0.0),
            distance: 0.0,
            active: true,
            lod_level: StarLodLevel::Detail,
        };

        star.rand_pos(width, height);
        star.rand_distance();
        star
    }

    // Update the star's LOD level based on distance
    #[inline]
    fn update_lod(&mut self) {
        self.lod_level = if self.distance < FAR_THRESH {
            StarLodLevel::Detail
        } else {
            StarLodLevel::Far
        };
    }

    #[inline]
    fn rand_distance(&mut self) {
        self.distance = rand::random_range(NEAR_PLANE..FAR_PLANE);
    }

    fn rand_pos(&mut self, width: u32, height: u32) {
        // Generate position centered around origin in world space
        // Scale by FAR_PLANE to give stars enough space
        let aspect_ratio = width as f32 / height as f32;
        let star_free = FloatRect::new(
            width as f32 / -2.0,
            height as f32 / -2.0,
            width as f32 * 0.90,
            height as f32 * 0.90,
        );
        loop {
            self.position = Vector2f::new(
                rand::random_range(-SPREAD..SPREAD),
                rand::random_range(-SPREAD..SPREAD),
            ) * aspect_ratio;
            if !star_free.contains(self.position) {
                break;
            }
        }
    }

    fn update(&mut self, speed: f32) {
        // Decrease distance (move closer)
        self.distance -= speed;
    }

    fn update_lazy(&mut self, width: u32, height: u32) {
        self.update_lod();
        // If star gets too close, reset it
        if self.distance < NEAR_PLANE - BEHIND_CAMERA {
            self.rand_pos(width, height);
            self.distance = FAR_PLANE - rand::random_range(0.0..FAR_PLANE / 200.0);
        }
        // If star gets too far, reset it
        if self.distance > FAR_PLANE {
            self.rand_pos(width, height);
            self.distance = -BEHIND_CAMERA + rand::random_range(0.0..FAR_PLANE / 200.0);
        }
        // Check visibility
        self.active = self.is_visible();
    }

    #[inline]
    fn is_visible(&self) -> bool {
        // Check if star is big enough to see
        NEAR_PLANE / self.distance > 0.001
    }

    // Create vertices for this star (a quad made of 4 vertices)
    fn create_vertices(
        &self,
        width: u32,
        height: u32,
        vertices: &mut [Vertex],
        index: usize,
        texture_size: &Vector2u,
        color: &Color,
    ) {
        // Create the 4 vertices of the quad (one star = 4 vertices)
        let i = index * 4;

        // If star is active, create a visible quad
        if self.active {
            // Calculate perspective scale factor
            let scale = NEAR_PLANE / self.distance;

            // Calculate projected screen position
            let screen_x = self.position.x * scale + width as f32 / 2.0;
            let screen_y = self.position.y * scale + height as f32 / 2.0;

            // Depth ratio for color (farther stars are dimmer)
            let depth_ratio = (self.distance - NEAR_PLANE) / (FAR_PLANE - NEAR_PLANE);
            let brightness = ((1.0 - depth_ratio) * 255.0) as u8;

            // Calculate radius based on distance
            let radius = STAR_RADIUS * scale;

            let darkness = 255 - brightness;
            let adjusted_color = Color::rgb(
                color.r.saturating_sub(darkness),
                color.g.saturating_sub(darkness),
                color.b.saturating_sub(darkness),
            );

            let mut ctx = StarRenderCtx {
                vertices,
                texture_size,
                color: &adjusted_color,
                i,
                screen_x,
                screen_y,
                radius,
            };

            // PERF: interestingly, the only difference in these functions is how the texture
            // coords are set. In detailed, they are set to the dimensions of the texture_size. In
            // far, all are set to the center of the texture_size.
            // This makes a difference of a few percent points at profiling
            match self.lod_level {
                StarLodLevel::Detail => Self::create_vertecies_detailed(&mut ctx),
                StarLodLevel::Far => Self::create_vertecies_far(&mut ctx),
            }
        }
        // If star is not active, create an invisible quad
        else {
            let transparent = Color::rgba(0, 0, 0, 0); // Fully transparent

            // Set all 4 vertices to be invisible (zero size, transparent)
            for j in 0..4 {
                vertices[i + j].color = transparent;

                // PERF: we can just leave the position and tex_coords as before, since the thing
                // is transparent anyway.
            }
        }
    }

    fn create_vertecies_detailed(ctx: &mut StarRenderCtx<'_>) {
        let tex_x: f32 = ctx.texture_size.x as f32;
        let tex_y: f32 = ctx.texture_size.y as f32;

        for j in 0..4 {
            ctx.vertices[ctx.i + j].color = *ctx.color;
        }

        ctx.vertices[ctx.i].position =
            Vector2f::new(ctx.screen_x - ctx.radius, ctx.screen_y - ctx.radius);
        ctx.vertices[ctx.i + 1].position =
            Vector2f::new(ctx.screen_x + ctx.radius, ctx.screen_y - ctx.radius);
        ctx.vertices[ctx.i + 2].position =
            Vector2f::new(ctx.screen_x + ctx.radius, ctx.screen_y + ctx.radius);
        ctx.vertices[ctx.i + 3].position =
            Vector2f::new(ctx.screen_x - ctx.radius, ctx.screen_y + ctx.radius);

        ctx.vertices[ctx.i].tex_coords = Vector2f::new(0.0, 0.0);
        ctx.vertices[ctx.i + 1].tex_coords = Vector2f::new(tex_x, 0.0);
        ctx.vertices[ctx.i + 2].tex_coords = Vector2f::new(tex_x, tex_y);
        ctx.vertices[ctx.i + 3].tex_coords = Vector2f::new(0.0, tex_y);
    }

    fn create_vertecies_far(ctx: &mut StarRenderCtx<'_>) {
        let tex_x: f32 = ctx.texture_size.x as f32 / 2.0;
        let tex_y: f32 = ctx.texture_size.y as f32 / 2.0;
        let tex_center = Vector2f::new(tex_x, tex_y);

        for j in 0..4 {
            ctx.vertices[ctx.i + j].color = *ctx.color;
        }

        ctx.vertices[ctx.i].position =
            Vector2f::new(ctx.screen_x - ctx.radius, ctx.screen_y - ctx.radius);
        ctx.vertices[ctx.i + 1].position =
            Vector2f::new(ctx.screen_x + ctx.radius, ctx.screen_y - ctx.radius);
        ctx.vertices[ctx.i + 2].position =
            Vector2f::new(ctx.screen_x + ctx.radius, ctx.screen_y + ctx.radius);
        ctx.vertices[ctx.i + 3].position =
            Vector2f::new(ctx.screen_x - ctx.radius, ctx.screen_y + ctx.radius);

        // Use a fixed texture coordinate that's known to be non-transparent
        ctx.vertices[ctx.i].tex_coords = tex_center;
        ctx.vertices[ctx.i + 1].tex_coords = tex_center;
        ctx.vertices[ctx.i + 2].tex_coords = tex_center;
        ctx.vertices[ctx.i + 3].tex_coords = tex_center;
    }
}

struct Stars {
    stars: Vec<Star>,
    vertex_buffer: FBox<VertexBuffer>,
    vertices: Vec<Vertex>,
    video: VideoMode,
    speed: f32,
    texture: FBox<Texture>,
    last_sorted_frame: u64,
    texture_size: Vector2u,
    texture_color: Color,
}

impl Stars {
    pub fn new(video: VideoMode, amount: usize, sprite_path: Option<PathBuf>) -> SfResult<Self> {
        let (texture, texture_color) = Self::create_star_texture(sprite_path)?;

        debug!(
            "Star texture dimensions: {}x{}",
            texture.size().x,
            texture.size().y
        );

        // Create stars
        let mut stars: Vec<Star> = Vec::with_capacity(amount);
        for _ in 0..amount {
            stars.push(Star::new(video.width, video.height));
        }

        // Create a vertex array to store our quad data (4 vertices per star)
        let mut vertices = vec![Vertex::default(); amount * 4];

        // Initialize all vertices as transparent (this is crucial)
        for vertex in &mut vertices {
            vertex.color = Color::TRANSPARENT; // Fully transparent
        }

        // Create the vertex buffer
        let mut vertex_buffer =
            VertexBuffer::new(PrimitiveType::QUADS, amount * 4, VertexBufferUsage::DYNAMIC)?;

        // Initialize vertex data
        for (i, star) in stars.iter().enumerate() {
            star.create_vertices(
                video.width,
                video.height,
                &mut vertices,
                i,
                &texture.size(),
                &texture_color,
            );
        }

        // Update the vertex buffer with initial data
        // PERF: this takes a lot of time, but since vertex buffers are stored in the gpu memory,
        // it saves us time later when drawing.
        // I have tried the performance with just vertex arrays (Vec<Vertex>) and it is worse.
        vertex_buffer.update(&vertices, 0)?;

        let mut stars = Stars {
            stars,
            vertex_buffer,
            vertices,
            video,
            speed: DEFAULT_SPEED,
            last_sorted_frame: 0,
            texture_size: texture.size(),
            texture,
            texture_color,
        };

        stars.sort();

        Ok(stars)
    }

    // Creates a procedural star texture
    fn create_star_texture(sprite_path: Option<PathBuf>) -> SfResult<(FBox<Texture>, Color)> {
        // Load star texture
        let star_image = match sprite_path {
            None => Image::from_memory(include_bytes!("../../../resources/star.png"))?,
            Some(p) => Image::from_file(p.to_str().expect("could not convert path to str"))?,
        };

        // Debug: Check the center pixel
        let center_x = star_image.size().x / 2;
        let center_y = star_image.size().y / 2;
        let center_color = star_image
            .pixel_at(center_x, center_y)
            .expect("could not get center color of star sprite");
        info!(
            "Center pixel of star texture: R:{}, G:{}, B:{}, A:{}",
            center_color.r, center_color.g, center_color.b, center_color.a
        );

        let mut texture = Texture::from_image(&star_image, IntRect::default())?;
        texture.set_smooth(true); // Enable smoothing for better scaling

        Ok((texture, center_color))
    }

    fn update_vertices(&mut self) -> SfResult<()> {
        // Update all vertices in the vertices array
        for (i, star) in self.stars.iter().enumerate() {
            star.create_vertices(
                self.video.width,
                self.video.height,
                &mut self.vertices,
                i,
                &self.texture_size,
                &self.texture_color,
            );
        }

        // Update the vertex buffer with the new vertex data
        // This updates all vertices, including the "invisible" ones
        self.vertex_buffer.update(&self.vertices, 0)?;

        Ok(())
    }

    fn sort(&mut self) {
        self.stars
            .sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());
    }
}

impl<'s, const N: usize> ComprehensiveElement<'s, N> for Stars {
    fn update(&mut self, counters: &Counters<N>, info: &mut Info<'s>) {
        if self.speed == 0.0 {
            return;
        }

        // Update star positions
        for star in self.stars.iter_mut() {
            star.update(self.speed);
        }

        // Sort stars by distance - only when needed
        if counters.frames % LAZY_STAR_UPDATE_INTERVAL == 0 {
            for star in self.stars.iter_mut() {
                star.update_lazy(self.video.width, self.video.height);
            }

            self.sort();
            self.last_sorted_frame = counters.frames;
            info.set_custom_info("last_sort", self.last_sorted_frame);
        }

        // Update vertex buffer
        if let Err(e) = self.update_vertices() {
            error!("bad stars update: {e}");
        }
    }

    #[allow(clippy::field_reassign_with_default)] // wtf? I'm not doing that
    fn draw_with(
        &mut self,
        sfml_w: &mut FBox<RenderWindow>,
        _egui_w: &mut bewegrs::egui_sfml::SfEgui,
        _counters: &Counters<N>,
        _info: &mut Info<'s>,
    ) {
        // Create render states with our texture
        let mut states = sfml::graphics::RenderStates::default();
        states.texture = Some(&*self.texture);
        // Draw all stars with a single draw call
        sfml_w.draw_with_renderstates(&*self.vertex_buffer, &states);
    }

    fn z_level(&self) -> u16 {
        0
    }

    fn update_slow(&mut self, _counters: &Counters<N>, info: &mut Info<'s>) {
        info.set_custom_info(
            "LOD_Detailed",
            self.stars
                .iter()
                .filter(|s| s.lod_level == StarLodLevel::Detail)
                .count(),
        );
        info.set_custom_info(
            "LOD_Far",
            self.stars
                .iter()
                .filter(|s| s.lod_level == StarLodLevel::Far)
                .count(),
        );
    }

    fn process_event(&mut self, event: &Event, info: &mut Info<'s>) {
        match event {
            Event::KeyPressed {
                code: Key::W,
                shift,
                ..
            } => {
                self.speed += 0.1 * if *shift { 10.0 } else { 1.0 };
                info.set_custom_info("speed", format_args!("{:.03}", self.speed));
            }
            Event::KeyPressed {
                code: Key::S,
                shift,
                ..
            } => {
                self.speed -= 0.1 * if *shift { 10.0 } else { 1.0 };
                info.set_custom_info("speed", format_args!("{:.03}", self.speed));
            }
            Event::KeyPressed {
                code: Key::Space,
                shift: true,
                ..
            } => {
                self.speed = 0.0;
                info.set_custom_info("speed", format_args!("{:.03}", self.speed));
            }
            _ => (),
        }
    }
}
