use std::sync::Arc;

use bevy_ecs::component::Component;
use bevy_ecs::event::{Event, EventReader, Events};
use bevy_ecs::schedule::{IntoSystemConfigs, ScheduleLabel, Schedules};
use bevy_ecs::system::{ResMut, Resource};
use bevy_ecs::world::World;
use pollster::FutureExt;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

mod renderer;
use renderer::Renderer;

struct App {
	world: World,
}

impl App {
	pub fn new() -> Self {
		let mut world = World::default();

		world.init_resource::<Schedules>();
		world.init_resource::<Events<WinitEvent>>();
		world.init_resource::<RenderTargets>();

		Self { world }
	}

	pub fn run(&mut self) {
		// TODO When should this run?
		self.world.run_schedule(Startup);

		let event_loop = EventLoop::new().unwrap();
		event_loop.set_control_flow(ControlFlow::Poll);
		event_loop.run_app(self).unwrap();
	}

	pub fn add_systems<M>(
		&mut self,
		schedule: impl ScheduleLabel,
		systems: impl IntoSystemConfigs<M>,
	) -> &mut App {
		let mut schedules = self.world.resource_mut::<Schedules>();
		schedules.add_systems(schedule, systems);
		self
	}
}

impl ApplicationHandler for App {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let window = event_loop
			.create_window(Window::default_attributes())
			.unwrap();
		self.world
			.get_resource_mut::<RenderTargets>()
			.unwrap()
			.add(window);
	}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		use WinitEvent::*;
		let mut targets = self.world.get_resource_mut::<RenderTargets>().unwrap();
		match event {
			WindowEvent::RedrawRequested => {
				self.world.run_schedule(Update);
			}
			WindowEvent::CloseRequested => {
				targets.remove(window_id);
				if targets.len() == 0 {
					event_loop.exit();
				}
			}
			WindowEvent::Resized(size) => {
				self.world.send_event(Resized(window_id, size)).unwrap();
			}
			_ => (),
		};
	}
}

#[derive(Resource, Default)]
struct RenderTargets {
	targets: Vec<RenderTarget>,
}

impl RenderTargets {
	pub fn add(&mut self, window: Window) {
		let window = Arc::new(window);
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

		self.targets.push(RenderTarget {
			window,
			surface,
			config,
			renderer,
		});
	}

	pub fn get(&self, window_id: WindowId) -> Option<&RenderTarget> {
		self.targets
			.iter()
			.find(|target| target.window.id() == window_id)
	}

	pub fn get_mut(&mut self, window_id: WindowId) -> Option<&mut RenderTarget> {
		self.targets
			.iter_mut()
			.find(|target| target.window.id() == window_id)
	}

	pub fn remove(&mut self, window_id: WindowId) {
		self.targets
			.retain(|target| target.window.id() != window_id);
	}

	fn iter_mut(&mut self) -> impl Iterator<Item = &mut RenderTarget> {
		self.targets.iter_mut()
	}

	pub fn len(&self) -> usize {
		self.targets.len()
	}
}

#[derive(Component)]
struct RenderTarget {
	window: Arc<Window>,
	surface: wgpu::Surface<'static>,
	config: wgpu::SurfaceConfiguration,
	renderer: Renderer,
}

impl RenderTarget {
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
}

#[derive(Event, Debug)]
#[non_exhaustive]
enum WinitEvent {
	Resized(WindowId, PhysicalSize<u32>),
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct Startup;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct Update;

fn main() -> anyhow::Result<()> {
	App::new()
		.add_systems(Startup, hello_world)
		.add_systems(Update, render)
		.run();

	Ok(())
}

fn render(mut events: EventReader<WinitEvent>, mut targets: ResMut<RenderTargets>) {
	for event in events.read() {
		match event {
			WinitEvent::Resized(window_id, physical_size) => {
				targets.get_mut(*window_id).unwrap().resize(*physical_size);
			}
		}
		println!("{event:?}");
	}

	for target in targets.iter_mut() {
		let surface_texture = match target.surface.get_current_texture() {
			/* event_loop.exit() */
			Err(wgpu::SurfaceError::OutOfMemory) => todo!(),
			// Reconfigure the surface if lost
			Err(wgpu::SurfaceError::Lost) => {
				target.resize(target.window.inner_size());
				continue;
			}
			// Outdated, Timeout errors should be resolved by the next frame
			Err(err) => {
				eprintln!("{err}");
				continue;
			}
			Ok(surface_texture) => surface_texture,
		};

		target.renderer.render(&surface_texture.texture);
		surface_texture.present();
	}
}

fn hello_world() {
	println!("Hello world!");
}
