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
    fn rigid_body_type(&self) -> RigidBodyType;
    fn set_position(&mut self, position: Vector2f);
    fn get_position(&self) -> Vector2f;
    fn get_collider_shape(&self) -> Vector2f;
}
