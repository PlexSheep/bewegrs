use egui_sfml::{DrawInput, SfEgui};
use sfml::cpp::FBox;
use sfml::graphics::RenderWindow;
use sfml::window::Event;

pub mod nativeui;

#[derive(Default)]
struct MetaWindowData {
    input: String,
    messages: Vec<String>,
}

pub struct ComprehensiveUi {
    pub egui_window: SfEgui,
    meta_window_data: MetaWindowData,
}

impl ComprehensiveUi {
    pub fn add_event(&mut self, event: &Event) {
        self.egui_window.add_event(event);
    }

    pub fn new(window: &FBox<RenderWindow>) -> Self {
        Self {
            egui_window: SfEgui::new(window),
            meta_window_data: MetaWindowData::default(),
        }
    }

    pub fn prepare_draw(&mut self, window: &mut FBox<RenderWindow>) -> DrawInput {
        self.egui_window
            .run(window, |_rw, ctx| {
                let win = egui::Window::new("Hello egui-sfml!");
                win.show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Message");
                        let te_re = ui.text_edit_singleline(&mut self.meta_window_data.input);
                        if ui.button("Send").clicked()
                            || ui.input(|inp| inp.key_pressed(egui::Key::Enter))
                        {
                            self.meta_window_data
                                .messages
                                .push(self.meta_window_data.input.clone());
                            te_re.request_focus();
                            self.meta_window_data.input.clear();
                        }
                    });
                    for msg in &self.meta_window_data.messages {
                        ui.separator();
                        ui.label(msg);
                    }
                });
            })
            .unwrap()
    }

    pub fn draw(&mut self, di: DrawInput, window: &mut FBox<RenderWindow>) {
        self.egui_window.draw(di, window, None);
    }
}
