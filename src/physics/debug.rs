use bevy::{
    ecs::system::SystemParam,
    prelude::*,
    utils::{FloatOrd, HashMap},
};
use bevy_prototype_lyon::{entity::ShapeBundle, prelude::*, shapes::Line};
use bevy_rapier2d::{
    prelude::*,
    rapier::{
        math::Point,
        pipeline::DebugRenderObject,
        prelude::{DebugRenderBackend, DebugRenderPipeline},
    },
};

pub struct DebugRenderPlugin;

#[derive(StageLabel)]
struct PhysicsDebugRenderStage;

impl Plugin for DebugRenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugRenderContext {
            enabled: false,
            pipeline: {
                let mut pipeline = DebugRenderPipeline::default();
                pipeline.style = DebugRenderStyle {
                    rigid_body_axes_length: 0.0,
                    ..default()
                };
                pipeline
            },
            ..default()
        })
        .add_stage_after(
            PhysicsStages::Writeback,
            PhysicsDebugRenderStage,
            SystemStage::single(render_collision_shapes),
        );
    }
}

fn render_collision_shapes(
    ctx: Res<RapierContext>,
    mut renderer: RapierDebugRenderer,
    mut rapier_debug: ResMut<DebugRenderContext>,
) {
    if rapier_debug.enabled {
        rapier_debug.pipeline.render(
            &mut renderer,
            &ctx.bodies,
            &ctx.colliders,
            &ctx.impulse_joints,
            &ctx.multibody_joints,
            &ctx.narrow_phase,
        );
    }

    renderer.finish();
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
struct RapierDebugRenderer<'w, 's> {
    commands: Commands<'w, 's>,
    shape_paths: Local<'s, HashMap<HashEqColor, ShapePath>>,
    paths_query:
        Query<'w, 's, (&'static mut Path, &'static mut DrawMode), With<RapierDebugRenderShapes>>,
    custom_colors: Query<'w, 's, &'static ColliderDebugColor>,
    context: Res<'w, RapierContext>,
}

impl<'w, 's> RapierDebugRenderer<'w, 's> {
    /// Call after passing to rapier to draw final shapes
    fn finish(&mut self) {
        let mut query_iter = self.paths_query.iter_mut();

        for (color, shape_path) in self.shape_paths.drain() {
            let path = shape_path.build();
            let mode = DrawMode::Stroke(StrokeMode::new(*color, 1.0));

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
                    .insert(RapierDebugRenderShapes);
            }
        }

        // Clear out any shape entities we haven't used up
        for (mut path, _) in query_iter {
            *path = ShapePath::default().build();
        }
    }

    /// Helper to grab the objects custom collider color if it exists
    fn object_color(&self, object: DebugRenderObject, default: [f32; 4]) -> Color {
        let color = match object {
            DebugRenderObject::Collider(h, ..) => self.context.colliders.get(h).and_then(|co| {
                self.custom_colors
                    .get(Entity::from_bits(co.user_data as u64))
                    .map(|co| co.0)
                    .ok()
            }),
            _ => None,
        };

        let color = color.map(|co| co.as_hsla_f32()).unwrap_or(default);

        Color::Hsla {
            hue: color[0],
            saturation: color[1],
            lightness: color[2],
            alpha: color[3],
        }
    }
}

impl<'w, 's> DebugRenderBackend for RapierDebugRenderer<'w, 's> {
    /// Draw a debug line
    fn draw_line(
        &mut self,
        object: DebugRenderObject<'_>,
        a: Point<Real>,
        b: Point<Real>,
        color: [f32; 4],
    ) {
        let color = self.object_color(object, color);

        let shape_path_ref = self.shape_paths.entry(HashEqColor(color)).or_default();
        let shape_path = std::mem::take(shape_path_ref);
        *shape_path_ref = shape_path.add(&Line(Vec2::from(a), Vec2::from(b)));
    }
}
