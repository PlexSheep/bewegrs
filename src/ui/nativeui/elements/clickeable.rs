use sfml::{
    graphics::{Color, CustomShape, Drawable, Font, RenderTarget, Shape, Text, Transformable},
    system::{Vector2f, Vector2i},
    window::Event,
};

use crate::shapes::RectRoundShape;

use super::NativeElement;

pub struct Clickable<'s> {
    pub shape: CustomShape<'s>,
    text: Option<Text<'s>>,
    is_hovered: bool,
    is_pressed: bool,
}

impl<'s> Clickable<'s> {
    pub fn new_rect_round(width: f32, height: f32, radius: f32) -> Self {
        let rect_shape = RectRoundShape::new(width, height, radius);
        let mut shape = CustomShape::new(Box::new(rect_shape));

        // Set default colors
        shape.set_fill_color(Color::rgb(80, 80, 80));
        shape.set_outline_color(Color::rgb(120, 120, 120));
        shape.set_outline_thickness(4.0);

        Clickable {
            shape,
            text: None,
            is_hovered: false,
            is_pressed: false,
        }
    }

    pub fn with_text(mut self, text_str: &str, font: &'s Font, size: u32) -> Self {
        let mut text = Text::new(text_str, font, size);
        text.set_fill_color(Color::WHITE);

        // Center text in the button
        let text_rect = text.local_bounds();
        text.set_origin((text_rect.width / 2.0, text_rect.height / 2.0));

        let shape_bounds = self.shape.global_bounds();
        text.set_position((
            shape_bounds.left + shape_bounds.width / 2.0,
            shape_bounds.top + shape_bounds.height / 2.0 - 5.0, // Small visual adjustment
        ));

        self.text = Some(text);
        self
    }

    // Returns true if clicked (pressed and released on the element)
    pub fn update(&mut self, event: &Event, mouse_pos: impl Into<Vector2f>) -> bool {
        let contains = self.contains_point(mouse_pos);
        let old_hovered = self.is_hovered;
        let old_pressed = self.is_pressed;
        let mut clicked = false;

        match event {
            Event::MouseMoved { .. } => {
                self.is_hovered = contains;
                if self.is_pressed && !contains {
                    self.is_pressed = false;
                }
            }
            Event::MouseButtonPressed { .. } => {
                if contains {
                    self.is_pressed = true;
                }
            }
            Event::MouseButtonReleased { .. } => {
                if self.is_pressed && contains {
                    clicked = true;
                }
                self.is_pressed = false;
            }
            _ => {}
        }

        // Update visual appearance if state changed
        if old_hovered != self.is_hovered || old_pressed != self.is_pressed {
            self.update_appearance();
        }

        clicked
    }

    fn update_appearance(&mut self) {
        if self.is_pressed {
            self.shape.set_fill_color(Color::rgb(60, 60, 60));
            self.shape.set_outline_color(Color::rgb(180, 180, 180));
        } else if self.is_hovered {
            self.shape.set_fill_color(Color::rgb(100, 100, 100));
            self.shape.set_outline_color(Color::rgb(160, 160, 160));
        } else {
            self.shape.set_fill_color(Color::rgb(80, 80, 80));
            self.shape.set_outline_color(Color::rgb(120, 120, 120));
        }
    }

    pub fn draw(&self, target: &mut dyn RenderTarget) {
        target.draw(&self.shape);
        if let Some(text) = &self.text {
            target.draw(text);
        }
    }
}

impl Drawable for Clickable<'_> {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn RenderTarget,
        states: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        self.shape.draw(target, states);
        if let Some(text) = self.text.as_ref() {
            text.draw(target, states);
        }
    }
}

impl<'s> NativeElement<'s> for Clickable<'s> {
    fn set_position(&mut self, position: impl Into<Vector2f>) {
        self.shape.set_position(position);

        // Update text position if it exists
        if let Some(text) = &mut self.text {
            let shape_bounds = self.shape.global_bounds();
            text.set_position((
                shape_bounds.left + shape_bounds.width / 2.0,
                shape_bounds.top + shape_bounds.height / 2.0 - 5.0,
            ));
        }
    }

    fn position(&self) -> Vector2f {
        self.shape.position()
    }

    fn contains_point(&self, point: impl Into<Vector2f>) -> bool {
        let bounds = self.shape.global_bounds();
        let point: Vector2f = point.into();

        point.x >= bounds.left
            && point.x <= bounds.left + bounds.width
            && point.y >= bounds.top
            && point.y <= bounds.top + bounds.height
    }

    fn handle_event(&mut self, event: &Event, mouse_pos: Vector2i) -> bool {
        true
    }
}
