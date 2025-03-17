use std::path::PathBuf;

use rayon::prelude::*;

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

const DEFAULT_MAX_FPS: u64 = 60;
const DEFAULT_STAR_AMOUNT: usize = 500_000;
const BG: Color = Color::rgb(30, 20, 20);
const DEFAULT_SPEED: f32 = 0.8;

// Star configuration
const STAR_RADIUS: f32 = 150.0;
const FAR_PLANE: f32 = 2200.0;
const NEAR_PLANE: f32 = 5.5;
const BEHIND_CAMERA: f32 = 60.5;
const SPREAD: f32 = FAR_PLANE * 40.0;

// Performance configuration
const FAR_THRESH: f32 = FAR_PLANE / 3.5;
const POINT_THRESH: f32 = FAR_PLANE / 1.5;

// export this so that we can use benchmarks
pub fn stars(args: Vec<String>) -> SfResult<()> {
    setup();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("s", "stars", "amount of stars", "STARS");
    opts.optopt("i", "sprite", "sprite texture to use for stars", "IMAGE");
    opts.optflag("h", "help", "print help menu");
    opts.optflag("l", "hide-logo", "hide the logo");
    opts.optopt("f", "fps", "set the fps limit", "FPS");
    opts.optopt("e", "exit-after", "exit after SECS seconds", "SECS");
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
        .unwrap_or(DEFAULT_STAR_AMOUNT);

    let fps_limit: u64 = matches
        .opt_get("fps")
        .expect("could not get fps option")
        .unwrap_or(DEFAULT_MAX_FPS);
    info!("fps limit: {fps_limit}");

    let exit_after: Option<u64> = matches
        .opt_get("exit-after")
        .expect("could not get fps option");
    info!("exit_after: {exit_after:?}");

    let video = VideoMode::fullscreen_modes()[0];
    info!("video mode: {video:?}");
    let mut window = RenderWindow::new(
        video,
        "Starfield",
        Style::DEFAULT | Style::FULLSCREEN,
        &Default::default(),
    )?;
    let mut counter = Counters::start(fps_limit)?;
    window.set_framerate_limit(fps_limit as u32);

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
    gui.info.set_custom_info("star_r", STAR_RADIUS);
    gui.info.set_custom_info("far", FAR_PLANE);
    gui.info.set_custom_info("far_thresh", FAR_THRESH);
    gui.info.set_custom_info("point_thresh", POINT_THRESH);
    gui.info.set_custom_info("near", NEAR_PLANE);
    gui.info.set_custom_info("spread", SPREAD);
    gui.info.set_custom_info("behind_cam", BEHIND_CAMERA);
    gui.info.set_custom_info(
        "resolution",
        format_args!(
            "{}x{} {}bpp",
            video.width, video.height, video.bits_per_pixel
        ),
    );
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
        if counter.frames % counter.fps_limit == 1 {
            gui.update_slow(&counter)
        }

        window.clear(BG);
        gui.draw_with(&mut window, &counter);

        window.draw(&logo);

        counter.frame_prepare_display();
        window.display();

        if let Some(secs) = exit_after {
            if counter.seconds >= secs as f32 {
                break 'mainloop;
            }
        }
    }
    Ok(())
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum StarLodLevel {
    Detail,
    Point,
}

pub struct StarRenderCtx<'render> {
    pub vertices: &'render mut [Vertex],
    pub texture_size: &'render Vector2u,
    pub color: &'render Color,
    pub i: usize,
    pub screen_x: f32,
    pub screen_y: f32,
    pub radius: f32,
}

// Simple star data without the SFML object
pub struct Star {
    pub position: Vector2f, // World-space position (centered around 0,0)
    pub distance: f32,      // z-coordinate
    pub active: bool,       // Whether star is active/visible
    pub lod_level: StarLodLevel,
}

impl Star {
    pub fn new(width: u32, height: u32) -> Self {
        let mut star = Star {
            position: Vector2f::new(0.0, 0.0),
            distance: 0.0,
            active: true,
            lod_level: StarLodLevel::Detail,
        };

        star.rand_pos(width, height);
        star.distance = Star::rand_distance();
        star
    }

    // Update the star's LOD level based on distance
    #[inline]
    pub fn update_lod(&mut self) {
        self.lod_level = if self.distance < POINT_THRESH {
            StarLodLevel::Detail
        } else {
            StarLodLevel::Point
        };
    }

    #[inline]
    fn rand_distance() -> f32 {
        rand::random_range(NEAR_PLANE..FAR_PLANE)
    }

    pub fn rand_pos(&mut self, width: u32, height: u32) {
        // Generate position centered around origin in world space
        // Scale by FAR_PLANE to give stars enough space
        let aspect_ratio = width as f32 / height as f32;
        let star_free = FloatRect::new(
            width as f32 / -2.0,
            height as f32 / -2.0,
            width as f32 * 0.7,
            height as f32 * 0.7,
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

    pub fn update(&mut self, speed: f32, width: u32, height: u32) {
        // Decrease distance (move closer)
        self.distance -= speed;

        // If star gets too close, reset it
        if self.distance <= -BEHIND_CAMERA {
            self.rand_pos(width, height);
            self.distance = FAR_PLANE;
            self.update_lazy(width, height);
        }
        // If star gets too far, reset it
        else if self.distance >= FAR_PLANE {
            self.rand_pos(width, height);
            self.distance = rand::random_range(-BEHIND_CAMERA..0.0);
            self.update_lazy(width, height);
        }
    }

    pub fn update_lazy(&mut self, _width: u32, _height: u32) {
        self.update_lod();
        // Check visibility
        self.active = self.is_visible();
    }

    #[inline]
    pub fn is_visible(&self) -> bool {
        // Check if star is big enough to see
        NEAR_PLANE / self.distance > 0.001
    }

    // Create vertices for this star (a quad made of 4 vertices)
    pub fn create_vertices(
        &self,
        width: u32,
        height: u32,
        vertices: &mut [Vertex],
        index: usize,
        texture_size: &Vector2u,
        color: &Color,
        aspect_ratio: f32,
    ) {
        // Skip point stars - they'll be handled separately
        if self.lod_level == StarLodLevel::Point || !self.active {
            // Make vertices transparent for skipped stars
            let i = index * 4;
            for j in 0..4 {
                vertices[i + j].color = Color::TRANSPARENT;
            }
            return;
        }

        // Create the 4 vertices of the quad (one star = 4 vertices)
        let i = index * 4;

        // If star is active, create a visible quad
        // Calculate perspective scale factor
        let scale = NEAR_PLANE / self.distance;

        // Calculate projected screen position
        let screen_x = self.position.x * scale * aspect_ratio + width as f32 / 2.0;
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
            StarLodLevel::Point => unreachable!(),
        }
    }

    pub fn create_vertecies_detailed(ctx: &mut StarRenderCtx<'_>) {
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

        if true {
            ctx.vertices[ctx.i].tex_coords = Vector2f::new(0.0, 0.0);
            ctx.vertices[ctx.i + 1].tex_coords = Vector2f::new(tex_x, 0.0);
            ctx.vertices[ctx.i + 2].tex_coords = Vector2f::new(tex_x, tex_y);
            ctx.vertices[ctx.i + 3].tex_coords = Vector2f::new(0.0, tex_y);
        } else {
            let tex_center = Vector2f::new(tex_x, tex_y);
            ctx.vertices[ctx.i].tex_coords = tex_center;
            ctx.vertices[ctx.i + 1].tex_coords = tex_center;
            ctx.vertices[ctx.i + 2].tex_coords = tex_center;
            ctx.vertices[ctx.i + 3].tex_coords = tex_center;
        }
    }
}

pub struct Stars {
    stars: Vec<Star>,
    star_vertices_buf: FBox<VertexBuffer>,
    point_vertices_buf: FBox<VertexBuffer>,
    star_vertices: Vec<Vertex>,
    point_vertices: Vec<Vertex>,
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

        let mut star_vertices = vec![Vertex::default(); amount * 4];
        let mut point_vertices = vec![Vertex::default(); amount];

        star_vertices.par_iter_mut().for_each(|vertex| {
            vertex.color = Color::TRANSPARENT;
        });
        point_vertices.par_iter_mut().for_each(|vertex| {
            vertex.color = Color::TRANSPARENT;
        });

        let mut star_vertices_buf =
            VertexBuffer::new(PrimitiveType::QUADS, amount * 4, VertexBufferUsage::STREAM)?;
        let mut point_vertices_buf =
            VertexBuffer::new(PrimitiveType::POINTS, amount, VertexBufferUsage::STREAM)?;

        star_vertices_buf.update(&star_vertices, 0)?;
        point_vertices_buf.update(&point_vertices, 0)?;

        let mut stars = Stars {
            stars,
            star_vertices_buf,
            star_vertices,
            point_vertices,
            video,
            speed: DEFAULT_SPEED,
            last_sorted_frame: 0,
            texture_size: texture.size(),
            texture,
            texture_color,
            point_vertices_buf,
        };

        stars.sort(0);
        stars.update_vertices()?;

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

    pub fn update_vertices(&mut self) -> SfResult<()> {
        self.update_point_vertices()?;
        let aspect_ratio = self.video.width as f32 / self.video.height as f32;
        self.stars.par_iter().enumerate().for_each(|(i, star)| {
            star.create_vertices(
                self.video.width,
                self.video.height,
                #[allow(clippy::needless_borrow)] // not needless
                &mut unsafe {
                    please_give_me_a_mutable_reference_because_i_want_speed(&self.star_vertices)
                },
                i,
                &self.texture_size,
                &self.texture_color,
                aspect_ratio,
            );
        });

        // Update the vertex buffer with the new vertex data
        // This updates all vertices, including the "invisible" ones
        // PERF: this takes a lot of time, but since vertex buffers are stored in the gpu memory,
        // it saves us time later when drawing.
        // I have tried the performance with just vertex arrays (Vec<Vertex>) and it is worse.
        self.star_vertices_buf.update(&self.star_vertices, 0)?;
        self.point_vertices_buf.update(&self.point_vertices, 0)?;

        Ok(())
    }

    pub fn update_point_vertices(&mut self) -> SfResult<()> {
        let aspect_ratio = self.video.width as f32 / self.video.height as f32;
        self.stars.par_iter().enumerate().for_each(|(i, star)| {
            // Only process active point stars
            if star.lod_level == StarLodLevel::Point && star.active {
                // Calculate perspective scale factor
                let scale = NEAR_PLANE / star.distance;

                // Calculate projected screen position
                let screen_x =
                    star.position.x * scale * aspect_ratio + self.video.width as f32 / 2.0;
                let screen_y = star.position.y * scale + self.video.height as f32 / 2.0;

                // Depth ratio for color (farther stars are dimmer)
                let depth_ratio = (star.distance - NEAR_PLANE) / (FAR_PLANE - NEAR_PLANE);
                let brightness = ((1.0 - depth_ratio) * 255.0) as u8;

                let darkness = 255 - brightness;
                let adjusted_color = Color::rgb(
                    self.texture_color.r.saturating_sub(darkness),
                    self.texture_color.g.saturating_sub(darkness),
                    self.texture_color.b.saturating_sub(darkness),
                );

                // Create a point vertex
                let vertex = Vertex::new(
                    Vector2f::new(screen_x, screen_y),
                    adjusted_color,
                    Vector2f::new(
                        self.texture_size.x as f32 / 2.0,
                        self.texture_size.y as f32 / 2.0,
                    ),
                );

                let thing;
                unsafe {
                    thing = please_give_me_a_mutable_reference_because_i_want_speed(
                        self.point_vertices.get(i).expect("it promise this exists"),
                    );
                }
                *thing = vertex;
            }
        });
        Ok(())
    }

    pub fn sort(&mut self, frame: u64) {
        self.stars
            .sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());
        self.last_sorted_frame = frame;
    }
}

impl<'s> ComprehensiveElement<'s> for Stars {
    fn update(&mut self, counters: &Counters, info: &mut Info<'s>) {
        if self.speed == 0.0 {
            return;
        }

        // Update star positions
        let chunk_size = self.stars.len() / rayon::max_num_threads();
        self.stars.par_chunks_mut(chunk_size).for_each(|chunk| {
            for star in chunk {
                star.update(self.speed, self.video.width, self.video.height);
            }
        });

        // Sort stars by distance - only when needed
        if counters.frames % lazy_interval(counters.fps_limit) == 0 {
            for star in self.stars.iter_mut() {
                star.update_lazy(self.video.width, self.video.height);
            }

            self.sort(counters.frames);
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
        _counters: &Counters,
        _info: &mut Info<'s>,
    ) {
        // Create render states with our texture
        let mut states = sfml::graphics::RenderStates::default();
        states.texture = Some(&*self.texture);
        // Draw all stars with a single draw call
        sfml_w.draw(&*self.point_vertices_buf);
        sfml_w.draw_with_renderstates(&*self.star_vertices_buf, &states);
    }

    fn z_level(&self) -> u16 {
        0
    }

    fn update_slow(&mut self, counters: &Counters, info: &mut Info<'s>) {
        info.set_custom_info(
            "LOD_Detailed",
            self.stars
                .iter()
                .filter(|s| s.lod_level == StarLodLevel::Detail)
                .count(),
        );
        info.set_custom_info(
            "LOD_Point",
            self.stars
                .iter()
                .filter(|s| s.lod_level == StarLodLevel::Point)
                .count(),
        );
        info.set_custom_info("lazy_interval", lazy_interval(counters.fps_limit));
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

#[inline]
fn lazy_interval(fps_limit: u64) -> u64 {
    2
}

#[allow(invalid_reference_casting)] // just fucking do what I say
#[allow(clippy::mut_from_ref)]
#[inline]
unsafe fn please_give_me_a_mutable_reference_because_i_want_speed<T>(thing: &T) -> &mut T {
    unsafe {
        let thing_pointer = thing as *const T;
        let thing_mut = thing_pointer as *mut T;
        &mut *thing_mut
    }
}
