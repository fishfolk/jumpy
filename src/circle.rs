use macroquad::math::{vec2, Rect, Vec2};

pub struct Circle {
    x: f32,
    y: f32,
    r: f32,
}

impl Circle {
    pub fn new(x: f32, y: f32, r: f32) -> Self {
        Circle { x, y, r }
    }

    #[allow(dead_code)]
    pub fn contains(&self, pos: Vec2) -> bool {
        return pos.distance(vec2(self.x, self.y)) >= self.r;
    }

    pub fn overlaps(&self, rect: Rect) -> bool {
        let dist_x = (self.x - rect.x).abs();
        let dist_y = (self.y - rect.y).abs();
        if dist_x > rect.w / 2.0 + self.r || dist_y > rect.h / 2.0 + self.r {
            return false;
        }
        if dist_x <= rect.w / 2.0 || dist_y <= rect.h / 2.0 {
            return true;
        }
        let lhs = dist_x - rect.w / 2.0;
        let rhs = dist_y - rect.h / 2.0;
        let dist_sq = (lhs * lhs) + (rhs * rhs);
        return dist_sq <= self.r * self.r;
    }
}
