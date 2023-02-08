use ::bevy::utils::HashMap;

use super::*;

/// Resource that maps [`ColliderShape`]s to a rapier [`SharedShape`][rapier::SharedShape].
#[derive(Clone, Deref, DerefMut, TypeUlid, Default)]
#[ulid = "01GRVP9XNY2AA8D9FTSC51HE6E"]
pub struct ColliderShapeCache(HashMap<ColliderShape, rapier::SharedShape>);

impl ColliderShapeCache {
    pub fn shared_shape(&mut self, shape: ColliderShape) -> &mut rapier::SharedShape {
        self.entry(shape).or_insert_with(|| shape.shared_shape())
    }
}

/// The Jumpy collision shape type.
#[derive(Clone, Copy, Debug)]
pub enum ColliderShape {
    Circle { diameter: f32 },
    Rectangle { size: Vec2 },
}

impl ColliderShape {
    pub fn compute_aabb(&self, transform: Transform) -> rapier::Aabb {
        match self {
            ColliderShape::Circle { diameter } => rapier::Ball {
                radius: *diameter / 2.0,
            }
            .aabb(&rapier::Isometry::new(
                transform.translation.truncate().to_array().into(),
                transform.rotation.to_euler(EulerRot::XYZ).2,
            )),
            ColliderShape::Rectangle { size } => rapier::Cuboid {
                half_extents: (*size / 2.0).to_array().into(),
            }
            .aabb(&rapier::Isometry::new(
                transform.translation.truncate().to_array().into(),
                transform.rotation.to_euler(EulerRot::XYZ).2,
            )),
        }
    }

    pub fn shared_shape(&self) -> rapier::SharedShape {
        match self {
            ColliderShape::Circle { diameter } => rapier::SharedShape::ball(*diameter / 2.0),
            ColliderShape::Rectangle { size } => {
                rapier::SharedShape::cuboid(size.x / 2.0, size.y / 2.0)
            }
        }
    }
}

impl Default for ColliderShape {
    fn default() -> Self {
        Self::Rectangle {
            size: vec2(10.0, 10.0),
        }
    }
}

impl PartialEq for ColliderShape {
    fn eq(&self, other: &Self) -> bool {
        use ordered_float::OrderedFloat as F;
        match (self, other) {
            (
                Self::Circle {
                    diameter: l_diameter,
                },
                Self::Circle {
                    diameter: r_diameter,
                },
            ) => F(*l_diameter) == F(*r_diameter),
            (Self::Rectangle { size: l_size }, Self::Rectangle { size: r_size }) => {
                F(l_size.x) == F(r_size.x) && F(l_size.y) == F(r_size.y)
            }
            _ => false,
        }
    }
}
impl Eq for ColliderShape {}

impl std::hash::Hash for ColliderShape {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use ordered_float::OrderedFloat as F;
        core::mem::discriminant(self).hash(state);

        match self {
            ColliderShape::Circle { diameter } => F(*diameter).hash(state),
            ColliderShape::Rectangle { size } => {
                F(size.x).hash(state);
                F(size.y).hash(state);
            }
        }
    }
}

impl PartialOrd for ColliderShape {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use ordered_float::OrderedFloat as F;
        use std::cmp::Ordering::*;

        Some(match self {
            ColliderShape::Circle { diameter: r1 } => match other {
                ColliderShape::Circle { diameter: r2 } => F(*r1).cmp(&F(*r2)),
                ColliderShape::Rectangle { .. } => Less,
            },
            ColliderShape::Rectangle { size: s1 } => match other {
                ColliderShape::Rectangle { size: s2 } => {
                    let xdiff = F(s1.x).cmp(&F(s2.x));
                    if xdiff == Equal {
                        F(s1.y).cmp(&F(s2.y))
                    } else {
                        xdiff
                    }
                }
                ColliderShape::Circle { .. } => Greater,
            },
        })
    }
}
impl Ord for ColliderShape {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
