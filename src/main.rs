use std::sync::Arc;

use pollster::FutureExt;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

mod renderer;
use renderer::Renderer;

enum App {
	Active(State),
	Inactive,
}

struct State {
	window: Arc<Window>,
	surface: wgpu::Surface<'static>,
	config: wgpu::SurfaceConfiguration,
	renderer: Renderer,
}

impl ApplicationHandler for App {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let window = Arc::new(
			event_loop
				.create_window(Window::default_attributes())
				.unwrap(),
		);

		let instance = wgpu::Instance::default();
		let surface = instance.create_surface(window.clone()).unwrap();

		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				force_fallback_adapter: false,
				// Request an adapter which can render to our surface
				compatible_surface: Some(&surface),
			})
			.block_on()
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
			desired_maximum_frame_latency: 2,
		};

		let mut renderer = Renderer::new(adapter, swapchain_format).block_on().unwrap();
		renderer.update_camera(size.width, size.height);
		surface.configure(&renderer.device, &config);

		*self = Self::Active(State {
			window,
			surface,
			config,
			renderer,
		});
	}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		let state = match self {
			App::Active(state) => state,
			App::Inactive => panic!(),
		};

		match event {
			WindowEvent::CloseRequested => event_loop.exit(),
			WindowEvent::RedrawRequested => {
				if state.window.id() != window_id {
					return;
				}
				let Err(err) = state.redraw() else {
					return;
				};
				match err {
					wgpu::SurfaceError::OutOfMemory => event_loop.exit(),
					// Reconfigure the surface if lost
					wgpu::SurfaceError::Lost => state.resize(state.window.inner_size()),
					// Outdated, Timeout errors should be resolved by the next frame
					err => eprintln!("{err}"),
				};
			}
			WindowEvent::Resized(size) => state.resize(size),
			_ => (),
		}
	}
}

impl State {
	fn resize(&mut self, PhysicalSize { width, height }: PhysicalSize<u32>) {
		// Reconfigure the surface with the new size
		self.config.width = width;
		self.config.height = height;
		self.surface.configure(&self.renderer.device, &self.config);
		// Update the camera data sent to the gpu
		self.renderer.update_camera(width, height);
		// On macos the window needs to be redrawn manually after resizing
		self.window.request_redraw();
	}

	fn redraw(&mut self) -> Result<(), wgpu::SurfaceError> {
		let surface_texture = self.surface.get_current_texture()?;
		self.renderer.render(&surface_texture.texture);
		surface_texture.present();
		Ok(())
	}
}

fn main() -> anyhow::Result<()> {
	let event_loop = EventLoop::new()?;
	event_loop.set_control_flow(ControlFlow::Poll);

	let mut app = App::Inactive;
	event_loop.run_app(&mut app)?;

	Ok(())
}
