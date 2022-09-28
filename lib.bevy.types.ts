// generated with https://github.com/jakobhellermann/bevy_reflect_ts_type_export
// bevy_core
type bool = boolean;
type f32 = number;
type f64 = number;
type i8 = number;
type i16 = number;
type i32 = number;
type i64 = number;
type isize = number;
type u8 = number;
type u16 = number;
type u32 = number;
type u64 = number;
type usize = number;
type Cowstr = String;

type Name = {
  hash: number,
  name: string,
};

const Name: BevyType<Name> = { typeName: "bevy_core::name::Name" };
// core
type Rangef32 = unknown;
type Duration = unknown;

const Rangef32: BevyType<Rangef32> = { typeName: "core::ops::range::Range<f32>" };
const Duration: BevyType<Duration> = { typeName: "core::time::Duration" };
// hashbrown
type HashSetString = unknown;

const HashSetString: BevyType<HashSetString> = { typeName: "hashbrown::set::HashSet<alloc::string::String>" };
// bevy_math
type Rect = {
  min: Vec2,
  max: Vec2,
};

const Rect: BevyType<Rect> = { typeName: "bevy_math::rect::Rect" };
// std
type Instant = unknown;

const Instant: BevyType<Instant> = { typeName: "std::time::Instant" };
// bevy_ecs
type Entity = unknown;

const Entity: BevyType<Entity> = { typeName: "bevy_ecs::entity::Entity" };
// bevy_time
type Stopwatch = {
  elapsed: Duration,
  paused: boolean,
};
type Time = {
  delta: Duration,
  last_update: Instant | null,
  delta_seconds_f64: number,
  delta_seconds: number,
  seconds_since_startup: number,
  time_since_startup: Duration,
  startup: Instant,
};
type Timer = {
  stopwatch: Stopwatch,
  duration: Duration,
  repeating: boolean,
  finished: boolean,
  times_finished_this_tick: number,
};

const Stopwatch: BevyType<Stopwatch> = { typeName: "bevy_time::stopwatch::Stopwatch" };
const Time: BevyType<Time> = { typeName: "bevy_time::time::Time" };
const Timer: BevyType<Timer> = { typeName: "bevy_time::timer::Timer" };
// bevy_asset
type HandleAnimationClip = {
  id: HandleId,
};
type HandleAudioSink = {
  id: HandleId,
};
type HandleAudioSource = {
  id: HandleId,
};
type HandleGltf = {
  id: HandleId,
};
type HandleGltfMesh = {
  id: HandleId,
};
type HandleGltfNode = {
  id: HandleId,
};
type HandleGltfPrimitive = {
  id: HandleId,
};
type HandleStandardMaterial = {
  id: HandleId,
};
type HandleMesh = {
  id: HandleId,
};
type HandleSkinnedMeshInverseBindposes = {
  id: HandleId,
};
type HandleShader = {
  id: HandleId,
};
type HandleImage = {
  id: HandleId,
};
type HandleDynamicScene = {
  id: HandleId,
};
type HandleScene = {
  id: HandleId,
};
type HandleColorMaterial = {
  id: HandleId,
};
type HandleTextureAtlas = {
  id: HandleId,
};
type HandleFont = {
  id: HandleId,
};
type HandleFontAtlasSet = {
  id: HandleId,
};
type HandleId = unknown;

const HandleAnimationClip: BevyType<HandleAnimationClip> = { typeName: "bevy_asset::handle::Handle<bevy_animation::AnimationClip>" };
const HandleAudioSink: BevyType<HandleAudioSink> = { typeName: "bevy_asset::handle::Handle<bevy_audio::audio_output::AudioSink>" };
const HandleAudioSource: BevyType<HandleAudioSource> = { typeName: "bevy_asset::handle::Handle<bevy_audio::audio_source::AudioSource>" };
const HandleGltf: BevyType<HandleGltf> = { typeName: "bevy_asset::handle::Handle<bevy_gltf::Gltf>" };
const HandleGltfMesh: BevyType<HandleGltfMesh> = { typeName: "bevy_asset::handle::Handle<bevy_gltf::GltfMesh>" };
const HandleGltfNode: BevyType<HandleGltfNode> = { typeName: "bevy_asset::handle::Handle<bevy_gltf::GltfNode>" };
const HandleGltfPrimitive: BevyType<HandleGltfPrimitive> = { typeName: "bevy_asset::handle::Handle<bevy_gltf::GltfPrimitive>" };
const HandleStandardMaterial: BevyType<HandleStandardMaterial> = { typeName: "bevy_asset::handle::Handle<bevy_pbr::pbr_material::StandardMaterial>" };
const HandleMesh: BevyType<HandleMesh> = { typeName: "bevy_asset::handle::Handle<bevy_render::mesh::mesh::Mesh>" };
const HandleSkinnedMeshInverseBindposes: BevyType<HandleSkinnedMeshInverseBindposes> = { typeName: "bevy_asset::handle::Handle<bevy_render::mesh::mesh::skinning::SkinnedMeshInverseBindposes>" };
const HandleShader: BevyType<HandleShader> = { typeName: "bevy_asset::handle::Handle<bevy_render::render_resource::shader::Shader>" };
const HandleImage: BevyType<HandleImage> = { typeName: "bevy_asset::handle::Handle<bevy_render::texture::image::Image>" };
const HandleDynamicScene: BevyType<HandleDynamicScene> = { typeName: "bevy_asset::handle::Handle<bevy_scene::dynamic_scene::DynamicScene>" };
const HandleScene: BevyType<HandleScene> = { typeName: "bevy_asset::handle::Handle<bevy_scene::scene::Scene>" };
const HandleColorMaterial: BevyType<HandleColorMaterial> = { typeName: "bevy_asset::handle::Handle<bevy_sprite::mesh2d::color_material::ColorMaterial>" };
const HandleTextureAtlas: BevyType<HandleTextureAtlas> = { typeName: "bevy_asset::handle::Handle<bevy_sprite::texture_atlas::TextureAtlas>" };
const HandleFont: BevyType<HandleFont> = { typeName: "bevy_asset::handle::Handle<bevy_text::font::Font>" };
const HandleFontAtlasSet: BevyType<HandleFontAtlasSet> = { typeName: "bevy_asset::handle::Handle<bevy_text::font_atlas_set::FontAtlasSet>" };
const HandleId: BevyType<HandleId> = { typeName: "bevy_asset::handle::HandleId" };
// bevy_gltf
type GltfExtras = {
  value: string,
};

const GltfExtras: BevyType<GltfExtras> = { typeName: "bevy_gltf::GltfExtras" };
// bevy_core_pipeline
type ClearColor = unknown;
type ClearColorConfig = unknown;
type Camera2d = {
  clear_color: ClearColorConfig,
};
type Camera3d = {
  clear_color: ClearColorConfig,
  depth_load_op: Camera3dDepthLoadOp,
};
type Camera3dDepthLoadOp = unknown;

const ClearColor: BevyType<ClearColor> = { typeName: "bevy_core_pipeline::clear_color::ClearColor" };
const ClearColorConfig: BevyType<ClearColorConfig> = { typeName: "bevy_core_pipeline::clear_color::ClearColorConfig" };
const Camera2d: BevyType<Camera2d> = { typeName: "bevy_core_pipeline::core_2d::camera_2d::Camera2d" };
const Camera3d: BevyType<Camera3d> = { typeName: "bevy_core_pipeline::core_3d::camera_3d::Camera3d" };
const Camera3dDepthLoadOp: BevyType<Camera3dDepthLoadOp> = { typeName: "bevy_core_pipeline::core_3d::camera_3d::Camera3dDepthLoadOp" };
// bevy_pbr
type CubemapVisibleEntities = {
};
type AmbientLight = {
  color: Color,
  brightness: number,
};
type DirectionalLight = {
  color: Color,
  illuminance: number,
  shadows_enabled: boolean,
  shadow_projection: OrthographicProjection,
  shadow_depth_bias: number,
  shadow_normal_bias: number,
};
type DirectionalLightShadowMap = {
  size: number,
};
type PointLight = {
  color: Color,
  intensity: number,
  range: number,
  radius: number,
  shadows_enabled: boolean,
  shadow_depth_bias: number,
  shadow_normal_bias: number,
};
type PointLightShadowMap = {
  size: number,
};
type SpotLight = {
  color: Color,
  intensity: number,
  range: number,
  radius: number,
  shadows_enabled: boolean,
  shadow_depth_bias: number,
  shadow_normal_bias: number,
  outer_angle: number,
  inner_angle: number,
};

const CubemapVisibleEntities: BevyType<CubemapVisibleEntities> = { typeName: "bevy_pbr::bundle::CubemapVisibleEntities" };
const AmbientLight: BevyType<AmbientLight> = { typeName: "bevy_pbr::light::AmbientLight" };
const DirectionalLight: BevyType<DirectionalLight> = { typeName: "bevy_pbr::light::DirectionalLight" };
const DirectionalLightShadowMap: BevyType<DirectionalLightShadowMap> = { typeName: "bevy_pbr::light::DirectionalLightShadowMap" };
const PointLight: BevyType<PointLight> = { typeName: "bevy_pbr::light::PointLight" };
const PointLightShadowMap: BevyType<PointLightShadowMap> = { typeName: "bevy_pbr::light::PointLightShadowMap" };
const SpotLight: BevyType<SpotLight> = { typeName: "bevy_pbr::light::SpotLight" };
// bevy_render
type Camera = {
  viewport: Viewport | null,
  priority: number,
  is_active: boolean,
};
type CameraRenderGraph = unknown;
type Viewport = {
  physical_position: UVec2,
  physical_size: UVec2,
  depth: Rangef32,
};
type OrthographicProjection = {
  left: number,
  right: number,
  bottom: number,
  top: number,
  near: number,
  far: number,
  window_origin: WindowOrigin,
  scaling_mode: ScalingMode,
  scale: number,
};
type PerspectiveProjection = {
  fov: number,
  aspect_ratio: number,
  near: number,
  far: number,
};
type Projection = unknown;
type ScalingMode = unknown;
type WindowOrigin = unknown;
type Color = unknown;
type SkinnedMesh = {
  inverse_bindposes: HandleSkinnedMeshInverseBindposes,
  joints: Entity[],
};
type Aabb = {
  center: Vec3A,
  half_extents: Vec3A,
};
type CubemapFrusta = {
};
type Frustum = {
};
type Msaa = {
  samples: number,
};
type ComputedVisibility = {
  is_visible_in_hierarchy: boolean,
  is_visible_in_view: boolean,
};
type Visibility = {
  is_visible: boolean,
};
type VisibleEntities = {
};

const Camera: BevyType<Camera> = { typeName: "bevy_render::camera::camera::Camera" };
const CameraRenderGraph: BevyType<CameraRenderGraph> = { typeName: "bevy_render::camera::camera::CameraRenderGraph" };
const Viewport: BevyType<Viewport> = { typeName: "bevy_render::camera::camera::Viewport" };
const OrthographicProjection: BevyType<OrthographicProjection> = { typeName: "bevy_render::camera::projection::OrthographicProjection" };
const PerspectiveProjection: BevyType<PerspectiveProjection> = { typeName: "bevy_render::camera::projection::PerspectiveProjection" };
const Projection: BevyType<Projection> = { typeName: "bevy_render::camera::projection::Projection" };
const ScalingMode: BevyType<ScalingMode> = { typeName: "bevy_render::camera::projection::ScalingMode" };
const WindowOrigin: BevyType<WindowOrigin> = { typeName: "bevy_render::camera::projection::WindowOrigin" };
const Color: BevyType<Color> = { typeName: "bevy_render::color::Color" };
const SkinnedMesh: BevyType<SkinnedMesh> = { typeName: "bevy_render::mesh::mesh::skinning::SkinnedMesh" };
const Aabb: BevyType<Aabb> = { typeName: "bevy_render::primitives::Aabb" };
const CubemapFrusta: BevyType<CubemapFrusta> = { typeName: "bevy_render::primitives::CubemapFrusta" };
const Frustum: BevyType<Frustum> = { typeName: "bevy_render::primitives::Frustum" };
const Msaa: BevyType<Msaa> = { typeName: "bevy_render::view::Msaa" };
const ComputedVisibility: BevyType<ComputedVisibility> = { typeName: "bevy_render::view::visibility::ComputedVisibility" };
const Visibility: BevyType<Visibility> = { typeName: "bevy_render::view::visibility::Visibility" };
const VisibleEntities: BevyType<VisibleEntities> = { typeName: "bevy_render::view::visibility::VisibleEntities" };
// bevy_sprite
type Mesh2dHandle = unknown;
type Anchor = unknown;
type Sprite = {
  color: Color,
  flip_x: boolean,
  flip_y: boolean,
  custom_size: Vec2 | null,
  rect: Rect | null,
  anchor: Anchor,
};

const Mesh2dHandle: BevyType<Mesh2dHandle> = { typeName: "bevy_sprite::mesh2d::mesh::Mesh2dHandle" };
const Anchor: BevyType<Anchor> = { typeName: "bevy_sprite::sprite::Anchor" };
const Sprite: BevyType<Sprite> = { typeName: "bevy_sprite::sprite::Sprite" };
// bevy_hierarchy
type Children = unknown;
type Parent = unknown;

const Children: BevyType<Children> = { typeName: "bevy_hierarchy::components::children::Children" };
const Parent: BevyType<Parent> = { typeName: "bevy_hierarchy::components::parent::Parent" };
// bevy_text
type HorizontalAlign = unknown;
type Text = {
  sections: TextSection[],
  alignment: TextAlignment,
};
type TextAlignment = {
  vertical: VerticalAlign,
  horizontal: HorizontalAlign,
};
type TextSection = {
  value: string,
  style: TextStyle,
};
type TextStyle = {
  font: HandleFont,
  font_size: number,
  color: Color,
};
type VerticalAlign = unknown;

const HorizontalAlign: BevyType<HorizontalAlign> = { typeName: "bevy_text::text::HorizontalAlign" };
const Text: BevyType<Text> = { typeName: "bevy_text::text::Text" };
const TextAlignment: BevyType<TextAlignment> = { typeName: "bevy_text::text::TextAlignment" };
const TextSection: BevyType<TextSection> = { typeName: "bevy_text::text::TextSection" };
const TextStyle: BevyType<TextStyle> = { typeName: "bevy_text::text::TextStyle" };
const VerticalAlign: BevyType<VerticalAlign> = { typeName: "bevy_text::text::VerticalAlign" };
// glam
type BVec2 = {
  x: boolean,
  y: boolean,
};
type BVec3 = {
  x: boolean,
  y: boolean,
  z: boolean,
};
type BVec4 = {
  x: boolean,
  y: boolean,
  z: boolean,
  w: boolean,
};
type BVec3A = unknown;
type BVec4A = unknown;
type Affine2 = {
  matrix2: Mat2,
  translation: Vec2,
};
type Affine3A = {
  matrix3: Mat3A,
  translation: Vec3A,
};
type Mat3 = {
  x_axis: Vec3,
  y_axis: Vec3,
  z_axis: Vec3,
};
type Mat2 = {
  x_axis: Vec2,
  y_axis: Vec2,
};
type Mat3A = {
  x_axis: Vec3A,
  y_axis: Vec3A,
  z_axis: Vec3A,
};
type Mat4 = {
  x_axis: Vec4,
  y_axis: Vec4,
  z_axis: Vec4,
  w_axis: Vec4,
};
type Quat = unknown;
type Vec3A = {
  x: number,
  y: number,
  z: number,
};
type Vec4 = {
  x: number,
  y: number,
  z: number,
  w: number,
};
type Vec2 = {
  x: number,
  y: number,
};
type Vec3 = {
  x: number,
  y: number,
  z: number,
};
type DAffine2 = {
  matrix2: DMat2,
  translation: DVec2,
};
type DAffine3 = {
  matrix3: DMat3,
  translation: DVec3,
};
type DMat2 = {
  x_axis: DVec2,
  y_axis: DVec2,
};
type DMat3 = {
  x_axis: DVec3,
  y_axis: DVec3,
  z_axis: DVec3,
};
type DMat4 = {
  x_axis: DVec4,
  y_axis: DVec4,
  z_axis: DVec4,
  w_axis: DVec4,
};
type DQuat = unknown;
type DVec2 = {
  x: number,
  y: number,
};
type DVec3 = {
  x: number,
  y: number,
  z: number,
};
type DVec4 = {
  x: number,
  y: number,
  z: number,
  w: number,
};
type IVec2 = {
  x: number,
  y: number,
};
type IVec3 = {
  x: number,
  y: number,
  z: number,
};
type IVec4 = {
  x: number,
  y: number,
  z: number,
  w: number,
};
type UVec2 = {
  x: number,
  y: number,
};
type UVec3 = {
  x: number,
  y: number,
  z: number,
};
type UVec4 = {
  x: number,
  y: number,
  z: number,
  w: number,
};

const BVec2: BevyType<BVec2> = { typeName: "glam::bool::bvec2::BVec2" };
const BVec3: BevyType<BVec3> = { typeName: "glam::bool::bvec3::BVec3" };
const BVec4: BevyType<BVec4> = { typeName: "glam::bool::bvec4::BVec4" };
const BVec3A: BevyType<BVec3A> = { typeName: "glam::bool::sse2::bvec3a::BVec3A" };
const BVec4A: BevyType<BVec4A> = { typeName: "glam::bool::sse2::bvec4a::BVec4A" };
const Affine2: BevyType<Affine2> = { typeName: "glam::f32::affine2::Affine2" };
const Affine3A: BevyType<Affine3A> = { typeName: "glam::f32::affine3a::Affine3A" };
const Mat3: BevyType<Mat3> = { typeName: "glam::f32::mat3::Mat3" };
const Mat2: BevyType<Mat2> = { typeName: "glam::f32::sse2::mat2::Mat2" };
const Mat3A: BevyType<Mat3A> = { typeName: "glam::f32::sse2::mat3a::Mat3A" };
const Mat4: BevyType<Mat4> = { typeName: "glam::f32::sse2::mat4::Mat4" };
const Quat: BevyType<Quat> = { typeName: "glam::f32::sse2::quat::Quat" };
const Vec3A: BevyType<Vec3A> = { typeName: "glam::f32::sse2::vec3a::Vec3A" };
const Vec4: BevyType<Vec4> = { typeName: "glam::f32::sse2::vec4::Vec4" };
const Vec2: BevyType<Vec2> = { typeName: "glam::f32::vec2::Vec2" };
const Vec3: BevyType<Vec3> = { typeName: "glam::f32::vec3::Vec3" };
const DAffine2: BevyType<DAffine2> = { typeName: "glam::f64::daffine2::DAffine2" };
const DAffine3: BevyType<DAffine3> = { typeName: "glam::f64::daffine3::DAffine3" };
const DMat2: BevyType<DMat2> = { typeName: "glam::f64::dmat2::DMat2" };
const DMat3: BevyType<DMat3> = { typeName: "glam::f64::dmat3::DMat3" };
const DMat4: BevyType<DMat4> = { typeName: "glam::f64::dmat4::DMat4" };
const DQuat: BevyType<DQuat> = { typeName: "glam::f64::dquat::DQuat" };
const DVec2: BevyType<DVec2> = { typeName: "glam::f64::dvec2::DVec2" };
const DVec3: BevyType<DVec3> = { typeName: "glam::f64::dvec3::DVec3" };
const DVec4: BevyType<DVec4> = { typeName: "glam::f64::dvec4::DVec4" };
const IVec2: BevyType<IVec2> = { typeName: "glam::i32::ivec2::IVec2" };
const IVec3: BevyType<IVec3> = { typeName: "glam::i32::ivec3::IVec3" };
const IVec4: BevyType<IVec4> = { typeName: "glam::i32::ivec4::IVec4" };
const UVec2: BevyType<UVec2> = { typeName: "glam::u32::uvec2::UVec2" };
const UVec3: BevyType<UVec3> = { typeName: "glam::u32::uvec3::UVec3" };
const UVec4: BevyType<UVec4> = { typeName: "glam::u32::uvec4::UVec4" };
// bevy_transform
type GlobalTransform = unknown;
type Transform = {
  translation: Vec3,
  rotation: Quat,
  scale: Vec3,
};

const GlobalTransform: BevyType<GlobalTransform> = { typeName: "bevy_transform::components::global_transform::GlobalTransform" };
const Transform: BevyType<Transform> = { typeName: "bevy_transform::components::transform::Transform" };
// bevy_animation
type AnimationPlayer = {
  paused: boolean,
  repeat: boolean,
  speed: number,
  elapsed: number,
  animation_clip: HandleAnimationClip,
};

const AnimationPlayer: BevyType<AnimationPlayer> = { typeName: "bevy_animation::AnimationPlayer" };
// bevy_ui
type FocusPolicy = unknown;
type Interaction = unknown;
type Size = {
  width: Val,
  height: Val,
};
type UiRect = {
  left: Val,
  right: Val,
  top: Val,
  bottom: Val,
};
type AlignContent = unknown;
type AlignItems = unknown;
type AlignSelf = unknown;
type CalculatedSize = {
  size: Size,
};
type Direction = unknown;
type Display = unknown;
type FlexDirection = unknown;
type FlexWrap = unknown;
type JustifyContent = unknown;
type Node = {
  size: Vec2,
};
type Overflow = unknown;
type PositionType = unknown;
type Style = {
  display: Display,
  position_type: PositionType,
  direction: Direction,
  flex_direction: FlexDirection,
  flex_wrap: FlexWrap,
  align_items: AlignItems,
  align_self: AlignSelf,
  align_content: AlignContent,
  justify_content: JustifyContent,
  position: UiRect,
  margin: UiRect,
  padding: UiRect,
  border: UiRect,
  flex_grow: number,
  flex_shrink: number,
  flex_basis: Val,
  size: Size,
  min_size: Size,
  max_size: Size,
  aspect_ratio: number | null,
  overflow: Overflow,
};
type UiColor = unknown;
type UiImage = unknown;
type Val = unknown;
type Button = {
};
type ImageMode = unknown;

const FocusPolicy: BevyType<FocusPolicy> = { typeName: "bevy_ui::focus::FocusPolicy" };
const Interaction: BevyType<Interaction> = { typeName: "bevy_ui::focus::Interaction" };
const Size: BevyType<Size> = { typeName: "bevy_ui::geometry::Size" };
const UiRect: BevyType<UiRect> = { typeName: "bevy_ui::geometry::UiRect" };
const AlignContent: BevyType<AlignContent> = { typeName: "bevy_ui::ui_node::AlignContent" };
const AlignItems: BevyType<AlignItems> = { typeName: "bevy_ui::ui_node::AlignItems" };
const AlignSelf: BevyType<AlignSelf> = { typeName: "bevy_ui::ui_node::AlignSelf" };
const CalculatedSize: BevyType<CalculatedSize> = { typeName: "bevy_ui::ui_node::CalculatedSize" };
const Direction: BevyType<Direction> = { typeName: "bevy_ui::ui_node::Direction" };
const Display: BevyType<Display> = { typeName: "bevy_ui::ui_node::Display" };
const FlexDirection: BevyType<FlexDirection> = { typeName: "bevy_ui::ui_node::FlexDirection" };
const FlexWrap: BevyType<FlexWrap> = { typeName: "bevy_ui::ui_node::FlexWrap" };
const JustifyContent: BevyType<JustifyContent> = { typeName: "bevy_ui::ui_node::JustifyContent" };
const Node: BevyType<Node> = { typeName: "bevy_ui::ui_node::Node" };
const Overflow: BevyType<Overflow> = { typeName: "bevy_ui::ui_node::Overflow" };
const PositionType: BevyType<PositionType> = { typeName: "bevy_ui::ui_node::PositionType" };
const Style: BevyType<Style> = { typeName: "bevy_ui::ui_node::Style" };
const UiColor: BevyType<UiColor> = { typeName: "bevy_ui::ui_node::UiColor" };
const UiImage: BevyType<UiImage> = { typeName: "bevy_ui::ui_node::UiImage" };
const Val: BevyType<Val> = { typeName: "bevy_ui::ui_node::Val" };
const Button: BevyType<Button> = { typeName: "bevy_ui::widget::button::Button" };
const ImageMode: BevyType<ImageMode> = { typeName: "bevy_ui::widget::image::ImageMode" };
