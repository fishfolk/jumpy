use bevy::{
    ecs::system::SystemParam,
    math::vec2,
    prelude::*,
    utils::{FloatOrd, HashMap},
};
use bevy_prototype_lyon::{entity::ShapeBundle, prelude::*};

use super::{collisions::Collider, PhysicsStages};

/// Physics debug rendering plugin
pub struct PhysicsDebugRenderPlugin;

/// Resource used to configure debug rendering
#[derive(Default)]
pub struct PhysicsDebugRenderConfig {
    pub enabled: bool,
}

/// Component that may be added to entities with colliders to change the color of it's collision debug render
#[derive(Component, Deref, DerefMut)]
pub struct ColliderDebugColor(pub Color);

/// The stage in which the physics debug shapes are generated
#[derive(StageLabel)]
struct PhysicsDebugRenderStage;

impl Plugin for PhysicsDebugRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsDebugRenderConfig>()
            .add_stage_after(
                PhysicsStages::UpdatePhysics,
                PhysicsDebugRenderStage,
                SystemStage::single(render_collision_shapes),
            );
    }
}

const DEFAULT_COLOR: Color = Color::ORANGE;

/// System that renders the debug shapes
fn render_collision_shapes(mut renderer: DebugRenderer, config: Res<PhysicsDebugRenderConfig>) {
    if config.enabled {
        renderer.render();
    }
}

#[derive(Component)]
struct RapierDebugRenderShapes;

#[derive(Deref, DerefMut)]
pub struct HashEqColor(Color);

impl std::hash::Hash for HashEqColor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let [r, g, b, a] = self.0.as_rgba_f32();
        FloatOrd(r).hash(state);
        FloatOrd(g).hash(state);
        FloatOrd(b).hash(state);
        FloatOrd(a).hash(state);
    }
}

impl std::cmp::PartialEq for HashEqColor {
    fn eq(&self, other: &Self) -> bool {
        let [r1, g1, b1, a1] = self.0.as_rgba_f32();
        let [r2, g2, b2, a2] = other.0.as_rgba_f32();
        FloatOrd(r1) == FloatOrd(r2)
            && FloatOrd(g1) == FloatOrd(g2)
            && FloatOrd(b1) == FloatOrd(b2)
            && FloatOrd(a1) == FloatOrd(a2)
    }
}
impl std::cmp::Eq for HashEqColor {}

/// Rapier debug rendering backend that uses Egui to draw the lines
#[derive(SystemParam)]
struct DebugRenderer<'w, 's> {
    commands: Commands<'w, 's>,
    shape_paths: Local<'s, HashMap<HashEqColor, ShapePath>>,
    paths_query:
        Query<'w, 's, (&'static mut Path, &'static mut DrawMode), With<RapierDebugRenderShapes>>,
    custom_colors: Query<'w, 's, &'static ColliderDebugColor>,
    colliders: Query<'w, 's, (Entity, &'static Collider)>,
}

impl<'w, 's> DebugRenderer<'w, 's> {
    /// Render the shapes
    fn render(&mut self) {
        self.draw_colliders();

        self.finish();
    }

    /// Draw collider shapes
    fn draw_colliders(&mut self) {
        for (entity, collider) in &self.colliders {
            let color = self.color(entity);

            let shape_path_ref = self.shape_paths.entry(HashEqColor(color)).or_default();
            let shape_path = std::mem::take(shape_path_ref);

            *shape_path_ref = shape_path.add(&ColliderRect {
                pos: collider.pos,
                size: vec2(collider.width, collider.height),
            });
        }
    }

    /// Called to finish generating shapes
    fn finish(&mut self) {
        let mut query_iter = self.paths_query.iter_mut();

        for (color, shape_path) in self.shape_paths.drain() {
            let path = shape_path.build();
            let mode = DrawMode::Stroke(StrokeMode::new(*color, 0.75));

            if let Some((mut old_path, mut old_mode)) = query_iter.next() {
                *old_path = path;
                *old_mode = mode;
            } else {
                self.commands
                    .spawn_bundle(ShapeBundle {
                        path,
                        mode,
                        // Set the rendering a tiny bit forward to avoid z-fighting with the editor
                        // overlays.
                        transform: Transform::from_xyz(0.0, 0.0, -0.1),
                        ..default()
                    })
                    .insert(Name::new("Rapier Debug Render Shapes"))
                    .insert(RapierDebugRenderShapes);
            }
        }

        // Clear out any shape entities we haven't used up
        for (mut path, _) in query_iter {
            *path = ShapePath::default().build();
        }
    }

    /// Helper to grab the objects custom collider color if it exists
    fn color(&self, entity: Entity) -> Color {
        self.custom_colors
            .get(entity)
            .map(|co| co.0)
            .ok()
            .unwrap_or(DEFAULT_COLOR)
    }
}

/// Helper type that implements [`Geometry`] for rendering rectangle shapes at a specific location.
struct ColliderRect {
    pos: Vec2,
    size: Vec2,
}

mod geom {
    use bevy_prototype_lyon::prelude::tess::{
        geom::euclid::{Point2D, Size2D},
        math::Rect,
        path::traits::PathBuilder,
    };

    use super::*;

    impl Geometry for ColliderRect {
        fn add_geometry(&self, b: &mut tess::path::path::Builder) {
            b.add_rectangle(
                &Rect {
                    origin: Point2D::new(
                        self.pos.x - self.size.x / 2.0,
                        self.pos.y - self.size.y / 2.0,
                    ),
                    size: Size2D::new(self.size.x, self.size.y),
                },
                tess::path::Winding::Positive,
            )
        }
    }
}
