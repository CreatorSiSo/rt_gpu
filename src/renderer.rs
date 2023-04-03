use glam::Vec3;
use std::borrow::Cow;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Sphere {
	pub radius: f32,
	pub position: Vec3,
	pub color: Vec3,
	_padding: [u32; 1],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	pub position: Vec3,
}

impl Vertex {
	fn descriptor<'a>() -> wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[wgpu::VertexAttribute {
				offset: 0,
				shader_location: 0,
				format: wgpu::VertexFormat::Float32x3,
			}],
		}
	}
}

#[rustfmt::skip]
const QUAD_VERTICES: &[Vertex] = &[
	Vertex { position: Vec3::new(-1.0, -1.0, 0.0) },
	Vertex { position: Vec3::new( 1.0,  1.0, 0.0) },
	Vertex { position: Vec3::new( 1.0, -1.0, 0.0) },
	Vertex { position: Vec3::new(-1.0,  1.0, 0.0) },
];

#[rustfmt::skip]
const QUAD_INDICES: &[u16] = &[
	0, 1, 2,
	0, 3, 1
];

pub struct Renderer {
	pub device: wgpu::Device,
	queue: wgpu::Queue,
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	render_pipeline: wgpu::RenderPipeline,
	objects_bind_group: wgpu::BindGroup,
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
					features: wgpu::Features::BUFFER_BINDING_ARRAY
						| wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY,
					// Make sure we use the texture resolution liits from the adapter, so we can support images the size of the swapchain.
					limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
				},
				None,
			)
			.await?;

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

		let object_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Object Buffer"),
			contents: bytemuck::cast_slice(&[
				Sphere {
					radius: 1.0,
					position: Vec3::ZERO,
					color: Vec3::new(0.9, 0.4, 1.0),
					_padding: [0],
				},
				Sphere {
					radius: 1.0,
					position: Vec3::ZERO,
					color: Vec3::new(0.6, 1.0, 0.5),
					_padding: [0],
				},
				Sphere {
					radius: 1.0,
					position: Vec3::ZERO,
					color: Vec3::new(0.4, 0.8, 0.8),
					_padding: [0],
				},
			]),
			usage: wgpu::BufferUsages::STORAGE,
		});

		let objects_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("Object Bind Group Layout"),
				entries: &[wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Storage { read_only: true },
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				}],
			});

		let objects_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Objects Bind Group"),
			layout: &objects_bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: object_buffer.as_entire_binding(),
			}],
		});

		// Load the shaders from disk
		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: None,
			source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
		});

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[&objects_bind_group_layout],
			push_constant_ranges: &[],
		});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[Vertex::descriptor()],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[Some(swapchain_format.into())],
			}),
			primitive: wgpu::PrimitiveState::default(),
			depth_stencil: None,
			multisample: wgpu::MultisampleState::default(),
			multiview: None,
		});

		Ok(Self {
			device,
			queue,
			vertex_buffer,
			index_buffer,
			render_pipeline,
			objects_bind_group,
		})
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
						store: true,
					},
				})],
				depth_stencil_attachment: None,
			});

			render_pass.set_pipeline(&self.render_pipeline);
			render_pass.set_bind_group(0, &self.objects_bind_group, &[]);
			render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
			render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
			render_pass.draw_indexed(0..(QUAD_INDICES.len() as u32), 0, 0..1)
		}

		self.queue.submit(std::iter::once(encoder.finish()));
	}
}
