use std::collections::HashMap;

use egui::Id;
use egui_sfml::SfEgui;
use sfml::cpp::FBox;
use sfml::graphics::{Font, RenderWindow};
use sfml::window::{Event, VideoMode};
use sfml::SfResult;

use crate::counters::Counters;

use self::elements::info::InfoElement;

pub mod elements;
pub mod nativeui;

pub trait ComprehensiveElement<'s>: 's {
    fn draw_with(
        &mut self,
        sfml_w: &mut FBox<RenderWindow>,
        egui_w: &mut SfEgui,
        counters: &Counters,
    );

    #[allow(unused_variables)]
    fn process_event(&mut self, event: &Event) {}
    #[allow(unused_variables)]
    fn update_slow(&mut self, counters: &Counters) {}
}

pub struct ComprehensiveUi<'s> {
    egui_window: SfEgui,
    pub font: &'s FBox<Font>,
    elements: HashMap<Id, Box<dyn ComprehensiveElement<'s>>>,
}

impl<'s> ComprehensiveUi<'s> {
    pub fn add_event(&mut self, event: &Event) {
        self.egui_window.add_event(event);

        for (_id, element) in self.elements.iter_mut() {
            element.process_event(event);
        }
    }

    pub fn build(
        window: &FBox<RenderWindow>,
        font: &'s FBox<Font>,
        video: &VideoMode,
        counters: &Counters,
    ) -> SfResult<Self> {
        let mut gui = Self {
            egui_window: SfEgui::new(window),
            elements: HashMap::new(),
            font,
        };
        gui.elements.insert(
            Id::new("Info Panel"),
            Box::new(InfoElement::new(font, video, counters)),
        );
        Ok(gui)
    }

    pub fn add(&mut self, id: impl Into<Id>, element: Box<dyn ComprehensiveElement<'s>>) {
        self.elements.insert(id.into(), element);
    }

    pub fn get(&mut self, id: impl Into<Id>) -> Option<&dyn ComprehensiveElement<'s>> {
        self.elements.get(&id.into()).map(|a| &**a)
    }

    pub fn remove(&mut self, id: impl Into<Id>) -> Option<Box<dyn ComprehensiveElement<'s>>> {
        self.elements.remove(&id.into())
    }

    pub fn draw_with(&mut self, window: &mut FBox<RenderWindow>, counters: &Counters) {
        for (_id, element) in self.elements.iter_mut() {
            element.draw_with(window, &mut self.egui_window, counters);
        }
    }

    pub fn update_slow(&mut self, counters: &Counters) {
        for (_id, element) in self.elements.iter_mut() {
            element.update_slow(counters);
        }
    }
}
