use std::collections::HashMap;

use egui_sfml::SfEgui;
use sfml::cpp::FBox;
use sfml::graphics::{Font, RenderWindow};
use sfml::window::{Event, VideoMode};

use crate::counter::Counter;
use crate::errors::BwgResult;

use self::elements::info::Info;

pub const UI_Z_LEVEL: u16 = 20000;
pub const DEFAULT_Z_LEVEL: u16 = 1000;

pub mod elements;
pub mod nativeui;

pub trait ComprehensiveElement<'s>: 's {
    fn z_level(&self) -> u16 {
        DEFAULT_Z_LEVEL
    }

    #[allow(unused_variables)]
    fn draw_with(
        &mut self,
        sfml_w: &mut FBox<RenderWindow>,
        egui_w: &mut SfEgui,
        counters: &Counter,
        info: &mut Info<'s>,
    ) {
    }

    #[allow(unused_variables)]
    fn process_event(&mut self, event: &Event, counters: &Counter, info: &mut Info<'s>) {}
    #[allow(unused_variables)]
    fn update_slow(&mut self, counters: &Counter, info: &mut Info<'s>) {}
    #[allow(unused_variables)]
    fn update(&mut self, counters: &Counter, info: &mut Info<'s>) {}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, Default)]
pub struct GElementID {
    inner: u128,
}

impl rand::distr::Distribution<GElementID> for rand::distr::StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> GElementID {
        GElementID {
            inner: rng.random(),
        }
    }
}

impl GElementID {
    pub fn new() -> Self {
        rand::random()
    }
}

pub struct ComprehensiveUi<'s> {
    egui_window: SfEgui,
    pub font: &'s FBox<Font>,
    pub info: Info<'s>,
    elements: HashMap<GElementID, Box<dyn ComprehensiveElement<'s>>>,
    pub counter: Counter,
}

impl<'s> ComprehensiveUi<'s> {
    pub fn add_event(&mut self, event: &Event) {
        self.egui_window.add_event(event);

        for element in self.elements.values_mut() {
            element.process_event(event, &self.counter, &mut self.info);
        }
        self.info.process_event(event);
    }

    pub fn build(
        window: &mut FBox<RenderWindow>,
        font: &'s FBox<Font>,
        video: &'s VideoMode,
        fps_limit: u64,
    ) -> BwgResult<Self> {
        let counters = Counter::start(fps_limit)?;
        window.set_framerate_limit(fps_limit as u32);

        let gui = Self {
            egui_window: SfEgui::new(window),
            elements: HashMap::new(),
            info: Info::new(font, video, &counters),
            font,
            counter: counters,
        };
        Ok(gui)
    }

    pub fn add(&mut self, element: Box<dyn ComprehensiveElement<'s>>) -> GElementID {
        let id = self.get_new_element_id();
        self.elements.insert(id, element);
        id
    }

    pub fn get(&self, id: &GElementID) -> Option<&dyn ComprehensiveElement<'s>> {
        self.elements.get(id).map(|v| &**v)
    }

    pub fn get_mut(&mut self, id: &GElementID) -> Option<&mut dyn ComprehensiveElement<'s>> {
        self.elements.get_mut(id).map(|v| &mut **v)
    }

    pub fn draw_with(&mut self, window: &mut FBox<RenderWindow>) {
        for element in self.elements.values_mut() {
            element.draw_with(window, &mut self.egui_window, &self.counter, &mut self.info);
        }
        self.info
            .draw_with(window, &mut self.egui_window, &self.counter);
    }

    pub fn update_slow(&mut self) {
        for element in self.elements.values_mut() {
            element.update_slow(&self.counter, &mut self.info);
        }
        self.info.update_slow(&self.counter);
    }

    pub fn update(&mut self) {
        for element in self.elements.values_mut() {
            element.update(&self.counter, &mut self.info);
        }
        self.info.update(&self.counter);
    }

    // BUG: this does not work
    pub fn set_no_cursor(&self, window: &mut FBox<RenderWindow>, arg: bool) {
        window.set_mouse_cursor_visible(arg);
        self.egui_window.context().set_cursor_icon(if arg {
            egui::CursorIcon::None
        } else {
            egui::CursorIcon::default()
        });
    }

    pub fn frame_start(&mut self) {
        self.counter.frame_start();
    }

    pub fn display(&mut self, window: &mut FBox<RenderWindow>) {
        self.counter.frame_prepare_display();
        window.display();
    }

    pub fn get_new_element_id(&self) -> GElementID {
        let mut id: GElementID;
        let mut guard = 0;
        loop {
            id = rand::random();

            if !self.elements.contains_key(&id) {
                break;
            }
            if guard > 20 {
                panic!(
                    "Could not find a new element id. This is almost certainly a super weird edge case, since the keyspace is 2^128 bit"
                )
            }
            guard += 1;
        }
        id
    }
}
