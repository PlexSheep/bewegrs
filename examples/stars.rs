use sfml::{
    cpp::FBox,
    graphics::{
        Color, FloatRect, Font, Image, IntRect, PrimitiveType, RectangleShape, RenderTarget,
        RenderWindow, Texture, Transformable, Vertex, VertexBuffer, VertexBufferUsage,
    },
    system::Vector2f,
    window::{Event, Key, Style, VideoMode},
    SfResult,
};
use tracing::{debug, info};

use bewegrs::{
    counters::Counters,
    setup,
    ui::{elements::info::Info, ComprehensiveElement, ComprehensiveUi},
};

const MAX_FPS: usize = 60;
const BG: Color = Color::rgb(30, 20, 20);
const STAR_AMOUNT: usize = 800_000;
const DEFAULT_SPEED: f32 = 0.8;

// Star configuration
const STAR_RADIUS: f32 = 30.0;
const FAR_PLANE: f32 = 800.0;
const NEAR_PLANE: f32 = 5.5;
const SPREAD: f32 = FAR_PLANE * 40.0;

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

    let mut font = Font::new()?;
    font.load_from_memory_static(include_bytes!("../resources/sansation.ttf"))?;

    let profile_image = &*Image::from_memory(include_bytes!("../resources/profile.png"))?;
    let mut texture = Texture::from_image(profile_image, IntRect::default())?;
    texture.set_smooth(true);

    let mut gui = ComprehensiveUi::build(&window, &font, &video, &counter)?;
    gui.set_no_cursor(&mut window, true);

    gui.info
        .set_logo(&texture, "Christoph J. Scherr\nsoftware@cscherr.de")?;

    let stars = Stars::new(video, STAR_AMOUNT)?;
    gui.info.set_custom_info("stars", stars.stars.len());
    gui.info.set_custom_info("speed", stars.speed);
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

// Simple star data without the SFML object
struct Star {
    position: Vector2f, // World-space position (centered around 0,0)
    distance: f32,      // z-coordinate
    active: bool,       // Whether star is active/visible
}

impl Star {
    fn new(width: u32, height: u32) -> Self {
        let mut star = Star {
            position: Vector2f::new(0.0, 0.0),
            distance: 0.0,
            active: true,
        };

        star.rand_pos(width, height);
        star.rand_distance();
        star
    }

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

    fn update(&mut self, width: u32, height: u32, speed: f32) {
        // Decrease distance (move closer)
        self.distance -= speed;

        // If star gets too close, reset it
        if self.distance <= NEAR_PLANE {
            self.rand_pos(width, height);
            self.rand_distance();
        }
        // If star gets too far, reset it
        if self.distance >= FAR_PLANE {
            self.rand_pos(width, height);
            self.rand_distance();
        }

        // Check visibility
        self.active = self.is_visible(width, height);
    }

    fn is_visible(&self, _width: u32, _height: u32) -> bool {
        // Calculate perspective scale factor
        let scale = NEAR_PLANE / self.distance;

        // we could also check if the star is in the viewport

        // Check if star is big enough to see
        scale > 0.01
    }

    // Create vertices for this star (a quad made of 4 vertices)
    fn create_vertices(&self, width: u32, height: u32, vertices: &mut [Vertex], index: usize) {
        // Calculate perspective scale factor
        let scale = NEAR_PLANE / self.distance;

        // Depth ratio for color (farther stars are dimmer)
        let depth_ratio = (self.distance - NEAR_PLANE) / (FAR_PLANE - NEAR_PLANE);
        let brightness = ((1.0 - depth_ratio) * 255.0) as u8;

        // Calculate projected screen position
        let screen_x = self.position.x * scale + width as f32 / 2.0;
        let screen_y = self.position.y * scale + height as f32 / 2.0;

        // Calculate radius based on distance
        let radius = STAR_RADIUS * scale;

        // Create the 4 vertices of the quad (one star = 4 vertices)
        let i = index * 4;

        // If star is active, create a visible quad
        if self.active {
            let color = Color::rgb(brightness, brightness, brightness);

            // Top-left vertex
            vertices[i].position = Vector2f::new(screen_x - radius, screen_y - radius);
            vertices[i].color = color;

            // Top-right vertex
            vertices[i + 1].position = Vector2f::new(screen_x + radius, screen_y - radius);
            vertices[i + 1].color = color;

            // Bottom-right vertex
            vertices[i + 2].position = Vector2f::new(screen_x + radius, screen_y + radius);
            vertices[i + 2].color = color;

            // Bottom-left vertex
            vertices[i + 3].position = Vector2f::new(screen_x - radius, screen_y + radius);
            vertices[i + 3].color = color;
        }
        // If star is not active, create an invisible quad
        else {
            let transparent = Color::rgba(0, 0, 0, 0); // Fully transparent

            // Set all 4 vertices to be invisible (zero size, transparent)
            for j in 0..4 {
                vertices[i + j].position = Vector2f::new(0.0, 0.0);
                vertices[i + j].color = transparent;
            }
        }
    }
}

struct Stars {
    stars: Vec<Star>,
    vertex_buffer: FBox<VertexBuffer>,
    vertices: Vec<Vertex>,
    video: VideoMode,
    speed: f32,
}

impl Stars {
    pub fn new(video: VideoMode, amount: usize) -> SfResult<Self> {
        // Create stars
        let mut stars: Vec<Star> = Vec::with_capacity(amount);
        for _ in 0..amount {
            stars.push(Star::new(video.width, video.height));
        }

        // Sort stars by distance (farthest first for proper rendering)
        stars.sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());

        // Create a vertex array to store our quad data (4 vertices per star)
        let mut vertices = vec![Vertex::default(); amount * 4];

        // Initialize all vertices as transparent (this is crucial)
        for vertex in &mut vertices {
            vertex.color = Color::rgba(0, 0, 0, 0); // Fully transparent
        }

        // Create the vertex buffer
        let mut vertex_buffer =
            VertexBuffer::new(PrimitiveType::QUADS, amount * 4, VertexBufferUsage::DYNAMIC)?;

        // Initialize vertex data
        for (i, star) in stars.iter().enumerate() {
            star.create_vertices(video.width, video.height, &mut vertices, i);
        }

        // Update the vertex buffer with initial data
        vertex_buffer.update(&vertices, 0)?;

        Ok(Stars {
            stars,
            vertex_buffer,
            vertices,
            video,
            speed: DEFAULT_SPEED,
        })
    }

    fn update_vertices(&mut self) -> SfResult<()> {
        // Clear all vertices by setting them to transparent
        for vertex in &mut self.vertices {
            vertex.color = Color::rgba(0, 0, 0, 0); // Fully transparent
        }

        // Sort stars by distance (farthest first for proper overlay)
        self.stars
            .sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());

        // Update all vertices in the vertices array
        for (i, star) in self.stars.iter().enumerate() {
            star.create_vertices(self.video.width, self.video.height, &mut self.vertices, i);
        }

        // Update the vertex buffer with the new vertex data
        // This updates all vertices, including the "invisible" ones
        self.vertex_buffer.update(&self.vertices, 0)?;

        Ok(())
    }
}

impl<'s, const N: usize> ComprehensiveElement<'s, N> for Stars {
    fn update(&mut self, _counters: &Counters<N>, _info: &mut Info<'s>) {
        // Update star positions
        for star in self.stars.iter_mut() {
            star.update(self.video.width, self.video.height, self.speed);
        }

        // Update vertex buffer
        if let Err(e) = self.update_vertices() {
            eprintln!("Error updating vertex buffer: {:?}", e);
        }
    }

    fn draw_with(
        &mut self,
        sfml_w: &mut FBox<RenderWindow>,
        _egui_w: &mut egui_sfml::SfEgui,
        _counters: &Counters<N>,
        _info: &mut Info<'s>,
    ) {
        // Draw all stars with a single draw call
        sfml_w.draw(&*self.vertex_buffer);
    }

    fn z_level(&self) -> u16 {
        0
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
            _ => (),
        }
    }
}
