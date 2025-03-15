use egui_sfml::{DrawInput, SfEgui};
use sfml::cpp::FBox;
use sfml::graphics::{Color, Font, RenderTarget, RenderWindow, Text, Transformable};
use sfml::system::Vector2f;
use sfml::window::VideoMode;

use crate::counters::Counters;

#[derive(Default)]
pub enum InfoElementType {
    Egui,
    Overlay,
    #[default]
    None,
}

impl InfoElementType {
    fn next(&mut self) {
        *self = match self {
            Self::Overlay => Self::None,
            Self::None => Self::Egui,
            Self::Egui => Self::Overlay,
        };
    }
}

pub struct InfoElement<'s> {
    info_element_type: InfoElementType,
    overlay: Text<'s>,
}

impl<'s> InfoElement<'s> {
    pub fn new(font: &'s FBox<Font>, video: &VideoMode) -> Self {
        let mut overlay = Text::new("", font, 15);
        overlay.set_fill_color(Color::rgb(200, 200, 200));
        overlay.set_outline_color(Color::rgb(20, 20, 20));
        overlay.set_outline_thickness(1.0);
        overlay.set_position(Vector2f::new(
            video.width as f32 * 0.01,
            video.height as f32 - (video.height as f32 * 0.11),
        ));
        Self {
            info_element_type: Default::default(),
            overlay,
        }
    }
    pub fn prepare_draw(
        &mut self,
        window: &mut FBox<RenderWindow>,
        egui_window: &mut SfEgui,
        counters: &Counters,
    ) -> DrawInput {
        self.overlay.set_string(&counters.text);
        egui_window
            .run(window, |_rw, ctx| {
                let win = egui::Window::new("Info");
                win.show(ctx, |ui| {
                    ui.label(&counters.text);
                });
            })
            .unwrap()
    }

    pub fn draw_with(
        &mut self,
        window: &mut FBox<RenderWindow>,
        egui_window: &mut SfEgui,
        counters: &Counters,
    ) {
        match self.info_element_type {
            InfoElementType::None => (),
            InfoElementType::Egui => {
                let di = self.prepare_draw(window, egui_window, counters);
                egui_window.draw(di, window, None);
            }
            InfoElementType::Overlay => window.draw(&self.overlay),
        }
    }

    pub fn update_slow(&mut self, counters: &Counters) {
        self.overlay.set_string(&counters.text);
    }

    pub fn next_type(&mut self) {
        self.info_element_type.next()
    }
}
