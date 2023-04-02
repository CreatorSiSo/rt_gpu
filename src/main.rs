use bevy::pbr::{MeshPipeline, PbrPlugin};
use bevy::prelude::*;
use bevy::render::render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext};
use bevy::render::renderer::RenderContext;
use bevy::render::RenderApp;

mod pbr_material;
use pbr_material::PbrMaterial;

fn main() {
	App::new()
		.add_plugins(
			DefaultPlugins
				.build()
				.add_after::<ImagePlugin, _>(RendererPlugin),
		)
		.add_startup_system(setup_scene)
		.run();
}

fn setup_scene(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<PbrMaterial>>,
) {
	// Add a camera so we can see the debug-render
	// commands.spawn(Camera3dBundle {
	// 	transform: Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
	// 	..default()
	// });

	// create a sphere mesh
	let sphere_mesh = meshes.add(
		Mesh::try_from(shape::Icosphere {
			subdivisions: 4,
			radius: 1.0,
		})
		.unwrap(),
	);

	// create a standard material
	let sphere_material = materials.add(PbrMaterial {
		base_color: Color::rgb(0.8, 0.7, 0.6),
		metallic: 0.2,
		perceptual_roughness: 0.7,
		..default()
	});

	// create a PbrBundle with the sphere mesh and material
	// commands.spawn(MaterialMeshBundle {
	// 	mesh: sphere_mesh,
	// 	material: sphere_material,
	// 	transform: Transform::from_xyz(0.0, 0.0, 0.0),
	// 	..default()
	// });

	// commands.spawn(SpotLightBundle {
	// 	spot_light: SpotLight {
	// 		color: Color::WHITE,
	// 		intensity: 500.0,
	// 		..default()
	// 	},
	// 	transform: Transform::from_xyz(2., 3., 2.).looking_at(Vec3::ZERO, Vec3::Y),
	// 	..default()
	// });
}

// Plugin to add systems related to the Renderer
pub struct RendererPlugin;

impl Plugin for RendererPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<MeshPipeline>();

		app.register_asset_reflect::<PbrMaterial>()
			.add_plugin(MaterialPlugin::<PbrMaterial>::default());
		app.world
			.resource_mut::<Assets<PbrMaterial>>()
			.set_untracked(
				Handle::<PbrMaterial>::default(),
				PbrMaterial {
					base_color: Color::rgb(1.0, 0.0, 0.5),
					..default()
				},
			);

		let render_app = match app.get_sub_app_mut(RenderApp) {
			Ok(render_app) => render_app,
			Err(_) => return,
		};

		let ray_tracing_pass_node = RayTracingPassNode {};

		let mut graph = render_app.world.resource_mut::<RenderGraph>();
		let draw_3d_graph = graph
			.get_sub_graph_mut(bevy::core_pipeline::core_3d::graph::NAME)
			.unwrap();

		draw_3d_graph.add_node("rt_node", ray_tracing_pass_node);
		draw_3d_graph.add_node_edge(
			"rt_node",
			bevy::core_pipeline::core_3d::graph::node::MAIN_PASS,
		);
	}
}

struct RayTracingPassNode {}

impl Node for RayTracingPassNode {
	fn run(
		&self,
		graph: &mut RenderGraphContext,
		render_context: &mut RenderContext,
		world: &World,
	) -> Result<(), NodeRunError> {
		dbg!(graph.input_info());
		Ok(())
	}
}
