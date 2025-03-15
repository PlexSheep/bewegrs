use sfml::{
    graphics::{Color, CustomShapePoints},
    system::Vector2f,
};

#[derive(Clone, Copy)]
pub struct TriangleShape;
#[derive(Clone, Copy)]
pub struct RoundedRectShape;

impl CustomShapePoints for TriangleShape {
    fn point_count(&self) -> usize {
        3
    }

    fn point(&self, point: usize) -> Vector2f {
        match point {
            0 => Vector2f { x: 20., y: 580. },
            1 => Vector2f { x: 400., y: 20. },
            2 => Vector2f { x: 780., y: 580. },
            p => panic!("Non-existent point: {p}"),
        }
    }
}

impl CustomShapePoints for RoundedRectShape {
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

pub fn hue_time(t: f32) -> Color {
    const fn lerp(from: f32, to: f32, amount: f32) -> f32 {
        from + amount * (to - from)
    }

    let frac = t.fract();

    let [r, g, b] = match (t % 6.0).floor() {
        0.0 => [255., lerp(0., 255., frac), 0.],
        1.0 => [lerp(255., 0., frac), 255., 0.],
        2.0 => [0., 255., lerp(0., 255., frac)],
        3.0 => [0., lerp(255., 0., frac), 255.],
        4.0 => [lerp(0., 255., frac), 0., 255.],
        _ => [255., 0., lerp(255., 0., frac)],
    };
    Color::rgb(r as u8, g as u8, b as u8)
}
