use crate::prelude::*;
use rapier2d::prelude as rapier;

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::Last, debug_render_colliders)
        .add_system_to_stage(CoreStage::Last, debug_render_damage_regions)
        .add_system_to_stage(CoreStage::Last, debug_render_emote_regions);
}

/// Resource configuring various debugging settings.
#[derive(Copy, Clone, TypeUlid, Default)]
#[ulid = "01GQG8DR27S5A5S4CR84NE7CXH"]
pub struct DebugSettings {
    /// Whether or not to render kinematic collider shapes.
    pub show_kinematic_colliders: bool,
    /// Whether or not to render damage region collider shapes.
    pub show_damage_regions: bool,
}

/// Resource containing the physics debug line entity.
#[derive(TypeUlid)]
#[ulid = "01GRYZGG44HPZB3X0N2MT7RMT2"]
pub struct RapierDebugContext {
    path_entity: Entity,
    debug_pipeline: rapier::DebugRenderPipeline,
}

struct RapierDebugBackend<'a> {
    points: &'a mut Vec<Vec2>,
    line_breaks: &'a mut Vec<usize>,
}

impl<'a> rapier::DebugRenderBackend for RapierDebugBackend<'a> {
    fn draw_line(
        &mut self,
        object: rapier::DebugRenderObject,
        a: rapier::Point<rapier::Real>,
        b: rapier::Point<rapier::Real>,
        // TODO: implement multi-colored rendering
        _color: [f32; 4],
    ) {
        let render = match object {
            rapier::DebugRenderObject::RigidBody(_, body) => body.is_enabled(),
            rapier::DebugRenderObject::Collider(_, collider) => collider.is_enabled(),
            rapier::DebugRenderObject::ImpulseJoint(_, _) => true,
            rapier::DebugRenderObject::MultibodyJoint(_, _, _) => true,
            rapier::DebugRenderObject::Other => true,
        };
        if render {
            self.points.push(vec2(a.x, a.y));
            self.points.push(vec2(b.x, b.y));
            self.line_breaks.push(self.points.len());
        }
    }
}

impl Clone for RapierDebugContext {
    fn clone(&self) -> Self {
        Self {
            path_entity: self.path_entity,
            debug_pipeline: default(),
        }
    }
}

impl FromWorld for RapierDebugContext {
    fn from_world(world: &mut World) -> Self {
        let entities = world.resource::<Entities>();
        let mut entities = entities.borrow_mut();
        let path_entity = entities.create();

        let transforms = world.components.get::<Transform>();
        let mut transforms = transforms.borrow_mut();
        transforms.insert(
            path_entity,
            Transform::from_translation(vec3(0.0, 0.0, -1.0)),
        );

        Self {
            path_entity,
            debug_pipeline: default(),
        }
    }
}

fn debug_render_colliders(
    settings: Res<DebugSettings>,
    mut collision_world: CollisionWorld,
    transforms: Comp<Transform>,
    mut paths: CompMut<Path2d>,
    mut debug_context: ResMut<RapierDebugContext>,
) {
    if settings.show_kinematic_colliders {
        // TODO: It's unfortunate that we are doing an extra sync here, just for debug rendering. We
        // should try find a way to avoid this. Without this, the collider body positions will be
        // out of sync when they are rendered.
        collision_world.update(&transforms);

        let mut points = Vec::new();
        let mut line_breaks = Vec::new();

        debug_context.debug_pipeline.render_colliders(
            &mut RapierDebugBackend {
                points: &mut points,
                line_breaks: &mut line_breaks,
            },
            &collision_world.ctx.rigid_body_set,
            &collision_world.ctx.collider_set,
        );

        // TODO: Provide a way to change the collider colors
        // An orange-y color
        const DEFAULT_COLOR: [f32; 4] = [205.0 / 255.0, 94.0 / 255.0, 15.0 / 255.0, 1.0];

        paths.insert(
            debug_context.path_entity,
            Path2d {
                color: DEFAULT_COLOR,
                points,
                line_breaks,
                ..default()
            },
        );
    } else {
        paths.remove(debug_context.path_entity);
    }
}

fn debug_render_damage_regions(
    settings: Res<DebugSettings>,
    entities: Res<Entities>,
    regions: Comp<DamageRegion>,
    transforms: Comp<Transform>,
    mut paths: CompMut<Path2d>,
) {
    let path_for_region = |rotation: f32, region: &DamageRegion| {
        let rect = Rect::new(0.0, 0.0, region.size.x, region.size.y);

        // The collision boxes don't rotate, so apply the opposite rotation of the object to the
        // debug lines to keep it upright.
        let angle = Vec2::from_angle(-rotation);

        // Red color
        const COLOR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        Path2d {
            color: COLOR,
            points: vec![
                angle.rotate(rect.top_left()),
                angle.rotate(rect.top_right()),
                angle.rotate(rect.bottom_right()),
                angle.rotate(rect.bottom_left()),
                angle.rotate(rect.top_left()),
            ],
            thickness: 1.0,
            ..default()
        }
    };

    if settings.show_damage_regions {
        for (ent, (region, transform)) in entities.iter_with((&regions, &transforms)) {
            paths.insert(
                ent,
                path_for_region(transform.rotation.to_euler(glam::EulerRot::XYZ).2, region),
            );
        }
    } else {
        for ent in entities.iter_with_bitset(regions.bitset()) {
            paths.remove(ent);
        }
    }
}

fn debug_render_emote_regions(
    settings: Res<DebugSettings>,
    entities: Res<Entities>,
    regions: Comp<EmoteRegion>,
    transforms: Comp<Transform>,
    mut paths: CompMut<Path2d>,
) {
    let path_for_region = |rotation: f32, region: &EmoteRegion| {
        let rect = Rect::new(0.0, 0.0, region.size.x, region.size.y);

        // The collision boxes don't rotate, so apply the opposite rotation of the object to the
        // debug lines to keep it upright.
        let angle = Vec2::from_angle(-rotation);

        // Green color
        const COLOR: [f32; 4] = [39.0 / 255.0, 191.0 / 255.0, 68.0 / 255.0, 1.0];
        Path2d {
            color: COLOR,
            points: vec![
                angle.rotate(rect.top_left()),
                angle.rotate(rect.top_right()),
                angle.rotate(rect.bottom_right()),
                angle.rotate(rect.bottom_left()),
                angle.rotate(rect.top_left()),
            ],
            thickness: 1.0,
            ..default()
        }
    };

    if settings.show_damage_regions {
        for (ent, (region, transform)) in entities.iter_with((&regions, &transforms)) {
            paths.insert(
                ent,
                path_for_region(transform.rotation.to_euler(glam::EulerRot::XYZ).2, region),
            );
        }
    } else {
        for ent in entities.iter_with_bitset(regions.bitset()) {
            paths.remove(ent);
        }
    }
}
