use egui_sfml::SfEgui;
use sfml::cpp::FBox;
use sfml::graphics::{Font, RenderWindow};
use sfml::window::{Event, VideoMode};
use sfml::SfResult;

use crate::counters::Counters;

use self::elements::info::Info;

pub const UI_LEVEL: u16 = 20000;

pub mod elements;
pub mod nativeui;

pub trait ComprehensiveElement<'s, const N: usize>: 's {
    fn z_level(&self) -> u16;

    fn draw_with(
        &mut self,
        sfml_w: &mut FBox<RenderWindow>,
        egui_w: &mut SfEgui,
        counters: &Counters<N>,
        info: &mut Info<'s>,
    );

    #[allow(unused_variables)]
    fn process_event(&mut self, event: &Event, info: &mut Info<'s>) {}
    #[allow(unused_variables)]
    fn update_slow(&mut self, counters: &Counters<N>, info: &mut Info<'s>) {}
    #[allow(unused_variables)]
    fn update(&mut self, counters: &Counters<N>, info: &mut Info<'s>) {}
}

pub struct ComprehensiveUi<'s, const N: usize> {
    egui_window: SfEgui,
    pub font: &'s FBox<Font>,
    pub info: Info<'s>,
    elements: Vec<Box<dyn ComprehensiveElement<'s, N>>>,
}

impl<'s, const N: usize> ComprehensiveUi<'s, N> {
    pub fn add_event(&mut self, event: &Event) {
        self.egui_window.add_event(event);

        for element in self.elements.iter_mut() {
            element.process_event(event, &mut self.info);
        }
        self.info.process_event(event);
    }

    pub fn build(
        window: &FBox<RenderWindow>,
        font: &'s FBox<Font>,
        video: &'s VideoMode,
        counters: &Counters<N>,
    ) -> SfResult<Self> {
        let gui = Self {
            egui_window: SfEgui::new(window),
            elements: Vec::new(),
            info: Info::new(font, video, counters),
            font,
        };
        Ok(gui)
    }

    pub fn add(&mut self, element: Box<dyn ComprehensiveElement<'s, N>>) {
        self.elements.push(element);
        self.elements.sort_by_key(|a| a.z_level());
    }

    pub fn draw_with(&mut self, window: &mut FBox<RenderWindow>, counters: &Counters<N>) {
        for element in self.elements.iter_mut() {
            element.draw_with(window, &mut self.egui_window, counters, &mut self.info);
        }
        self.info.draw_with(window, &mut self.egui_window, counters);
    }

    pub fn update_slow(&mut self, counters: &Counters<N>) {
        for element in self.elements.iter_mut() {
            element.update_slow(counters, &mut self.info);
        }
        self.info.update_slow(counters);
    }

    pub fn update(&mut self, counters: &Counters<N>) {
        for element in self.elements.iter_mut() {
            element.update(counters, &mut self.info);
        }
        self.info.update(counters);
    }
}
