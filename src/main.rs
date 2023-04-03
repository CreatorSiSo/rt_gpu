use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

mod renderer;
use renderer::{Renderer, Sphere};

struct App {
	window: Window,
	surface: wgpu::Surface,
	config: wgpu::SurfaceConfiguration,
	renderer: Renderer,
	scene: Vec<Sphere>,
}

impl App {
	async fn new(event_loop: &EventLoop<()>) -> anyhow::Result<Self> {
		let window = Window::new(&event_loop)?;
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

		let size = window.inner_size();
		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: swapchain_format,
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Fifo,
			alpha_mode: swapchain_capabilities.alpha_modes[0],
			view_formats: vec![],
		};

		let renderer = Renderer::new(adapter, swapchain_format).await?;
		surface.configure(&renderer.device, &config);

		Ok(Self {
			window,
			surface,
			config,
			renderer,
			scene: vec![],
		})
	}

	fn with_objects(mut self, mut objects: Vec<Sphere>) -> Self {
		self.scene.append(&mut objects);
		self
	}

	fn run(mut self, event_loop: EventLoop<()>) -> anyhow::Result<()> {
		event_loop.run(move |event, _, control_flow| {
			control_flow.set_wait();

			match event {
				Event::WindowEvent { event, window_id } => {
					self.handle_window_event(window_id, event, control_flow)
				}
				Event::RedrawRequested(window_id) => {
					if self.window.id() != window_id {
						return;
					}
					let Err(err) = self.redraw() else  {
						return;
					};
					match err {
						wgpu::SurfaceError::OutOfMemory => control_flow.set_exit(),
						// Reconfigure the surface if lost
						wgpu::SurfaceError::Lost => self.resize(self.window.inner_size()),
						// Outdated, Timeout errors should be resolved by the next frame
						err => eprintln!("{err}"),
					};
				}
				Event::MainEventsCleared => {
					// RedrawRequested will only trigger once, unless we manually request it.
					self.window.request_redraw();
				}
				_ => {}
			}
		});
	}

	fn handle_window_event(
		&mut self,
		_window_id: WindowId,
		event: WindowEvent,
		control_flow: &mut ControlFlow,
	) {
		match event {
			WindowEvent::CloseRequested => control_flow.set_exit(),
			WindowEvent::Resized(size) => self.resize(size),
			_ => {}
		}
	}

	fn redraw(&mut self) -> anyhow::Result<(), wgpu::SurfaceError> {
		let surface_texture = self.surface.get_current_texture()?;
		self.renderer.render(&surface_texture.texture);
		surface_texture.present();
		Ok(())
	}

	fn resize(&mut self, size: PhysicalSize<u32>) {
		// Reconfigure the surface with the new size
		self.config.width = size.width;
		self.config.height = size.height;
		self.surface.configure(&self.renderer.device, &self.config);
		// On macos the window needs to be redrawn manually after resizing
		self.window.request_redraw();
	}
}

#[pollster::main]
async fn main() -> anyhow::Result<()> {
	env_logger::init();

	let event_loop = EventLoop::new();
	App::new(&event_loop)
		.await?
		// .with_objects(vec![Sphere {
		// 	radius: 1.0,
		// 	position: Vec3::ZERO,
		// color: Vec3::new(0.6, 0.4, 0.1),
		// }])
		.run(event_loop)?;

	Ok(())
}
