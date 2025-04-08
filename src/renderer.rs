use glam::{Vec2, Vec3, Vec4};
use std::{borrow::Cow, collections::HashMap};
use wgpu::{util::DeviceExt, PipelineCompilationOptions};

#[repr(C)]
#[repr(align(8))]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
	width: u32,
	height: u32,
}

#[repr(C)]
#[repr(align(16))]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Sphere {
	pub position: Vec3,
	pub radius: f32,
	pub color: Vec4,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	pub position: Vec3,
	pub uv: Vec2,
}

impl Vertex {
	const fn new(position: Vec3, uv: Vec2) -> Self {
		Self { position, uv }
	}

	fn descriptor<'a>() -> wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					shader_location: 0,
					format: wgpu::VertexFormat::Float32x3,
				},
				wgpu::VertexAttribute {
					offset: std::mem::size_of::<Vec3>() as wgpu::BufferAddress,
					shader_location: 1,
					format: wgpu::VertexFormat::Float32x2,
				},
			],
		}
	}
}

#[rustfmt::skip]
const QUAD_VERTICES: &[Vertex] = &[
	Vertex::new(Vec3::new(-1.0, -1.0, 0.0), Vec2::new(-1.0, -1.0)),
	Vertex::new(Vec3::new( 1.0,  1.0, 0.0), Vec2::new(1.0, 1.0)),
	Vertex::new(Vec3::new( 1.0, -1.0, 0.0), Vec2::new(1.0, -1.0)),
	Vertex::new(Vec3::new(-1.0,  1.0, 0.0), Vec2::new(-1.0, 1.0)),
];

#[rustfmt::skip]
const QUAD_INDICES: &[u16] = &[
	0, 1, 2,
	0, 3, 1
];

fn create_bind_group(
	device: &wgpu::Device,
	label: &'static str,
	layout_entries: &[wgpu::BindGroupLayoutEntry],
	entries: &[wgpu::BindGroupEntry],
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
	let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		label: Some(&format!("{label} Bind Group Layout")),
		entries: layout_entries,
	});

	let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
		label: Some(&format!("{label} Bind Group")),
		layout: &bind_group_layout,
		entries,
	});

	(bind_group_layout, bind_group)
}

fn create_shader_module(
	device: &wgpu::Device,
	label: &'static str,
	source: &'static str,
) -> wgpu::ShaderModule {
	device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some(label),
		source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
	})
}

pub struct Renderer {
	pub device: wgpu::Device,
	queue: wgpu::Queue,
	render_pipeline: wgpu::RenderPipeline,
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	objects_bind_group: wgpu::BindGroup,
	camera_buffer: wgpu::Buffer,
	camera_bind_group: wgpu::BindGroup,
}

impl Renderer {
	pub async fn new(
		adapter: wgpu::Adapter,
		swapchain_format: wgpu::TextureFormat,
	) -> anyhow::Result<Self> {
		// Create the logical device and command queue
		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: None,
					required_features: wgpu::Features::BUFFER_BINDING_ARRAY
						| wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY,
					..Default::default()
				},
				None,
				// Some(Path::new("./traces")),
			)
			.await?;

		device.start_capture();

		let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Vertex Buffer"),
			contents: bytemuck::cast_slice(QUAD_VERTICES),
			usage: wgpu::BufferUsages::VERTEX,
		});

		let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Index Buffer"),
			contents: bytemuck::cast_slice(QUAD_INDICES),
			usage: wgpu::BufferUsages::INDEX,
		});

		let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Camera Buffer"),
			contents: bytemuck::cast_slice(&[CameraUniform {
				width: 1,
				height: 1,
			}]),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});

		let (camera_bind_group_layout, camera_bind_group) = create_bind_group(
			&device,
			"Camera",
			&[wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			}],
			&[wgpu::BindGroupEntry {
				binding: 0,
				resource: wgpu::BindingResource::Buffer(camera_buffer.as_entire_buffer_binding()),
			}],
		);

		let spheres = &[
			Sphere {
				radius: 1.0,
				position: Vec3::new(-1.5, 0.0, 0.5),
				color: Vec4::new(1.0, 0.1, 0.1, 1.0),
			},
			Sphere {
				radius: 0.5,
				position: Vec3::new(-0.5, 0.0, 0.2),
				color: Vec4::new(0.1, 1.0, 0.1, 1.0),
			},
			Sphere {
				radius: 0.25,
				position: Vec3::new(0.0, 0.0, 0.0),
				color: Vec4::new(0.1, 0.1, 1.0, 1.0),
			},
			Sphere {
				radius: 0.5,
				position: Vec3::new(0.5, 0.0, 0.2),
				color: Vec4::new(0.0, 1.0, 0.1, 1.0),
			},
			Sphere {
				radius: 1.0,
				position: Vec3::new(1.5, 0.0, 0.5),
				color: Vec4::new(1.0, 0.1, 0.1, 1.0),
			},
		];

		let object_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Object Buffer"),
			contents: bytemuck::cast_slice(spheres),
			usage: wgpu::BufferUsages::STORAGE,
		});

		let (objects_bind_group_layout, objects_bind_group) = create_bind_group(
			&device,
			"Object",
			&[wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Storage { read_only: true },
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			}],
			&[wgpu::BindGroupEntry {
				binding: 0,
				resource: object_buffer.as_entire_binding(),
			}],
		);

		// Load the shaders from disk
		let shader = create_shader_module(&device, "Screen Shader", include_str!("shader.wgsl"));

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[&camera_bind_group_layout, &objects_bind_group_layout],
			push_constant_ranges: &[],
		});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main".into(),
				buffers: &[Vertex::descriptor()],
				compilation_options: PipelineCompilationOptions {
					constants: &HashMap::new(),
					zero_initialize_workgroup_memory: false,
				},
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main".into(),
				targets: &[Some(swapchain_format.into())],
				compilation_options: PipelineCompilationOptions {
					constants: &HashMap::new(),
					zero_initialize_workgroup_memory: false,
				},
			}),
			primitive: wgpu::PrimitiveState::default(),
			depth_stencil: None,
			multisample: wgpu::MultisampleState::default(),
			multiview: None,
			cache: None,
		});

		Ok(Self {
			device,
			queue,
			render_pipeline,
			vertex_buffer,
			index_buffer,
			camera_buffer,
			camera_bind_group,
			objects_bind_group,
		})
	}

	pub fn update_camera(&mut self, width: u32, height: u32) {
		self.queue.write_buffer(
			&self.camera_buffer,
			0,
			bytemuck::cast_slice(&[CameraUniform { width, height }]),
		)
	}

	/// Renders the next frame into the provided [`wgpu::Texture`]
	pub fn render(&mut self, texture: &wgpu::Texture) {
		let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 1.0,
							g: 0.0,
							b: 1.0,
							a: 1.0,
						}),
						store: wgpu::StoreOp::Store,
					},
				})],
				depth_stencil_attachment: None,
				..Default::default()
			});

			render_pass.set_pipeline(&self.render_pipeline);

			render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
			render_pass.set_bind_group(1, &self.objects_bind_group, &[]);

			render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
			render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

			render_pass.draw_indexed(0..(QUAD_INDICES.len() as u32), 0, 0..1)
		}

		self.queue.submit(std::iter::once(encoder.finish()));
	}
}
