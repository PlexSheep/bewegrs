use sfml::graphics::{CustomShape, Shape};
use sfml::{graphics::CustomShapePoints, system::Vector2f};
use std::f32::consts::PI;

#[derive(Clone, Debug)]
pub struct RectRoundShape {
    width: f32,
    height: f32,
    radius: f32,
    points_per_corner: usize,
}

impl RectRoundShape {
    pub fn new<'s>(width: f32, height: f32, radius: f32) -> CustomShape<'s> {
        // Ensure radius isn't too large
        let max_radius = width.min(height) / 2.0;
        let radius = radius.min(max_radius);

        let inner = RectRoundShape {
            width,
            height,
            radius,
            points_per_corner: 8, // Default corner resolution
        };
        let mut shape = CustomShape::new(Box::new(inner));
        shape.set_outline_thickness(3.0);
        shape
    }

    pub fn basic_shape(width: f32, height: f32, radius: f32) -> Self {
        // Ensure radius isn't too large
        let max_radius = width.min(height) / 2.0;
        let radius = radius.min(max_radius);

        RectRoundShape {
            width,
            height,
            radius,
            points_per_corner: 8, // Default corner resolution
        }
    }

    pub fn with_corner_points(mut self, points: usize) -> Self {
        self.points_per_corner = points.max(4);
        self
    }
}

impl CustomShapePoints for RectRoundShape {
    fn point_count(&self) -> usize {
        // 4 corners with points_per_corner points each
        self.points_per_corner * 4
    }

    fn point(&self, index: usize) -> Vector2f {
        let total_points = self.point_count();

        if index >= total_points {
            panic!("Point index out of bounds: {}", index);
        }

        // Each corner gets points_per_corner points
        let quarter = index / self.points_per_corner;
        let i = index % self.points_per_corner;

        // Calculate the angle within the quarter circle (0 to π/2)
        let angle_per_point = (PI / 2.0) / (self.points_per_corner as f32 - 1.0);
        let corner_angle = i as f32 * angle_per_point;

        // Base angle for each quarter (where each quarter circle starts)
        let base_angles = [PI, 3.0 * PI / 2.0, 0.0, PI / 2.0];
        let base_angle = base_angles[quarter];

        // Calculate final angle
        let angle = base_angle + corner_angle;

        // Centers of each quarter circle
        let centers = [
            (self.radius, self.radius),                            // Top-left
            (self.width - self.radius, self.radius),               // Top-right
            (self.width - self.radius, self.height - self.radius), // Bottom-right
            (self.radius, self.height - self.radius),              // Bottom-left
        ];

        let (center_x, center_y) = centers[quarter];

        // Calculate point on the circle
        let x = center_x + self.radius * angle.cos();
        let y = center_y + self.radius * angle.sin();

        Vector2f { x, y }
    }
}
