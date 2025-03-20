use rapier2d::prelude::{ColliderBuilder, RigidBody, RigidBodyBuilder};
use sfml::{
    SfResult,
    graphics::{
        CircleShape, Color, CustomShape, CustomShapePoints, Font, RectangleShape, RenderTarget,
        RenderWindow, Shape, Transformable, glsl::Vec2,
    },
    system::Vector2f,
    window::{Event, Key, Style, VideoMode},
};
use tracing::info;

use bewegrs::{
    errors::BwgResult,
    graphic::{ComprehensiveElement, ComprehensiveUi},
    physics::{PhysicsElement, world::PhysicsWorld2D},
    setup,
    shapes::RectRoundShape,
};

const MAX_FPS: u64 = 60;
const BG: Color = Color::rgb(30, 20, 20);

struct Thing<'s> {
    shape: CustomShape<'s>,
}

impl Thing<'_> {
    const X: f32 = 300.0;
    const Y: f32 = 200.0;
    const R: f32 = 8.0;

    fn new() -> Self {
        let mut shape = RectRoundShape::new(Self::X, Self::Y, Self::R);
        shape.set_fill_color(Color::RED);
        shape.set_origin((Self::X / 2.0, Self::Y / 2.0));
        shape.set_position((600.0, 200.0));
        Self { shape }
    }
}

impl<'s> ComprehensiveElement<'s> for Thing<'s> {
    fn draw_with(
        &mut self,
        sfml_w: &mut sfml::cpp::FBox<RenderWindow>,
        _egui_w: &mut egui_sfml::SfEgui,
        _counters: &bewegrs::counter::Counter,
        _info: &mut bewegrs::graphic::elements::info::Info<'s>,
    ) {
        sfml_w.draw(&self.shape);
    }
}

impl<'s> PhysicsElement<'s> for Thing<'s> {
    fn init_rigid_body(&self) -> rapier2d::prelude::RigidBody {
        RigidBodyBuilder::dynamic().build()
    }

    fn init_collider(&self) -> rapier2d::prelude::Collider {
        ColliderBuilder::cuboid(Self::X / 2.0, Self::Y / 2.0).build()
    }
    fn set_position(&mut self, position: Vector2f) {
        self.shape.set_position(position);
    }
    fn get_position(&self) -> Vector2f {
        self.shape.position()
    }
}

struct Floor<'s> {
    shape: RectangleShape<'s>,
}

impl Floor<'_> {
    const X: f32 = 900.0;
    const Y: f32 = 20.0;

    fn new() -> Self {
        let mut shape = RectangleShape::with_size((Self::X, Self::Y).into());
        shape.set_fill_color(Color::GREEN);
        shape.set_origin((Self::X / 2.0, Self::Y / 2.0));
        shape.set_position((600.0, 600.0));
        Self { shape }
    }
}

impl<'s> ComprehensiveElement<'s> for Floor<'s> {
    fn draw_with(
        &mut self,
        sfml_w: &mut sfml::cpp::FBox<RenderWindow>,
        _egui_w: &mut egui_sfml::SfEgui,
        _counters: &bewegrs::counter::Counter,
        _info: &mut bewegrs::graphic::elements::info::Info<'s>,
    ) {
        sfml_w.draw(&self.shape);
    }
}

impl<'s> PhysicsElement<'s> for Floor<'s> {
    fn init_rigid_body(&self) -> rapier2d::prelude::RigidBody {
        RigidBodyBuilder::fixed().build()
    }

    fn init_collider(&self) -> rapier2d::prelude::Collider {
        ColliderBuilder::cuboid(Self::X / 2.0, Self::Y / 2.0).build()
    }
    fn set_position(&mut self, position: Vector2f) {
        self.shape.set_position(position);
    }
    fn get_position(&self) -> Vector2f {
        self.shape.position()
    }
}

fn main() -> BwgResult<()> {
    setup(true);

    let video = VideoMode::new(1200, 800, 32);
    info!("video mode: {video:?}");
    let mut window = RenderWindow::new(video, "Drop it!", Style::DEFAULT, &Default::default())?;

    let mut font = Font::new()?;
    font.load_from_memory_static(include_bytes!("../resources/sansation.ttf"))?;

    let mut gui = ComprehensiveUi::build(&mut window, &font, &video, MAX_FPS)?;

    let mut world = PhysicsWorld2D::build()?;

    let the_ground = Floor::new();
    let my_box = Thing::new();

    world.add(Box::new(the_ground));
    world.add(Box::new(my_box));

    gui.add(Box::new(world));

    'mainloop: loop {
        while let Some(event) = window.poll_event() {
            gui.add_event(&event);
            match event {
                Event::Closed
                | Event::KeyPressed {
                    code: Key::Escape, ..
                } => break 'mainloop,
                _ => (),
            }
        }

        gui.frame_start();
        window.clear(BG);

        gui.update();
        if gui.counter.frames % MAX_FPS == 1 {
            gui.update_slow()
        }

        gui.draw_with(&mut window);

        gui.display(&mut window);
    }
    Ok(())
}
