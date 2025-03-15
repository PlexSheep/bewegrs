use sfml::{graphics::CustomShapePoints, system::Vector2f};

#[derive(Clone, Copy)]
pub struct RectRoundShape;

impl CustomShapePoints for RectRoundShape {
    fn point_count(&self) -> usize {
        4
    }

    fn point(&self, point: usize) -> Vector2f {
        match point {
            0 => Vector2f { x: 20., y: 20. },
            1 => Vector2f { x: 20., y: 980. },
            2 => Vector2f { x: 980., y: 980. },
            3 => Vector2f { x: 980., y: 20. },

            p => panic!("Non-existent point: {p}"),
        }
    }
}
