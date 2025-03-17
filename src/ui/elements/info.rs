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

use crate::counter::Counter;

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
    video: &'s VideoMode,
}

impl<'s> Info<'s> {
    pub const DEFAULT_NAME: &'static str = "Info";

    pub fn new(font: &'s FBox<Font>, video: &'s VideoMode, counters: &Counter) -> Self {
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
            video,
        }
    }

    pub fn set_logo(&mut self, logo_texture: &'s Texture, logo_text: impl Display) -> SfResult<()> {
        let mut logo = Sprite::with_texture(logo_texture);
        let logo_rect = logo.texture_rect();
        let scale = 1.0 / ((logo_rect.width + logo_rect.height) as f32 / 100.0);
        debug!("logo_rect: {logo_rect:?}");

        const LOGO_TEXT_SIZE: u32 = 13;

        let mut logo_text = Text::new(
            &logo_text.to_string(),
            self.overlay
                .font()
                .expect("could not get font for logo_text"),
            LOGO_TEXT_SIZE,
        );

        logo.set_scale(scale);
        logo.set_position((
            logo_rect.width as f32 * scale + 10.0,
            self.video.height as f32 - (logo_rect.height as f32 * scale),
        ));
        logo_text.set_position((
            1.3 * logo_rect.width as f32 * scale + 10.0,
            self.video.height as f32
                - (logo_rect.height as f32 * scale)
                - LOGO_TEXT_SIZE as f32 * 2.5,
        ));
        logo.set_origin((logo_rect.width as f32, logo_rect.height as f32));

        self.logo = Some(logo);
        self.logo_text = Some(logo_text);
        Ok(())
    }

    pub fn set_custom_info(&mut self, key: impl Display, value: impl Display) {
        self.custom_info.insert(key.to_string(), value.to_string());
    }

    pub fn prepare_draw(
        &mut self,
        window: &mut FBox<RenderWindow>,
        egui_window: &mut SfEgui,
        counters: &Counter,
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

    fn get_text(&self, counters: &Counter) -> String {
        let mut buf: String = format!("{}\n", counters.text);
        for (key, value) in &self.custom_info {
            if let Err(e) = writeln!(buf, "{key}: {value}") {
                error!("could not write to format buffer for info widget: {e}");
            }
        }
        buf
    }

    pub fn draw_with(
        &mut self,
        window: &mut FBox<RenderWindow>,
        egui_window: &mut SfEgui,
        counters: &Counter,
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

    pub fn update_slow(&mut self, counters: &Counter) {
        self.overlay.set_string(&counters.text);
    }

    pub fn update(&mut self, _counters: &Counter) {}

    pub fn process_event(&mut self, event: &sfml::window::Event) {
        if let sfml::window::Event::KeyPressed { code: Key::F10, .. } = event {
            self.kind.next();
        }
    }
    pub fn z_level(&self) -> u16 {
        super::super::UI_LEVEL
    }

    pub fn video(&self) -> &VideoMode {
        self.video
    }
}
