use std::collections::HashMap;
use std::fmt::{Display, Write};

use egui_sfml::{DrawInput, SfEgui};
use sfml::SfResult;
use sfml::cpp::FBox;
use sfml::graphics::{
    Color, Font, RenderTarget, RenderWindow, Shape, Sprite, Text, Texture, Transformable,
};
use sfml::system::Vector2f;
use sfml::window::{Key, VideoMode};
use tracing::{debug, error};

use crate::counters::Counters;

#[derive(Default)]
pub enum InfoKind {
    Egui,
    #[default]
    Overlay,
    None,
}

impl InfoKind {
    fn next(&mut self) {
        *self = match self {
            Self::Overlay => Self::None,
            Self::None => Self::Egui,
            Self::Egui => Self::Overlay,
        };
    }
}

pub struct Info<'s> {
    kind: InfoKind,
    overlay: Text<'s>,
    custom_info: HashMap<String, String>,
    logo: Option<Sprite<'s>>,
    logo_text: Option<Text<'s>>,
}

impl<'s> Info<'s> {
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
            logo: None,
            logo_text: None,
        }
    }

    pub fn set_logo(&mut self, logo_texture: &'s Texture, logo_text: impl Display) -> SfResult<()> {
        let mut logo = Sprite::with_texture(logo_texture);
        let logo_rect = logo.texture_rect();
        let scale = 1.0 / ((logo_rect.width + logo_rect.height) as f32 / 100.0);
        debug!("logo rect: {logo_rect:?}");
        debug!("logo scale: {scale}");

        let mut logo_text = Text::new(
            &logo_text.to_string(),
            self.overlay
                .font()
                .expect("could not get font for logo_text"),
            12,
        );

        logo.set_scale(scale);
        logo.set_position((400.0, 400.0));
        logo_text.set_position((660.0, 400.0));
        logo.set_origin((logo_rect.width as f32, logo_rect.height as f32));
        logo_text.set_origin((logo_rect.width as f32, logo_rect.height as f32));

        self.logo = Some(logo);
        self.logo_text = Some(logo_text);
        Ok(())
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

    pub fn set_kind(&mut self, kind: InfoKind) {
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

    pub fn draw_with<const N: usize>(
        &mut self,
        window: &mut FBox<RenderWindow>,
        egui_window: &mut SfEgui,
        counters: &Counters<N>,
    ) {
        match self.kind {
            InfoKind::None => (),
            InfoKind::Egui => {
                let di = self.prepare_draw(window, egui_window, counters);
                egui_window.draw(di, window, None);
            }
            InfoKind::Overlay => {
                let _ = self.prepare_draw(window, egui_window, counters);
                window.draw(&self.overlay)
            }
        }
        if self.logo.is_some() && self.logo_text.is_some() {
            window.draw(self.logo.as_ref().unwrap());
            window.draw(self.logo_text.as_ref().unwrap());
        }
    }

    pub fn update_slow<const N: usize>(&mut self, counters: &Counters<N>) {
        self.overlay.set_string(&counters.text);
    }

    pub fn update<const N: usize>(&mut self, _counters: &Counters<N>) {}

    pub fn process_event(&mut self, event: &sfml::window::Event) {
        if let sfml::window::Event::KeyPressed { code: Key::F10, .. } = event {
            self.kind.next();
        }
    }
    pub fn z_level(&self) -> u16 {
        super::super::UI_LEVEL
    }
}
