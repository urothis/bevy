use crate::{primitives::Frustum, Camera, CameraProjection, OrthographicProjection, Projection};
use bevy_ecs::prelude::*;
use bevy_reflect::{std_traits::ReflectDefault, Reflect, ReflectDeserialize, ReflectSerialize};
use bevy_transform::prelude::{GlobalTransform, Transform};
use serde::{Deserialize, Serialize};
use wgpu_types::{LoadOp, TextureFormat, TextureUsages};

/// A 2D camera component. Enables the 2D render graph for a [`Camera`].
#[derive(Component, Default, Reflect, Clone)]
#[reflect(Component, Default, Clone)]
#[require(
    Camera,
    Projection::Orthographic(OrthographicProjection::default_2d()),
    Frustum = OrthographicProjection::default_2d().compute_frustum(&GlobalTransform::from(Transform::default())),
)]
pub struct Camera2d;

/// A 3D camera component. Enables the main 3D render graph for a [`Camera`].
///
/// The camera coordinate space is right-handed X-right, Y-up, Z-back.
/// This means "forward" is -Z.
#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default, Clone)]
#[require(Camera, Projection)]
pub struct Camera3d {
    /// The depth clear operation to perform for the main 3d pass.
    pub depth_load_op: Camera3dDepthLoadOp,
    /// The texture usages for the depth texture created for the main 3d pass.
    pub depth_texture_usages: Camera3dDepthTextureUsage,
}

impl Default for Camera3d {
    fn default() -> Self {
        Self {
            depth_load_op: Default::default(),
            depth_texture_usages: TextureUsages::RENDER_ATTACHMENT.into(),
        }
    }
}

#[derive(Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize, Clone)]
pub struct Camera3dDepthTextureUsage(pub u32);

impl From<TextureUsages> for Camera3dDepthTextureUsage {
    fn from(value: TextureUsages) -> Self {
        Self(value.bits())
    }
}

impl From<Camera3dDepthTextureUsage> for TextureUsages {
    fn from(value: Camera3dDepthTextureUsage) -> Self {
        Self::from_bits_truncate(value.0)
    }
}

/// The depth clear operation to perform for the main 3d pass.
#[derive(Reflect, Serialize, Deserialize, Clone, Debug)]
#[reflect(Serialize, Deserialize, Clone, Default)]
pub enum Camera3dDepthLoadOp {
    /// Clear with a specified value.
    /// Note that 0.0 is the far plane due to bevy's use of reverse-z projections.
    Clear(f32),
    /// Load from memory.
    Load,
}

impl Default for Camera3dDepthLoadOp {
    fn default() -> Self {
        Camera3dDepthLoadOp::Clear(0.0)
    }
}

impl From<Camera3dDepthLoadOp> for LoadOp<f32> {
    fn from(config: Camera3dDepthLoadOp) -> Self {
        match config {
            Camera3dDepthLoadOp::Clear(x) => LoadOp::Clear(x),
            Camera3dDepthLoadOp::Load => LoadOp::Load,
        }
    }
}

/// If this component is added to a camera, the camera will use an intermediate "high dynamic range" render texture.
/// This allows rendering with a wider range of lighting values. However, this does *not* affect
/// whether the camera will render with hdr display output (which bevy does not support currently)
/// and only affects the intermediate render texture.
#[derive(Component, Default, Copy, Clone, Reflect, PartialEq, Eq, Hash, Debug)]
#[reflect(Component, Default, PartialEq, Hash, Debug)]
pub struct Hdr;

/// Color space for alpha compositing. Affects how overlapping semi-transparent layers blend.
#[derive(Component, Copy, Clone, Reflect, PartialEq, Eq, Hash, Debug, Default)]
#[reflect(Component, PartialEq, Hash, Debug, Default)]
pub enum CompositingSpace {
    /// Gamma-encoded blending. Matches most image editors. Uses default sRGB target.
    #[default]
    Srgb,
    /// Linear light blending. Physically correct.
    Linear,
    /// Perceptually uniform blending. Often smoother gradients. Requires [`Hdr`] because its value can be outside [0, 1].
    Oklab,
}

impl CompositingSpace {
    /// Whether this is the linear space, which needs no encode step.
    #[inline]
    pub fn is_linear(self) -> bool {
        matches!(self, CompositingSpace::Linear)
    }
}

/// The format of the depth/stencil texture created for a camera's main pass.
///
/// This is a component on Camera rather than a field on Camera3d so
/// custom camera render graphs and 2D/3D camera views can use the same
/// attachment selection path. The default remains Depth32Float
/// for compatibility with Bevy's historical 3D depth texture.
#[derive(
    Component, Reflect, Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Hash,
)]
#[reflect(Component, Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub enum DepthStencilFormat {
    /// A stencil-only attachment.
    Stencil8,
    /// A 16-bit unsigned-normalized depth attachment.
    Depth16Unorm,
    /// A depth attachment with at least 24 bits of depth precision.
    Depth24Plus,
    /// A depth attachment with at least 24 bits of depth precision and 8 bits of stencil.
    Depth24PlusStencil8,
    /// A 32-bit floating-point depth attachment.
    #[default]
    Depth32Float,
    /// A 32-bit floating-point depth attachment with 8 bits of stencil.
    Depth32FloatStencil8,
}

impl DepthStencilFormat {
    /// Returns the underlying WebGPU texture format.
    #[inline]
    pub const fn format(self) -> TextureFormat {
        match self {
            Self::Stencil8 => TextureFormat::Stencil8,
            Self::Depth16Unorm => TextureFormat::Depth16Unorm,
            Self::Depth24Plus => TextureFormat::Depth24Plus,
            Self::Depth24PlusStencil8 => TextureFormat::Depth24PlusStencil8,
            Self::Depth32Float => TextureFormat::Depth32Float,
            Self::Depth32FloatStencil8 => TextureFormat::Depth32FloatStencil8,
        }
    }

    /// Returns whether this format has a stencil aspect.
    #[inline]
    pub const fn has_stencil(self) -> bool {
        matches!(
            self,
            Self::Stencil8 | Self::Depth24PlusStencil8 | Self::Depth32FloatStencil8
        )
    }
}

impl From<DepthStencilFormat> for TextureFormat {
    #[inline]
    fn from(format: DepthStencilFormat) -> Self {
        match format {
            DepthStencilFormat::Stencil8 => TextureFormat::Stencil8,
            DepthStencilFormat::Depth16Unorm => TextureFormat::Depth16Unorm,
            DepthStencilFormat::Depth24Plus => TextureFormat::Depth24Plus,
            DepthStencilFormat::Depth24PlusStencil8 => TextureFormat::Depth24PlusStencil8,
            DepthStencilFormat::Depth32Float => TextureFormat::Depth32Float,
            DepthStencilFormat::Depth32FloatStencil8 => TextureFormat::Depth32FloatStencil8,
        }
    }
}

/// Selects the stencil test used by camera-view pipelines.
///
/// Disabled preserves Bevy's normal depth-only behavior. Equal enables a
/// read-only stencil comparison against the render pass's stencil reference,
/// which is useful for rendering a view through a previously written portal
/// mask.
#[derive(
    Component, Reflect, Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Hash,
)]
#[reflect(Component, Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub enum StencilTest {
    /// Do not test the stencil aspect.
    #[default]
    Disabled,
    /// Only pass fragments whose stencil value equals the pass reference.
    Equal,
}
