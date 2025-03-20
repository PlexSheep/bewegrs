use rapier2d::prelude::*;
use sfml::system::Vector2f;

use crate::graphic::ComprehensiveElement;

pub mod world;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, Default)]
pub struct PElementID {
    inner: u128,
}

impl rand::distr::Distribution<PElementID> for rand::distr::StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> PElementID {
        PElementID {
            inner: rng.random(),
        }
    }
}

impl PElementID {
    pub fn new() -> Self {
        rand::random()
    }
}
pub trait PhysicsElement<'s>: ComprehensiveElement<'s> {
    fn init_rigid_body(&self) -> RigidBody;
    fn init_collider(&self) -> Collider;
    fn set_position(&mut self, position: Vector2f);
    fn get_position(&self) -> Vector2f;
}
