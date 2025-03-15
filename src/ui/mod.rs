use egui_sfml::SfEgui;
use sfml::cpp::FBox;
use sfml::graphics::{Font, RenderWindow};
use sfml::window::{Event, Key, VideoMode};
use sfml::SfResult;

use crate::counters::Counters;

use self::elements::info::InfoElement;

pub mod elements;
pub mod nativeui;

pub struct ComprehensiveUi<'s> {
    egui_window: SfEgui,
    info_element: InfoElement<'s>,
    video: VideoMode,
    pub font: &'s FBox<Font>,
}

impl<'s> ComprehensiveUi<'s> {
    pub fn add_event(&mut self, event: &Event) {
        self.egui_window.add_event(event);

        match event {
            Event::KeyPressed { code: Key::F10, .. } => self.info_element.next_type(),
            _ => (),
        }
    }

    pub fn build(
        window: &FBox<RenderWindow>,
        font: &'s FBox<Font>,
        video: VideoMode,
        counters: &Counters,
    ) -> SfResult<Self> {
        let gui = Self {
            video,
            egui_window: SfEgui::new(window),
            info_element: InfoElement::new(font, &video, counters),
            font,
        };
        Ok(gui)
    }

    pub fn draw_with(&mut self, window: &mut FBox<RenderWindow>, counters: &Counters) {
        self.info_element
            .draw_with(window, &mut self.egui_window, counters)
    }

    pub fn update_slow(&mut self, counters: &Counters) {
        self.info_element.update_slow(counters);
    }
}
