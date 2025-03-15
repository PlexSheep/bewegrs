use bewegrs::{counters::Counters, setup, ui::ComprehensiveUi};
use sfml::{
    SfResult,
    graphics::{CircleShape, Color, FloatRect, RenderTarget, RenderWindow, Shape, Transformable},
    system::Vector2f,
    window::{Event, Key, Style, VideoMode},
};
use tracing::debug;

const MAX_FPS: usize = 60;
const STAR_COUNT: usize = 5000;

// Star configuration
const SPREAD: f32 = 30.5;
const STAR_RADIUS: f32 = 20.5;
const FAR_PLANE: f32 = 10.0;
const NEAR_PLANE: f32 = 0.1;
const SPEED: f32 = 1.5;

struct Star {
    position: Vector2f,
    z: f32,
}

fn create_stars(count: usize, scale: f32, video: &VideoMode) -> Vec<Star> {
    let mut stars = Vec::with_capacity(count);

    let window_size =
        Vector2f::new(video.width as f32 / 2.0, video.height as f32 / 2.0) * NEAR_PLANE;
    let pos = -window_size * 0.25;
    let dimensions = window_size * 0.5;
    let star_free_zone = FloatRect::new(pos.x, pos.y, dimensions.x, dimensions.y);
    debug!("no stars zone: {:?}", star_free_zone);

    for _ in 0..count {
        let mut x;
        let mut y;
        let mut guard = 0;
        loop {
            // Generate random x,y position (centered around origin)
            x = (rand::random::<f32>() - 0.5) * (scale * video.width as f32);
            y = (rand::random::<f32>() - 0.5) * (scale * video.height as f32);
            if star_free_zone.contains((x, y).into()) || guard > 20 {
                break;
            }
            guard += 1;
        }

        // Random z depth between NEAR_PLANE and FAR_PLANE
        let z = (FAR_PLANE - NEAR_PLANE) * rand::random::<f32>() + NEAR_PLANE;

        stars.push(Star {
            position: Vector2f::new(x, y),
            z,
        });
    }

    stars.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap().reverse());

    stars
}

fn main() -> SfResult<()> {
    setup();

    let video = VideoMode::fullscreen_modes()[0];
    let mut window = RenderWindow::new(
        video,
        "Starfield",
        Style::DEFAULT | Style::FULLSCREEN,
        &Default::default(),
    )?;

    let mut counter = Counters::<MAX_FPS>::start()?;
    window.set_framerate_limit(MAX_FPS as u32);

    let mut font = sfml::graphics::Font::new()?;
    font.load_from_memory_static(include_bytes!("../resources/sansation.ttf"))?;

    let mut gui = ComprehensiveUi::build(&window, &font, &video, &counter)?;

    // Create stars
    let mut stars = create_stars(STAR_COUNT, SPREAD, &video);

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

        // Update GUI
        gui.update(&counter);
        if counter.frames % MAX_FPS as u64 == 1 {
            gui.update_slow(&counter);

            // Add some custom info to the UI
            gui.info.set_custom_info("stars", STAR_COUNT);
        }

        // Update star positions (z decreases as we move forward)
        let dt = 1.0 / MAX_FPS as f32;
        for star in &mut stars {
            star.z -= SPEED * dt;

            // If star gets too close, move it back to far plane
            if star.z < NEAR_PLANE {
                star.z = FAR_PLANE;

                // Optional: randomize x,y when star is recycled
                // star.position.x = (rand::random::<f32>() - 0.5) * FAR_PLANE;
                // star.position.y = (rand::random::<f32>() - 0.5) * FAR_PLANE;
            }
        }

        // Render
        window.clear(Color::BLACK);

        // Draw stars
        for star in &stars {
            // Apply perspective projection - closer stars appear larger and further from center
            let scale = NEAR_PLANE / star.z;

            // Depth ratio for color (farther stars are dimmer)
            let depth_ratio = (star.z - NEAR_PLANE) / (FAR_PLANE - NEAR_PLANE);
            let color_value = (255.0 * (1.0 - depth_ratio)) as u8;

            // Position with perspective and centered in window
            let screen_pos = Vector2f::new(
                star.position.x * scale + window.size().x as f32 / 2.0,
                star.position.y * scale + window.size().y as f32 / 2.0,
            );

            // Size based on distance
            let size = STAR_RADIUS * scale;

            // Only draw if in screen bounds and large enough to see
            if size > 0.1
                && screen_pos.x >= 0.0
                && screen_pos.x <= window.size().x as f32
                && screen_pos.y >= 0.0
                && screen_pos.y <= window.size().y as f32
            {
                let mut circle = CircleShape::new(size, 8);
                circle.set_position(screen_pos);
                circle.set_origin((size, size));
                circle.set_fill_color(Color::rgb(color_value, color_value, color_value));
                window.draw(&circle);
            }
        }

        // Draw GUI elements
        gui.draw_with(&mut window, &counter);

        counter.frame_prepare_display();
        window.display();
    }

    Ok(())
}
