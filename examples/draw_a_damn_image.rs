use sfml::{
    SfResult,
    graphics::{Image, IntRect, RenderTarget, RenderWindow, Sprite, Texture},
    system::{Time, sleep},
    window::{Style, VideoMode},
};
fn main() -> SfResult<()> {
    let video = VideoMode::desktop_mode();
    let mut window = RenderWindow::new(video, "Custom shape", Style::DEFAULT, &Default::default())?;

    let texture = Texture::from_image(
        &*Image::from_memory(include_bytes!("../resources/logo.png"))?,
        IntRect::default(),
    )?;
    let image = Sprite::with_texture(&texture);

    // dont even need a loop since nothing changes

    window.draw(&image);

    window.display();

    sleep(Time::seconds(4.0)); // just so you can look at it

    Ok(())
}
