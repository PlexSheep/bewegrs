use sfml::{
    SfResult,
    graphics::{Color, Font, Image, IntRect, RenderTarget, RenderWindow, Sprite, Texture},
    window::{Event, Key, Style, VideoMode},
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
