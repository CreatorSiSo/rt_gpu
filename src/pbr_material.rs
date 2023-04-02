use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, AsBindGroupShaderType, ShaderType};

/// Mostly a copy of [`bevy::pbr::StandardMaterial`]
///
/// A material with "standard" properties used in PBR lighting
/// Standard property values with pictures here
/// <https://google.github.io/filament/Material%20Properties.pdf>.
#[derive(AsBindGroup, Clone, Debug, FromReflect, Reflect, TypeUuid)]
#[uuid = "d0324576-938b-42ac-8604-7bd3febf0e37"]
#[uniform(0, PbrMaterialUniform)]
pub struct PbrMaterial {
	/// The color of the surface of the material before lighting.
	///
	/// Doubles as diffuse albedo for non-metallic, specular for metallic and a mix for everything
	/// in between. If used together with a `base_color_texture`, this is factored into the final
	/// base color as `base_color * base_color_texture_value`
	///
	/// Defaults to [`Color::WHITE`].
	pub base_color: Color,

	// Use a color for user friendliness even though we technically don't use the alpha channel
	// Might be used in the future for exposure correction in HDR
	/// Color the material "emits" to the camera.
	///
	/// This is typically used for monitor screens or LED lights.
	/// Anything that can be visible even in darkness.
	///
	/// The emissive color is added to what would otherwise be the material's visible color.
	/// This means that for a light emissive value, in darkness,
	/// you will mostly see the emissive component.
	///
	/// The default emissive color is black, which doesn't add anything to the material color.
	///
	/// Note that **an emissive material won't light up surrounding areas like a light source**,
	/// it just adds a value to the color seen on screen.
	pub emissive: Color,

	/// Linear perceptual roughness, clamped to `[0.089, 1.0]` in the shader.
	///
	/// Defaults to `0.5`.
	///
	/// Low values result in a "glossy" material with specular highlights,
	/// while values close to `1` result in rough materials.
	///
	/// If used together with a roughness/metallic texture, this is factored into the final base
	/// color as `roughness * roughness_texture_value`.
	///
	/// 0.089 is the minimum floating point value that won't be rounded down to 0 in the
	/// calculations used.
	//
	// Technically for 32-bit floats, 0.045 could be used.
	// See <https://google.github.io/filament/Filament.html#materialsystem/parameterization/>
	pub perceptual_roughness: f32,

	/// How "metallic" the material appears, within `[0.0, 1.0]`.
	///
	/// This should be set to 0.0 for dielectric materials or 1.0 for metallic materials.
	/// For a hybrid surface such as corroded metal, you may need to use in-between values.
	///
	/// Defaults to `0.00`, for dielectric.
	///
	/// If used together with a roughness/metallic texture, this is factored into the final base
	/// color as `metallic * metallic_texture_value`.
	pub metallic: f32,
}

impl Default for PbrMaterial {
	fn default() -> Self {
		Self {
			// White because it gets multiplied with texture values if someone uses a texture.
			base_color: Color::WHITE,
			emissive: Color::BLACK,
			// Matches Blender's default roughness.
			perceptual_roughness: 0.5,
			// Metallic should generally be set to 0.0 or 1.0.
			metallic: 0.0,
		}
	}
}

impl Material for PbrMaterial {
	fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
		bevy::render::render_resource::ShaderRef::Default
	}

	fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
		bevy::render::render_resource::ShaderRef::Default
	}

	fn alpha_mode(&self) -> AlphaMode {
		AlphaMode::Opaque
	}

	fn depth_bias(&self) -> f32 {
		0.0
	}

	fn prepass_vertex_shader() -> bevy::render::render_resource::ShaderRef {
		bevy::render::render_resource::ShaderRef::Default
	}

	fn prepass_fragment_shader() -> bevy::render::render_resource::ShaderRef {
		bevy::render::render_resource::ShaderRef::Default
	}

	fn specialize(
		pipeline: &bevy::pbr::MaterialPipeline<Self>,
		descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
		layout: &bevy::render::mesh::MeshVertexBufferLayout,
		key: bevy::pbr::MaterialPipelineKey<Self>,
	) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
		Ok(())
	}
}

/// The GPU representation of the uniform data of a [`PbrMaterial`].
#[derive(Clone, Default, ShaderType)]
struct PbrMaterialUniform {
	/// Doubles as diffuse albedo for non-metallic, specular for metallic and a mix for everything
	/// in between.
	pub base_color: Vec4,
	// Use a color for user friendliness even though we technically don't use the alpha channel
	// Might be used in the future for exposure correction in HDR
	pub emissive: Vec4,
	/// Linear perceptual roughness, clamped to [0.089, 1.0] in the shader
	/// Defaults to minimum of 0.089
	pub roughness: f32,
	/// From [0.0, 1.0], dielectric to pure metallic
	pub metallic: f32,
}

impl AsBindGroupShaderType<PbrMaterialUniform> for PbrMaterial {
	fn as_bind_group_shader_type(
		&self,
		_images: &bevy::render::render_asset::RenderAssets<Image>,
	) -> PbrMaterialUniform {
		PbrMaterialUniform {
			base_color: self.base_color.as_linear_rgba_f32().into(),
			emissive: self.base_color.as_linear_rgba_f32().into(),
			roughness: self.perceptual_roughness,
			metallic: self.metallic,
		}
	}
}
