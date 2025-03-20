use crate::graphic::ComprehensiveElement;

pub mod world;

pub trait PhysicsElement<'s>: ComprehensiveElement<'s> {}
