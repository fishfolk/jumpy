use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::Last, debug_render_kinematic_colliders)
        .add_system_to_stage(CoreStage::Last, debug_render_damage_regions);
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

fn debug_render_kinematic_colliders(
    settings: Res<DebugSettings>,
    entities: Res<Entities>,
    bodies: Comp<KinematicBody>,
    transforms: Comp<Transform>,
    mut paths: CompMut<Path2d>,
) {
    let path_for_body = |rotation: f32, body: &KinematicBody| {
        let rect = Rect::new(body.offset.x, body.offset.y, body.size.x, body.size.y);

        // The collision boxes don't rotate, so apply the opposite rotation of the object to the
        // debug lines to keep it upright.
        let angle = Vec2::from_angle(-rotation);

        Path2d {
            // An orange-y color
            color: [205.0 / 255.0, 94.0 / 255.0, 15.0 / 255.0, 1.0],
            points: vec![
                angle.rotate(rect.top_left()),
                angle.rotate(rect.top_right()),
                angle.rotate(rect.bottom_right()),
                angle.rotate(rect.bottom_left()),
                angle.rotate(rect.top_left()),
            ],
            thickness: 0.9,
            ..default()
        }
    };

    if settings.show_kinematic_colliders {
        for (ent, (body, transform)) in entities.iter_with((&bodies, &transforms)) {
            if body.is_deactivated {
                paths.remove(ent);
            } else {
                paths.insert(
                    ent,
                    path_for_body(transform.rotation.to_euler(glam::EulerRot::XYZ).2, body),
                );
            }
        }
    } else {
        for ent in entities.iter_with_bitset(bodies.bitset()) {
            paths.remove(ent);
        }
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

        Path2d {
            // Red color
            color: [1.0, 0.0, 0.0, 1.0],
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
