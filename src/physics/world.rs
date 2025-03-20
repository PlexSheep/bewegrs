use std::collections::HashMap;

use rapier2d::prelude::*;
use sfml::system::Vector2f;

use crate::counter::Counter;
use crate::errors::BwgResult;
use crate::graphic::ComprehensiveElement;
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
}

impl<'s> PhysicsWorld2D<'s> {
    pub fn build() -> BwgResult<Self> {
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
        })
    }

    pub fn add(&mut self, element: Box<dyn PhysicsElement<'s>>) -> PElementID {
        let id = self.get_new_element_id();

        let rbody_h = self.rigid_body_set.insert(element.init_rigid_body());

        let mut coll = element.init_collider();
        let pos = element.get_position();
        coll.set_position(Isometry::new(vector![pos.x, pos.y], 0.0));
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
        let pos = elem.position();
        Some(Vector2f::from((pos.translation.x, pos.translation.y)))
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
}

impl<'s> ComprehensiveElement<'s> for PhysicsWorld2D<'s> {
    fn update(&mut self, _counters: &Counter, _info: &mut Info<'s>) {
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
                let pos = elem.position();
                Some(Vector2f::from((pos.translation.x, pos.translation.y)))
            }
            .unwrap();

            element.set_position(pos);
        }
    }
    fn draw_with(
        &mut self,
        sfml_w: &mut sfml::cpp::FBox<sfml::graphics::RenderWindow>,
        egui_w: &mut egui_sfml::SfEgui,
        counters: &Counter,
        info: &mut Info<'s>,
    ) {
        for (_colh, element) in self.elements.values_mut() {
            element.draw_with(sfml_w, egui_w, counters, info);
        }
    }
}
