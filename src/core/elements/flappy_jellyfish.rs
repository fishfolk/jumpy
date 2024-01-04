use crate::prelude::*;

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("flappy_jellyfish"))]
#[repr(C)]
pub struct FlappyJellyfishMeta {
    pub atlas: Handle<Atlas>,
    pub start_frame: u32,
    pub end_frame: u32,
    pub fps: f32,
}

impl FlappyJellyfishMeta {
    pub fn frames(&self) -> SVec<u32> {
        (self.start_frame..self.end_frame).collect()
    }
}

#[derive(Clone, Debug, HasSchema, Default)]
pub struct FlappyJellyfish {
    pub owner: Entity,
}

pub fn spawn(
    owner: Entity,
    jellyfish_ent: Entity,
    flappy_meta_handle: Handle<FlappyJellyfishMeta>,
) -> StaticSystem<(), ()> {
    (move |mut entities: ResMut<Entities>,
           mut jellyfishes: CompMut<Jellyfish>,
           mut flappy_jellyfishes: CompMut<FlappyJellyfish>,
           assets: Res<AssetServer>,
           mut atlas_sprites: CompMut<AtlasSprite>,
           mut animated_sprites: CompMut<AnimatedSprite>,
           mut transforms: CompMut<Transform>| {
        let Some(jellyfish) = jellyfishes.get_mut(jellyfish_ent) else {
            return;
        };
        let flappy_ent = entities.create();
        jellyfish.status = JellyfishStatus::Driving {
            owner,
            flappy: flappy_ent,
        };
        flappy_jellyfishes.insert(flappy_ent, FlappyJellyfish { owner });
        let flappy_meta = assets.get(flappy_meta_handle);
        atlas_sprites.insert(flappy_ent, AtlasSprite::new(flappy_meta.atlas));
        animated_sprites.insert(
            flappy_ent,
            AnimatedSprite {
                frames: flappy_meta.frames(),
                fps: flappy_meta.fps,
                repeat: true,
                ..default()
            },
        );
        let mut transf = *transforms.get(owner).unwrap();
        transf.translation.y += 75.0;
        transforms.insert(flappy_ent, transf);
        debug!("FLAPPY JELLYFISH | spawned");
    })
    .system()
}

pub fn kill(flappy: Entity) -> StaticSystem<(), ()> {
    (move |mut entities: ResMut<Entities>| {
        entities.kill(flappy);
        debug!("FLAPPY JELLYFISH | despawned");
    })
    .system()
}
