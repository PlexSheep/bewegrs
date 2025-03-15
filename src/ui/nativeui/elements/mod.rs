use sfml::graphics::Drawable;
use sfml::system::{Vector2f, Vector2i};
use sfml::window::Event;

pub mod clickeable;

pub trait NativeElement<'s>: Drawable {
    fn set_position(&mut self, position: impl Into<Vector2f>);
    fn position(&self) -> Vector2f;
    fn contains_point(&self, point: impl Into<Vector2f>) -> bool;
    fn handle_event(&mut self, event: &Event, mouse_pos: Vector2i) -> bool;
}
