use sfml::{
    graphics::{
        glsl::Vec2, CircleShape, Color, CustomShape, Font, Image, IntRect, RectangleShape,
        RenderTarget, RenderWindow, Shape, Sprite, Texture, Transformable,
    },
    window::{Event, Key, Style, VideoMode},
    SfResult,
};
fn main() -> SfResult<()> {
    let video = VideoMode::fullscreen_modes()[0];
    let mut window = RenderWindow::new(
        video,
        "Custom shape",
        Style::DEFAULT | Style::FULLSCREEN,
        &Default::default(),
    )?;
    window.set_framerate_limit(60);

    let mut font = Font::new()?;
    font.load_from_memory_static(include_bytes!("../resources/sansation.ttf"))?;

    let texture = Texture::from_image(
        &*Image::from_memory(include_bytes!("../resources/logo.png"))?,
        IntRect::default(),
    )?;
    let image = Sprite::with_texture(&texture);

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

        window.draw(&image);

        window.display();
    }
    Ok(())
}
