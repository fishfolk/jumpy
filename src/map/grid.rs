use bevy_prototype_lyon::prelude::tess::{
    geom::{
        euclid::{Point2D, Size2D},
        Rect,
    },
    path::traits::PathBuilder,
};

use super::*;

pub struct Grid {
    pub grid_size: UVec2,
    pub tile_size: UVec2,
}

impl Geometry for Grid {
    fn add_geometry(&self, b: &mut tess::path::path::Builder) {
        for x in 0..self.grid_size.x {
            for y in 0..self.grid_size.y {
                b.add_rectangle(
                    &Rect {
                        origin: Point2D::new(
                            x as f32 * self.tile_size.x as f32,
                            y as f32 * self.tile_size.y as f32,
                        ),
                        size: Size2D::new(self.tile_size.x as f32, self.tile_size.y as f32),
                    },
                    tess::path::Winding::Positive,
                );
            }
        }
    }
}
