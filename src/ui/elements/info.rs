use egui_sfml::{DrawInput, SfEgui};
use sfml::cpp::FBox;
use sfml::graphics::{Color, Font, RenderTarget, RenderWindow, Text, Transformable};
use sfml::system::Vector2f;
use sfml::window::{Key, VideoMode};
use tracing::debug;

use crate::counters::Counters;
use crate::ui::ComprehensiveElement;

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
    pub const DEFAULT_NAME: &'static str = "Info";

    pub fn new(font: &'s FBox<Font>, video: &VideoMode, counters: &Counters) -> Self {
        let mut overlay = Text::new(&counters.text, font, 17);
        debug!("info bounds: {:?}", overlay.global_bounds());
        overlay.set_fill_color(Color::rgb(200, 200, 200));
        overlay.set_outline_color(Color::rgb(20, 20, 20));
        overlay.set_outline_thickness(1.0);
        overlay.set_position(Vector2f::new(
            video.width as f32 * 0.005,
            video.height as f32 * 0.005,
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
                let win = egui::Window::new("Info").fixed_size((300.0, 12.0));
                win.show(ctx, |ui| {
                    ui.label(&counters.text);
                });
            })
            .unwrap()
    }

    pub fn next_type(&mut self) {
        self.info_element_type.next()
    }
}

impl<'s> ComprehensiveElement<'s> for InfoElement<'s> {
    fn draw_with(
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

    fn update_slow(&mut self, counters: &Counters) {
        self.overlay.set_string(&counters.text);
    }
    fn process_event(&mut self, event: &sfml::window::Event) {
        match event {
            sfml::window::Event::KeyPressed { code: Key::F10, .. } => {
                self.info_element_type.next();
            }
            _ => (),
        }
    }
}
