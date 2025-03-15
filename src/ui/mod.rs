use egui_sfml::{DrawInput, SfEgui};
use sfml::cpp::FBox;
use sfml::graphics::RenderWindow;
use sfml::window::Event;

use crate::counters::Counters;

pub mod nativeui;

#[derive(Default)]
struct MetaWindowData {
    input: String,
    messages: Vec<String>,
}

pub struct ComprehensiveUi {
    pub egui_window: SfEgui,
}

impl ComprehensiveUi {
    pub fn add_event(&mut self, event: &Event) {
        self.egui_window.add_event(event);
    }

    pub fn new(window: &FBox<RenderWindow>) -> Self {
        Self {
            egui_window: SfEgui::new(window),
        }
    }

    pub fn prepare_draw(
        &mut self,
        window: &mut FBox<RenderWindow>,
        counters: &Counters,
    ) -> DrawInput {
        self.egui_window
            .run(window, |_rw, ctx| {
                let win = egui::Window::new("Info");
                win.show(ctx, |ui| {
                    Self::info_win(ctx, ui, counters);
                });
            })
            .unwrap()
    }

    pub fn draw(&mut self, di: DrawInput, window: &mut FBox<RenderWindow>) {
        self.egui_window.draw(di, window, None);
    }

    fn info_win(_ctx: &egui::Context, ui: &mut egui::Ui, counters: &Counters) {
        ui.label(&counters.text);
    }
}
