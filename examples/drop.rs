use std::{any::Any, marker::PhantomData};

use rapier2d::{
    na::dimension,
    prelude::{ColliderBuilder, RigidBody, RigidBodyBuilder, RigidBodyType, SharedShape},
};
use sfml::{
    SfResult,
    graphics::{
        CircleShape, Color, CustomShape, CustomShapePoints, Font, RectangleShape, RenderTarget,
        RenderWindow, Shape, Transformable, glsl::Vec2,
    },
    system::Vector2f,
    window::{Event, Key, Style, VideoMode, mouse::Button},
};
use tracing::{info, instrument::WithSubscriber};

use bewegrs::{
    errors::BwgResult,
    graphic::{ComprehensiveElement, ComprehensiveUi},
    physics::{PhysicsElement, world::PhysicsWorld2D},
    setup,
    shapes::RectRoundShape,
};

const MAX_FPS: u64 = 60;
const BG: Color = Color::rgb(30, 20, 20);

const DEFAULT_X_SIZE: f32 = 300.0;
const DEFAULT_Y_SIZE: f32 = 200.0;
const R: f32 = 8.0;

struct PhysicsRect<'s, S>
where
    S: Shape<'s>,
{
    shape: S,
    lt: PhantomData<&'s S>,
    fixed: bool,
}

impl<'s, S: Shape<'s>> PhysicsRect<'s, S> {
    fn new(shape: S, stay_static: bool) -> Self {
        Self {
            shape,
            lt: PhantomData,
            fixed: stay_static,
        }
    }
}

impl<'s, S: Shape<'s>> ComprehensiveElement<'s> for PhysicsRect<'s, S> {
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

impl<'s, S: Shape<'s>> PhysicsElement<'s> for PhysicsRect<'s, S> {
    fn set_position(&mut self, position: Vector2f) {
        self.shape.set_position(position);
    }
    fn get_position(&self) -> Vector2f {
        self.shape.position()
    }

    fn rigid_body_type(&self) -> RigidBodyType {
        if self.fixed {
            RigidBodyType::Fixed
        } else {
            RigidBodyType::Dynamic
        }
    }

    fn get_collider_shape(&self) -> Vector2f {
        let bounds = self.shape.global_bounds();
        (bounds.width, bounds.height).into()
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

    let mut world = PhysicsWorld2D::build(20)?;

    let ground_size: Vector2f = (900.0, 20.0).into();
    let mut ground = RectangleShape::with_size(ground_size);
    ground.set_fill_color(Color::GREEN);
    ground.set_position((150.0, 750.0));
    // ground.set_origin(ground_size);
    let the_ground = PhysicsRect::new(ground, true);

    let mut rr = RectRoundShape::new(DEFAULT_X_SIZE, DEFAULT_Y_SIZE, R);
    rr.set_fill_color(Color::RED);
    rr.set_position((600.0, 100.0));
    // rr.set_origin((DEFAULT_X_SIZE / 2.0, DEFAULT_Y_SIZE / 2.0));
    let my_box = PhysicsRect::new(rr, false);

    world.add(Box::new(the_ground));
    world.add(Box::new(my_box));

    gui.set_world(world);

    'mainloop: loop {
        while let Some(event) = window.poll_event() {
            gui.add_event(&event);
            match event {
                Event::Closed
                | Event::KeyPressed {
                    code: Key::Escape, ..
                } => break 'mainloop,
                Event::MouseButtonPressed {
                    button: Button::Left,
                    x,
                    y,
                } => {
                    let mut clicky = CircleShape::new(8.0, 32);
                    clicky.set_fill_color(Color::GREEN);
                    clicky.set_origin((8.0, 8.0));
                    clicky.set_position((x as f32, y as f32));
                    let my_box = PhysicsRect::new(clicky, false);
                    gui.physics_world.as_mut().unwrap().add(Box::new(my_box));
                }
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
