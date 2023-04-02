use std::borrow::Cow;

use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::{Window, WindowId},
};

struct App {
	window: Window,
	instance: wgpu::Instance,
	surface: wgpu::Surface,
	adapter: wgpu::Adapter,
	device: wgpu::Device,
	queue: wgpu::Queue,
	shader: wgpu::ShaderModule,
	pipeline_layout: wgpu::PipelineLayout,
	render_pipeline: wgpu::RenderPipeline,
	config: wgpu::SurfaceConfiguration,
}

impl App {
	async fn new(event_loop: &EventLoop<()>) -> anyhow::Result<Self> {
		let window = Window::new(&event_loop)?;
		let size = window.inner_size();
		let instance = wgpu::Instance::default();
		let surface = unsafe { instance.create_surface(&window) }?;

		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				force_fallback_adapter: false,
				// Request an adapter which can render to our surface
				compatible_surface: Some(&surface),
			})
			.await
			.expect("Failed to find an appropriate adapter");

		let swapchain_capabilities = surface.get_capabilities(&adapter);
		let swapchain_format = swapchain_capabilities.formats[0];

		// Create the logical device and command queue
		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: None,
					features: wgpu::Features::empty(),
					// Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
					limits: wgpu::Limits::downlevel_webgl2_defaults()
						.using_resolution(adapter.limits()),
				},
				None,
			)
			.await
			.expect("Failed to create device");

		// Load the shaders from disk
		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: None,
			source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
		});

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: &[],
			push_constant_ranges: &[],
		});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[],
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

		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: swapchain_format,
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Fifo,
			alpha_mode: swapchain_capabilities.alpha_modes[0],
			view_formats: vec![],
		};

		surface.configure(&device, &config);

		Ok(Self {
			window,
			instance,
			surface,
			adapter,
			device,
			queue,
			shader,
			pipeline_layout,
			render_pipeline,
			config,
		})
	}
}

impl App {
	async fn run(mut self, event_loop: EventLoop<()>) -> anyhow::Result<()> {
		event_loop.run(move |event, _, control_flow| {
			// Have the closure take ownership of the resources.
			// `event_loop.run` never returns, therefore we must do this to ensure
			// the resources are properly cleaned up.
			let _ = (
				&self.instance,
				&self.adapter,
				&self.shader,
				&self.pipeline_layout,
			);

			*control_flow = ControlFlow::Wait;
			match event {
				Event::WindowEvent { event, .. } => self.handle_window_event(event, control_flow),
				Event::RedrawRequested(window_id) => self.handle_redraw(window_id),
				_ => {}
			}
		});
	}

	fn handle_window_event(&mut self, event: WindowEvent, control_flow: &mut ControlFlow) {
		match event {
			WindowEvent::Resized(size) => {
				// Reconfigure the surface with the new size
				self.config.width = size.width;
				self.config.height = size.height;
				self.surface.configure(&self.device, &self.config);
				// On macos the window needs to be redrawn manually after resizing
				self.window.request_redraw();
			}
			WindowEvent::CloseRequested => control_flow.set_exit(),
			_ => (),
		}
	}

	fn handle_redraw(&mut self, _window_id: WindowId) {
		let frame = self
			.surface
			.get_current_texture()
			.expect("Failed to acquire next swap chain texture");
		let view = frame
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: None,
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
						store: true,
					},
				})],
				depth_stencil_attachment: None,
			});
			render_pass.set_pipeline(&self.render_pipeline);
			render_pass.draw(0..3, 0..1);
		}

		self.queue.submit(Some(encoder.finish()));
		frame.present();
	}
}

#[pollster::main]
async fn main() -> anyhow::Result<()> {
	env_logger::init();

	let event_loop = EventLoop::new();
	App::new(&event_loop).await?.run(event_loop).await?;

	Ok(())
}
