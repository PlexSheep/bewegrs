use std::collections::HashMap;

use rapier2d::prelude::*;
use sfml::graphics::{RectangleShape, RenderTarget, Shape, Transformable};
use sfml::system::Vector2f;
use sfml::window::Event;

use crate::counter::Counter;
use crate::errors::BwgResult;
use crate::graphic::elements::info::Info;

use super::{PElementID, PhysicsElement};

pub const DEFAULT_GRAVITY: Vector<f32> = vector![0.0, 9.81];

pub struct PhysicsWorld2D<'s> {
    pub gravity: Vector<f32>,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub query_pipeline: QueryPipeline,
    pub physics_hooks: (),
    pub event_handler: (),

    elements: HashMap<PElementID, (ColliderHandle, Box<dyn PhysicsElement<'s>>)>,
    scale: f32,
}

impl<'s> PhysicsWorld2D<'s> {
    pub fn build(scale: u64) -> BwgResult<Self> {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        /* Create the ground. */
        let collider = ColliderBuilder::cuboid(100.0, 0.1).build();
        collider_set.insert(collider);

        /* Create the bouncing ball. */
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![0.0, 10.0])
            .build();
        let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
        let ball_body_handle = rigid_body_set.insert(rigid_body);
        collider_set.insert_with_parent(collider, ball_body_handle, &mut rigid_body_set);

        /* Create other structures necessary for the simulation. */
        let gravity = DEFAULT_GRAVITY;
        let integration_parameters = IntegrationParameters::default();
        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = DefaultBroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();
        let query_pipeline = QueryPipeline::new();
        let physics_hooks = ();
        let event_handler = ();

        let rigid_body_set = RigidBodySet::new();
        let collider_set = ColliderSet::new();

        let elements = HashMap::new();

        Ok(Self {
            gravity,
            integration_parameters,
            physics_pipeline,
            island_manager,
            broad_phase,
            narrow_phase,
            impulse_joint_set,
            multibody_joint_set,
            ccd_solver,
            query_pipeline,
            physics_hooks,
            event_handler,
            rigid_body_set,
            collider_set,
            elements,
            scale: scale as f32,
        })
    }

    pub fn add(&mut self, element: Box<dyn PhysicsElement<'s>>) -> PElementID {
        let id = self.get_new_element_id();

        let rbody_h = self
            .rigid_body_set
            .insert(RigidBodyBuilder::new(element.rigid_body_type()));

        let size = translate_from_position(&element.get_collider_shape(), self.scale);
        let mut coll = ColliderBuilder::cuboid(size.translation.x, size.translation.y).build();
        coll.set_position(translate_from_position(&element.get_position(), self.scale));
        let coll_h = self
            .collider_set
            .insert_with_parent(coll, rbody_h, &mut self.rigid_body_set);

        self.elements.insert(id, (coll_h, element));
        id
    }

    pub fn get(&self, id: &PElementID) -> Option<&dyn PhysicsElement<'s>> {
        self.elements.get(id).map(|v| v.1.as_ref())
    }

    fn get_collider_handle(&self, id: &PElementID) -> Option<ColliderHandle> {
        self.elements.get(id).map(|v| v.0)
    }

    pub fn get_mut(&mut self, id: &PElementID) -> Option<&mut dyn PhysicsElement<'s>> {
        self.elements.get_mut(id).map(|v| v.1.as_mut())
    }

    pub fn remove(&mut self, id: &PElementID) -> Option<Box<dyn PhysicsElement<'s>>> {
        let (id, bo) = self.elements.remove(id)?;
        self.collider_set
            .remove(id, &mut self.island_manager, &mut self.rigid_body_set, true);
        Some(bo)
    }

    fn get_position(&self, id: &PElementID) -> Option<Vector2f> {
        let col_h = self.get_collider_handle(id)?;
        let elem = &self.collider_set[col_h];
        Some(translate_to_position(elem.position(), self.scale))
    }

    pub fn get_new_element_id(&self) -> PElementID {
        let mut id: PElementID;
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

    pub fn update(&mut self, _counters: &Counter, _info: &mut Info<'s>) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &self.physics_hooks,
            &self.event_handler,
        );

        for (col_h, element) in self.elements.values_mut() {
            let pos = {
                let elem: &Collider = &self.collider_set[*col_h];
                Some(translate_to_position(elem.position(), self.scale))
            }
            .unwrap();

            element.set_position(pos);
        }
    }

    pub fn draw_with(
        &mut self,
        sfml_w: &mut sfml::cpp::FBox<sfml::graphics::RenderWindow>,
        egui_w: &mut egui_sfml::SfEgui,
        counters: &Counter,
        info: &mut Info<'s>,
    ) {
        // debug borders around actual graphic elemnts
        for (id, (_colh, element)) in self.elements.iter() {
            let position = self.get_position(id).unwrap();
            let size = element.get_collider_shape();
            let mut dbgbox = RectangleShape::new();
            dbgbox.set_origin((10.0, 10.0)); // TODO: set center correctly
            dbgbox.set_size(size * 1.05);
            dbgbox.set_outline_color(sfml::graphics::Color::MAGENTA);
            dbgbox.set_outline_thickness(0.7);
            dbgbox.set_position(position);
            dbgbox.set_fill_color(sfml::graphics::Color::TRANSPARENT);
            sfml_w.draw(&dbgbox);
        }
        // debug borders around collision
        for (id, (colh, _element)) in self.elements.iter() {
            let coll = &self.collider_set[*colh];
            let position = translate_to_position(coll.position(), self.scale);
            let size: Vector2f = (20.0, 20.0).into(); // TODO: get size correctly
            let mut dbgbox = RectangleShape::new();
            dbgbox.set_origin((10.0, 10.0)); // TODO: set center correctly
            dbgbox.set_size(size * 1.05);
            dbgbox.set_outline_color(sfml::graphics::Color::CYAN);
            dbgbox.set_outline_thickness(0.7);
            dbgbox.set_position(position);
            dbgbox.set_fill_color(sfml::graphics::Color::TRANSPARENT);
            sfml_w.draw(&dbgbox);
        }
        for (_colh, element) in self.elements.values_mut() {
            element.draw_with(sfml_w, egui_w, counters, info);
        }
    }

    pub fn process_event(&self, _event: &Event, _counter: &Counter, _info: &mut Info<'s>) {}

    pub fn update_slow(&mut self, _counters: &Counter, _info: &mut Info<'s>) {}
}

fn translate_from_position(point: &Vector2f, scale: f32) -> Isometry<Real> {
    Isometry::new(vector![point.x / scale, point.y / scale], 0.0)
}

fn translate_to_position(point: &Isometry<Real>, scale: f32) -> Vector2f {
    Vector2f::from((point.translation.x * scale, point.translation.y * scale))
}
