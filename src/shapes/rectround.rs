use sfml::{graphics::CustomShapePoints, system::Vector2f};
use std::f32::consts::PI;

#[derive(Clone, Debug)]
pub struct RectRoundShape {
    width: f32,
    height: f32,
    radius: f32,
    corner_points: usize,
}

impl RectRoundShape {
    pub fn new(width: f32, height: f32, radius: f32) -> Self {
        // Ensure radius isn't too large
        let max_radius = width.min(height) / 2.0;
        let radius = radius.min(max_radius);

        RectRoundShape {
            width,
            height,
            radius,
            corner_points: 8, // Default corner resolution
        }
    }

    pub fn with_corner_points(mut self, points: usize) -> Self {
        self.corner_points = points.max(4);
        self
    }
}

impl CustomShapePoints for RectRoundShape {
    fn point_count(&self) -> usize {
        self.corner_points * 4
    }

    fn point(&self, index: usize) -> Vector2f {
        let points_per_corner = self.corner_points;
        let total_points = self.point_count();

        if index >= total_points {
            panic!("Point index out of bounds: {}", index);
        }

        // Determine which corner this point belongs to
        let corner = index / points_per_corner;
        let point_in_corner = index % points_per_corner;

        // Calculate the angle for this point
        let angle_step = PI / 2.0 / (points_per_corner as f32);
        let corner_angle = point_in_corner as f32 * angle_step;

        // Calculate the position based on which corner we're in
        match corner {
            0 => {
                // Top-left corner
                let angle = PI + corner_angle;
                let x = self.radius + self.radius * angle.cos();
                let y = self.radius + self.radius * angle.sin();
                Vector2f { x, y }
            }
            1 => {
                // Top-right corner
                let angle = PI * 1.5 + corner_angle;
                let x = self.width - self.radius + self.radius * angle.cos();
                let y = self.radius + self.radius * angle.sin();
                Vector2f { x, y }
            }
            2 => {
                // Bottom-right corner
                let angle = corner_angle;
                let x = self.width - self.radius + self.radius * angle.cos();
                let y = self.height - self.radius + self.radius * angle.sin();
                Vector2f { x, y }
            }
            3 => {
                // Bottom-left corner
                let angle = PI * 0.5 + corner_angle;
                let x = self.radius + self.radius * angle.cos();
                let y = self.height - self.radius + self.radius * angle.sin();
                Vector2f { x, y }
            }
            _ => panic!("Invalid corner index"),
        }
    }
}
