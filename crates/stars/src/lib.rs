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
    counter::Counter,
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

const FULL_FRAME_INTERVAL: u64 = 5;

const UPDATE_TIERS: &[(std::ops::Range<u8>, u64)] = &[
    (00..10, 1),  // From nearest star to nearest+10% - every frame
    (10..30, 2),  // From nearest+10% to nearest+30% - every 2 frames
    (30..60, 4),  // From nearest+30% to nearest+60% - every 4 frames
    (60..100, 8), // From nearest+60% to end - every 8 frames
];

// export this so that we can use benchmarks
pub fn stars(args: Vec<String>) -> SfResult<()> {
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("s", "stars", "amount of stars", "STARS");
    opts.optopt("i", "sprite", "sprite texture to use for stars", "IMAGE");
    opts.optflag("h", "help", "print help menu");
    opts.optflag("l", "hide-logo", "hide the logo");
    opts.optflag("v", "verbose", "log more");
    opts.optflag("q", "quiet", "disable logging");
    opts.optopt("f", "fps", "set the fps limit", "FPS");
    opts.optopt("e", "exit-after", "exit after SECS seconds", "SECS");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };
    if !matches.opt_present("quiet") {
        setup(matches.opt_present("verbose"));
    }
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

    let mut font = Font::new()?;
    font.load_from_memory_static(include_bytes!("../../../resources/sansation.ttf"))?;

    let profile_image = &*Image::from_memory(include_bytes!("../../../resources/profile.png"))?;
    let mut texture = Texture::from_image(profile_image, IntRect::default())?;
    texture.set_smooth(true);

    let mut gui = ComprehensiveUi::build(&mut window, &font, &video, fps_limit)?;
    gui.set_no_cursor(&mut window, true);

    if !matches.opt_present("hide-logo") {
        gui.info
            .set_logo(&texture, "Christoph J. Scherr\nsoftware@cscherr.de")?;
    }

    let stars = Stars::new(video, stars_amount, sprite_path)?;
    gui.info.set_custom_info("stars", stars.stars.len());
    gui.info.set_custom_info("star_r", STAR_RADIUS);
    gui.info.set_custom_info("far", FAR_PLANE);
    gui.info.set_custom_info("near", NEAR_PLANE);
    gui.info.set_custom_info("spread", SPREAD);
    gui.info.set_custom_info("behind_cam", BEHIND_CAMERA);
    gui.info
        .set_custom_info("threadool_threads", rayon::current_num_threads());
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

        gui.frame_start();

        gui.update();
        if gui.counter.frames % gui.counter.fps_limit == 1 {
            gui.update_slow();
        }

        window.clear(BG);
        gui.draw_with(&mut window);

        window.draw(&logo);

        gui.display(&mut window);

        if let Some(secs) = exit_after {
            if gui.counter.seconds >= secs as f32 {
                break 'mainloop;
            }
        }
    }

    let frames = gui.counter.frames;
    let secs = gui.counter.seconds;
    info!(
        "{} frames in {} seconds ({:02.04} fps)",
        frames,
        secs,
        frames as f32 / secs as f32
    );

    Ok(())
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program,);
    print!(
        "{}\n{} v{}",
        opts.usage(&brief),
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
}

#[derive(Default, Clone, Copy)]
pub struct Star {
    /// World-space position (centered around 0,0)
    position: Vector2f,
    distance: f32,
    active: bool,
}

pub struct Stars {
    stars: Vec<Star>,
    star_vertices_buf: FBox<VertexBuffer>,
    star_vertices: Vec<Vertex>,
    video: VideoMode,
    speed: f32,
    texture: FBox<Texture>,
    last_sorted_frame: u64,
    texture_size: Vector2u,
    texture_color: Color,
}

struct StarRenderCtx<'render> {
    width: u32,
    height: u32,
    vertices: &'render mut [Vertex],
    index: usize,
    texture_size: &'render Vector2u,
    color: &'render Color,
    aspect_ratio: f32,
}

impl Star {
    fn new() -> Self {
        Star {
            position: Vector2f::new(0.0, 0.0),
            distance: 0.0,
            active: true,
        }
    }

    fn randomize(&mut self, width: u32, height: u32) {
        self.rand_pos(width, height);
        self.distance = Star::rand_distance();
    }

    #[inline]
    fn rand_distance() -> f32 {
        rand::random_range(NEAR_PLANE..FAR_PLANE)
    }

    fn rand_pos(&mut self, width: u32, height: u32) {
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

    fn update(&mut self, speed: f32, width: u32, height: u32) {
        self.distance -= speed;

        // If star gets too close, reset it
        if self.distance <= -BEHIND_CAMERA {
            self.rand_pos(width, height);
            self.distance = FAR_PLANE;
        }
        // If star gets too far, reset it
        else if self.distance >= FAR_PLANE {
            self.rand_pos(width, height);
            self.distance = -BEHIND_CAMERA;
        }

        // NOTE: setting these to constant values is important, because otherwise, we need to sort
        // the star array again. Otherwise, far stars would get rendered over near stars

        self.active = self.is_visible();
    }

    #[inline]
    fn is_visible(&self) -> bool {
        // Check if star is big enough to see
        NEAR_PLANE / self.distance > 0.001
    }

    // Create vertices for this star (a quad made of 4 vertices)
    fn update_vertices(&self, ctx: &mut StarRenderCtx) {
        if !self.active {
            // Make vertices transparent for skipped stars
            let i = ctx.index * 4;
            for j in 0..4 {
                ctx.vertices[i + j].color = Color::TRANSPARENT;
            }
            return;
        }

        // Create the 4 vertices of the quad (one star = 4 vertices)
        let i = ctx.index * 4;

        // If star is active, create a visible quad
        // Calculate perspective scale factor
        let scale = NEAR_PLANE / self.distance;

        // Calculate projected screen position
        let screen_x = self.position.x * scale * ctx.aspect_ratio + ctx.width as f32 / 2.0;
        let screen_y = self.position.y * scale + ctx.height as f32 / 2.0;

        // Depth ratio for color (farther stars are dimmer)
        let depth_ratio = (self.distance - NEAR_PLANE) / (FAR_PLANE - NEAR_PLANE);
        let brightness = ((1.0 - depth_ratio) * 255.0) as u8;

        // Calculate radius based on distance
        let radius = STAR_RADIUS * scale;

        let darkness = 255 - brightness;
        let adjusted_color = Color::rgb(
            ctx.color.r.saturating_sub(darkness),
            ctx.color.g.saturating_sub(darkness),
            ctx.color.b.saturating_sub(darkness),
        );

        let tex_x: f32 = ctx.texture_size.x as f32;
        let tex_y: f32 = ctx.texture_size.y as f32;

        for j in 0..4 {
            ctx.vertices[i + j].color = adjusted_color;
        }

        ctx.vertices[i].position = Vector2f::new(screen_x - radius, screen_y - radius);
        ctx.vertices[i + 1].position = Vector2f::new(screen_x + radius, screen_y - radius);
        ctx.vertices[i + 2].position = Vector2f::new(screen_x + radius, screen_y + radius);
        ctx.vertices[i + 3].position = Vector2f::new(screen_x - radius, screen_y + radius);

        ctx.vertices[i].tex_coords = Vector2f::new(0.0, 0.0);
        ctx.vertices[i + 1].tex_coords = Vector2f::new(tex_x, 0.0);
        ctx.vertices[i + 2].tex_coords = Vector2f::new(tex_x, tex_y);
        ctx.vertices[i + 3].tex_coords = Vector2f::new(0.0, tex_y);
    }
}

impl Stars {
    pub fn new(video: VideoMode, amount: usize, sprite_path: Option<PathBuf>) -> SfResult<Self> {
        let (texture, texture_color) = Self::create_star_texture(sprite_path)?;

        info!(
            "Star texture dimensions: {}x{}",
            texture.size().x,
            texture.size().y
        );

        let new_star = Star::new();
        let mut stars: Vec<Star> = vec![new_star; amount];
        stars
            .par_iter_mut()
            .for_each(|star| star.randomize(video.width, video.height));

        let mut star_vertices = vec![Vertex::default(); amount * 4];
        let mut point_vertices = vec![Vertex::default(); amount];

        star_vertices.par_iter_mut().for_each(|vertex| {
            vertex.color = Color::TRANSPARENT;
        });
        point_vertices.par_iter_mut().for_each(|vertex| {
            vertex.color = Color::TRANSPARENT;
        });

        let star_vertices_buf =
            VertexBuffer::new(PrimitiveType::QUADS, amount * 4, VertexBufferUsage::STREAM)?;

        let mut stars = Stars {
            stars,
            star_vertices_buf,
            star_vertices,
            video,
            speed: DEFAULT_SPEED,
            last_sorted_frame: 0,
            texture_size: texture.size(),
            texture,
            texture_color,
        };

        stars.sort(0);
        stars.update_vertex_ranges(&stars.get_update_ranges(0))?;

        Ok(stars)
    }

    fn find_index_zero_distance(&self) -> (usize, Option<&Star>) {
        self.stars
            .iter()
            .enumerate()
            .rev()
            .find(|(i, s)| s.distance < 0.0)
            .map(|(i, s)| (i, Some(s)))
            .unwrap_or((0, None))
    }

    fn create_star_texture(sprite_path: Option<PathBuf>) -> SfResult<(FBox<Texture>, Color)> {
        let star_image = match sprite_path {
            None => Image::from_memory(include_bytes!("../../../resources/star.png"))?,
            Some(p) => Image::from_file(p.to_str().expect("could not convert path to str"))?,
        };

        let center_x = star_image.size().x / 2;
        let center_y = star_image.size().y / 2;
        let center_color = star_image
            .pixel_at(center_x, center_y)
            .expect("could not get center color of star sprite");

        let mut texture = Texture::from_image(&star_image, IntRect::default())?;
        texture.set_smooth(true);

        Ok((texture, center_color))
    }

    fn star_chunks(&self) -> usize {
        self.stars.len().div_ceil(rayon::current_num_threads())
    }

    pub fn sort(&mut self, frame: u64) {
        self.stars
            .sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());
        self.last_sorted_frame = frame;
    }

    fn adjust_speed(&mut self, add_speed: f32, modifier: bool, fps_limit: u64) {
        let bounds = fps_limit as f32;
        self.speed += add_speed * if modifier { 10.0 } else { 1.0 };
        self.speed = self.speed.clamp(-bounds, bounds);
    }

    fn update_vertex_ranges(&mut self, ranges: &[(usize, usize)]) -> SfResult<()> {
        let aspect_ratio = self.video.width as f32 / self.video.height as f32;

        // Update vertices for each range
        for &(start, end) in ranges {
            // Skip empty ranges
            if start >= end {
                continue;
            }

            let range_size = end - start;
            let chunk_size = range_size.div_ceil(rayon::current_num_threads());

            // Create chunks based on the range
            self.stars[start..end]
                .par_chunks(chunk_size)
                .enumerate()
                .for_each(|(chunk_index, chunk)| {
                    // SAFETY: We're creating a mutable reference to the vector, but using
                    // it only for specific star's elements based on index
                    let vertices_ref = unsafe { please_mutable_ref_vec(&self.star_vertices) };
                    for (i, star) in chunk.iter().enumerate() {
                        // Calculate the absolute index in the stars array
                        let absolute_index = start + chunk_index * chunk_size + i;

                        let mut ctx = StarRenderCtx {
                            width: self.video.width,
                            height: self.video.height,
                            vertices: vertices_ref,
                            index: absolute_index, // Use the absolute index here
                            texture_size: &self.texture_size,
                            color: &self.texture_color,
                            aspect_ratio,
                        };

                        star.update_vertices(&mut ctx);
                    }
                });

            // Update only this section of the vertex buffer
            self.star_vertices_buf
                .update(&self.star_vertices[start * 4..end * 4], (start * 4) as u32)?;
        }

        Ok(())
    }

    fn get_update_ranges(&self, frame: u64) -> Vec<(usize, usize)> {
        let star_count = self.stars.len();

        let (nearest_idx, _) = self.find_index_zero_distance();
        let mut ranges_to_update = Vec::new();

        for (range_percent, frame_interval) in UPDATE_TIERS {
            if frame % *frame_interval != 0 {
                continue;
            }

            let lo_q: f32 = range_percent.start as f32 / 100.0;
            let hi_q: f32 = range_percent.end as f32 / 100.0;
            if frame % 47 == 0 {
                debug!("q {frame}: {lo_q} {hi_q}");
            }

            let mut near = (nearest_idx + (star_count as f32 * lo_q).ceil() as usize) % star_count;
            let mut far = (nearest_idx + (star_count as f32 * hi_q).ceil() as usize) % star_count;

            if far == 0 {
                far = star_count;
            }

            if far < near {
                ranges_to_update.push((near, far));
            } else {
                ranges_to_update.push((near, star_count));
                ranges_to_update.push((0, far));
            }
        }

        if frame % 47 == 0 {
            debug!("update_ranges {frame}: {ranges_to_update:?}");
        }

        for range in &ranges_to_update {
            assert!(range.1 <= self.stars.len())
        }

        ranges_to_update
    }
}

impl<'s> ComprehensiveElement<'s> for Stars {
    fn update(&mut self, counters: &Counter, info: &mut Info<'s>) {
        if self.speed == 0.0 {
            return;
        }

        // Update all star positions (cheap operation)
        let chunk_size = self.star_chunks();
        self.stars.par_chunks_mut(chunk_size).for_each(|chunk| {
            for star in chunk {
                star.update(self.speed, self.video.width, self.video.height);
            }
        });

        self.update_vertex_ranges(&self.get_update_ranges(counters.frames))
            .unwrap_or_else(|e| {
                error!("Error updating vertices: {}", e);
            });

        info.set_custom_info("near_star_idx", self.find_index_zero_distance().0);
    }

    fn draw_with(
        &mut self,
        sfml_w: &mut FBox<RenderWindow>,
        _egui_w: &mut bewegrs::egui_sfml::SfEgui,
        _counters: &Counter,
        _info: &mut Info<'s>,
    ) {
        let mut states = sfml::graphics::RenderStates::DEFAULT;
        states.texture = Some(&*self.texture);

        sfml_w.draw_with_renderstates(&*self.star_vertices_buf, &states);
    }

    fn z_level(&self) -> u16 {
        0
    }

    fn update_slow(&mut self, _counters: &Counter, info: &mut Info<'s>) {
        info.set_custom_info("last_sort", self.last_sorted_frame);
    }

    fn process_event(&mut self, event: &Event, counters: &Counter, info: &mut Info<'s>) {
        match event {
            Event::KeyPressed {
                code: Key::W,
                shift,
                ..
            } => {
                self.adjust_speed(0.1, *shift, counters.fps_limit);
                info.set_custom_info("speed", format_args!("{:.03}", self.speed));
            }
            Event::KeyPressed {
                code: Key::S,
                shift,
                ..
            } => {
                self.adjust_speed(-0.1, *shift, counters.fps_limit);
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

#[allow(invalid_reference_casting)]
#[allow(mutable_transmutes)]
#[allow(clippy::mut_from_ref)]
#[inline]
unsafe fn please_mutable_ref<T>(thing: &T) -> &mut T {
    unsafe { std::mem::transmute(thing) }
}

#[allow(invalid_reference_casting)]
#[allow(clippy::mut_from_ref)]
#[inline]
// seems redundant but is important for sized
unsafe fn please_mutable_ref_vec<T: Sized>(vec: &Vec<T>) -> &mut Vec<T> {
    unsafe { please_mutable_ref(vec) }
}
