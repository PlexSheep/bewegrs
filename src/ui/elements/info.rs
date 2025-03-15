use std::collections::HashMap;
use std::fmt::{Display, Write};

use egui_sfml::{DrawInput, SfEgui};
use sfml::cpp::FBox;
use sfml::graphics::{Color, Font, RenderTarget, RenderWindow, Text, Transformable};
use sfml::system::Vector2f;
use sfml::window::{Key, VideoMode};
use tracing::{debug, error};

use crate::counters::Counters;
use crate::ui::ComprehensiveElement;

#[derive(Default)]
pub enum InfoElementKind {
    Egui,
    #[default]
    Overlay,
    None,
}

impl InfoElementKind {
    fn next(&mut self) {
        *self = match self {
            Self::Overlay => Self::None,
            Self::None => Self::Egui,
            Self::Egui => Self::Overlay,
        };
    }
}

pub struct InfoElement<'s> {
    kind: InfoElementKind,
    overlay: Text<'s>,
    custom_info: HashMap<String, String>,
}

impl<'s> InfoElement<'s> {
    pub const DEFAULT_NAME: &'static str = "Info";

    pub fn new<const N: usize>(
        font: &'s FBox<Font>,
        video: &VideoMode,
        counters: &Counters<N>,
    ) -> Self {
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
            kind: Default::default(),
            overlay,
            custom_info: HashMap::new(),
        }
    }

    pub fn set_custom_info(&mut self, key: impl Display, value: impl Display) {
        self.custom_info.insert(key.to_string(), value.to_string());
    }

    pub fn prepare_draw<const N: usize>(
        &mut self,
        window: &mut FBox<RenderWindow>,
        egui_window: &mut SfEgui,
        counters: &Counters<N>,
    ) -> DrawInput {
        self.overlay.set_string(&self.get_text(counters));
        egui_window
            .run(window, |_rw, ctx| {
                let win = egui::Window::new("Info").fixed_size((300.0, 12.0));
                win.show(ctx, |ui| {
                    ui.label(self.get_text(counters));
                });
            })
            .unwrap()
    }

    pub fn next_kind(&mut self) {
        self.kind.next()
    }

    pub fn set_kind(&mut self, kind: InfoElementKind) {
        self.kind = kind;
    }

    fn get_text<const N: usize>(&self, counters: &Counters<N>) -> String {
        let mut buf: String = format!("{}\n", counters.text);
        for (key, value) in &self.custom_info {
            if let Err(e) = writeln!(buf, "{key}: {value}") {
                error!("could not write to format buffer for info widget: {e}");
            }
        }
        buf
    }
}

impl<'s, const N: usize> ComprehensiveElement<'s, N> for InfoElement<'s> {
    fn draw_with(
        &mut self,
        window: &mut FBox<RenderWindow>,
        egui_window: &mut SfEgui,
        counters: &Counters<N>,
    ) {
        match self.kind {
            InfoElementKind::None => (),
            InfoElementKind::Egui => {
                let di = self.prepare_draw(window, egui_window, counters);
                egui_window.draw(di, window, None);
            }
            InfoElementKind::Overlay => {
                let _ = self.prepare_draw(window, egui_window, counters);
                window.draw(&self.overlay)
            }
        }
    }

    fn update_slow(&mut self, counters: &Counters<N>) {
        self.overlay.set_string(&counters.text);
    }
    fn process_event(&mut self, event: &sfml::window::Event) {
        if let sfml::window::Event::KeyPressed { code: Key::F10, .. } = event {
            self.kind.next();
        }
    }
    fn z_level(&self) -> u16 {
        super::super::UI_LEVEL
    }
}
