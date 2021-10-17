use macroquad::{experimental::animation::Animation, prelude::*};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(remote = "Color")]
pub struct ColorDef {
    #[serde(rename = "red", alias = "r")]
    pub r: f32,
    #[serde(rename = "green", alias = "g")]
    pub g: f32,
    #[serde(rename = "blue", alias = "b")]
    pub b: f32,
    #[serde(rename = "alpha", alias = "a")]
    pub a: f32,
}

impl From<Color> for ColorDef {
    fn from(other: Color) -> Self {
        ColorDef {
            r: other.r,
            g: other.g,
            b: other.b,
            a: other.a,
        }
    }
}

impl From<ColorDef> for Color {
    fn from(other: ColorDef) -> Self {
        Color {
            r: other.r,
            g: other.g,
            b: other.b,
            a: other.a,
        }
    }
}

pub mod color_opt {
    use super::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<Color>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "super::ColorDef")] &'a Color);

        value.as_ref().map(Helper).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(#[serde(with = "super::ColorDef")] Color);

        let helper = Option::deserialize(deserializer)?;
        Ok(helper.map(|Helper(external)| external))
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(remote = "Animation")]
pub struct AnimationDef {
    pub name: String,
    pub row: u32,
    pub frames: u32,
    pub fps: u32,
}

impl From<&Animation> for AnimationDef {
    fn from(other: &Animation) -> Self {
        AnimationDef {
            name: other.name.clone(),
            row: other.row,
            frames: other.frames,
            fps: other.fps,
        }
    }
}

impl From<AnimationDef> for Animation {
    fn from(other: AnimationDef) -> Self {
        Animation {
            name: other.name,
            row: other.row,
            frames: other.frames,
            fps: other.fps,
        }
    }
}

pub mod animation_vec {
    use super::{Animation, AnimationDef};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &[Animation], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "AnimationDef")] &'a Animation);

        value
            .iter()
            .map(Helper)
            .collect::<Vec<Helper>>()
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Animation>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(#[serde(with = "AnimationDef")] Animation);

        let helper = Vec::deserialize(deserializer)?;
        Ok(helper
            .iter()
            .map(|Helper(external)| external.clone())
            .collect())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(remote = "FilterMode")]
pub enum FilterModeDef {
    #[serde(rename = "linear")]
    Linear,
    #[serde(rename = "nearest_neighbor")]
    Nearest,
}

pub fn default_filter_mode() -> FilterMode {
    FilterMode::Nearest
}

//
// #[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
// #[serde(remote = "Interpolation")]
// pub enum InterpolationDef {
//     Linear,
//     Bezier,
// }
//
// #[derive(Debug, Clone, Deserialize)]
// #[serde(remote = "Curve")]
// pub struct CurveDef {
//     pub points: Vec<(f32, f32)>,
//     pub interpolation: Interpolation,
//     pub resolution: usize,
// }
//
// pub mod opt_curve {
//     use super::Curve;
//     use serde::{Deserialize, Deserializer, Serialize, Serializer};
//
//     pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Curve>, D::Error>
//         where
//             D: Deserializer<'de>,
//     {
//         #[derive(Deserialize)]
//         struct Helper(#[serde(with = "super::CurveDef")] Curve);
//
//         let helper = Option::deserialize(deserializer)?;
//         Ok(helper.map(|Helper(external)| external))
//     }
// }
//
// #[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
// #[serde(remote = "ColorCurve")]
// pub struct ColorCurveDef {
//     #[serde(with = "ColorDef")]
//     pub start: Color,
//     #[serde(with = "ColorDef")]
//     pub mid: Color,
//     #[serde(with = "ColorDef")]
//     pub end: Color,
// }
//
// #[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
// #[serde(remote = "BlendMode")]
// pub enum BlendModeDef {
//     Alpha,
//     Additive,
// }
//
// #[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
// #[serde(remote = "EmissionShape")]
// pub enum EmissionShapeDef {
//     Point,
//     Rect { width: f32, height: f32 },
//     Sphere { radius: f32 },
// }
//
// #[derive(Debug, Clone, Deserialize)]
// #[serde(remote = "ParticleShape")]
// pub enum ParticleShapeDef {
//     Rectangle,
//     Circle {
//         subdivisions: u32,
//     },
//     CustomMesh {
//         vertices: Vec<f32>,
//         indices: Vec<u16>,
//     },
// }
//
// #[derive(Debug, Clone, Deserialize)]
// #[serde(remote = "AtlasConfig")]
// pub struct AtlasConfigDef {
//     n: u16,
//     m: u16,
//     start_index: u16,
//     end_index: u16,
// }
//
// pub mod opt_atlas_config {
//     use super::AtlasConfig;
//     use serde::{Deserialize, Deserializer, Serialize, Serializer};
//
//     pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<AtlasConfig>, D::Error>
//         where
//             D: Deserializer<'de>,
//     {
//         #[derive(Deserialize)]
//         struct Helper(#[serde(with = "super::AtlasConfigDef")] AtlasConfig);
//
//         let helper = Option::deserialize(deserializer)?;
//         Ok(helper.map(|Helper(external)| external))
//     }
// }
//
// #[derive(Debug, Clone, Deserialize)]
// #[serde(remote = "ParticleMaterial")]
// pub struct ParticleMaterialDef {
//     vertex: String,
//     fragment: String,
// }
//
// pub mod opt_particle_material {
//     use super::ParticleMaterial;
//     use serde::{Deserialize, Deserializer, Serialize, Serializer};
//
//     pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<ParticleMaterial>, D::Error>
//         where
//             D: Deserializer<'de>,
//     {
//         #[derive(Deserialize)]
//         struct Helper(#[serde(with = "super::ParticleMaterialDef")] ParticleMaterial);
//
//         let helper = Option::deserialize(deserializer)?;
//         Ok(helper.map(|Helper(external)| external))
//     }
// }
//
// #[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
// #[serde(remote = "PostProcessing")]
// pub struct PostProcessingDef;
//
// pub mod opt_post_processing {
//     use super::PostProcessing;
//     use serde::{Deserialize, Deserializer, Serialize, Serializer};
//
//     pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<PostProcessing>, D::Error>
//         where
//             D: Deserializer<'de>,
//     {
//         #[derive(Deserialize)]
//         struct Helper(#[serde(with = "super::PostProcessingDef")] PostProcessing);
//
//         let helper = Option::deserialize(deserializer)?;
//         Ok(helper.map(|Helper(external)| external))
//     }
// }
//
// #[derive(Debug, Clone, Deserialize)]
// #[serde(remote = "EmitterConfig")]
// pub struct EmitterConfigDef {
//     pub local_coords: bool,
//     pub emission_shape: EmissionShape,
//     pub one_shot: bool,
//     pub lifetime: f32,
//     pub lifetime_randomness: f32,
//     pub explosiveness: f32,
//     pub amount: u32,
//     #[serde(with = "ParticleShapeDef")]
//     pub shape: ParticleShape,
//     pub emitting: bool,
//     #[serde(with = "super::def_vec2")]
//     pub initial_direction: Vec2,
//     pub initial_direction_spread: f32,
//     pub initial_velocity: f32,
//     pub initial_velocity_randomness: f32,
//     pub linear_accel: f32,
//     pub size: f32,
//     pub size_randomness: f32,
//     #[serde(with = "opt_curve")]
//     pub size_curve: Option<Curve>,
//     #[serde(with = "BlendModeDef")]
//     pub blend_mode: BlendMode,
//     #[serde(with = "ColorCurveDef")]
//     pub colors_curve: ColorCurve,
//     #[serde(with = "super::def_vec2")]
//     pub gravity: Vec2,
//     #[serde(skip)]
//     pub texture: Option<Texture2D>,
//     #[serde(with = "opt_atlas_config")]
//     pub atlas: Option<AtlasConfig>,
//     #[serde(with = "opt_particle_material")]
//     pub material: Option<ParticleMaterial>,
//     #[serde(with = "opt_post_processing")]
//     pub post_processing: Option<PostProcessing>,
// }
