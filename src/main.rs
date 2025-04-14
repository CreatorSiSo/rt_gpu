use std::sync::Arc;
use std::time::{Duration, Instant};

use bevy_ecs::component::Component;
use bevy_ecs::event::{Event, EventReader, Events};
use bevy_ecs::schedule::{IntoSystemConfigs, ScheduleLabel, Schedules};
use bevy_ecs::system::{Commands, Query, Res, ResMut, Resource};
use bevy_ecs::world::World;
use glam::{Vec3, Vec4};
use pollster::FutureExt;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

mod renderer;
use renderer::{Renderer, Sphere};

struct App {
	world: World,
	redraw_requested: bool,
	last_update: Instant,
}

impl App {
	pub fn new() -> Self {
		let mut world = World::default();

		world.init_resource::<Schedules>();
		world.init_resource::<Events<WinitEvent>>();
		world.init_resource::<RenderTargets>();
		world.init_resource::<Time>();

		Self {
			world,
			last_update: Instant::now(),
			redraw_requested: false,
		}
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

	fn redraw_requested(&mut self, _event_loop: &ActiveEventLoop) {
		let now = Instant::now();
		let should_update = self.redraw_requested
			|| now.duration_since(self.last_update) >= Duration::from_millis(10);

		if should_update {
			self.redraw_requested = false;
			self.world.run_schedule(PreUpdate);
			self.world.run_schedule(Update);
			self.last_update = now;
			self.world.run_schedule(Extract);
			self.world.run_schedule(Render);
		}
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

	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		self.redraw_requested(event_loop);
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
				self.redraw_requested = true;
				self.redraw_requested(event_loop);
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
			WindowEvent::CursorMoved {
				device_id,
				position,
			} => {
				self.world
					.send_event(CursorMoved(device_id, position))
					.unwrap();
			}
			_ => (),
		};
	}
}

#[derive(Event, Debug)]
#[non_exhaustive]
enum WinitEvent {
	Resized(WindowId, PhysicalSize<u32>),
	CursorMoved(DeviceId, PhysicalPosition<f64>),
}

#[derive(Resource)]
struct Time {
	start: Instant,
	time_ms: f64,
}

impl Default for Time {
	fn default() -> Self {
		Self {
			start: Instant::now(),
			time_ms: 0.0,
		}
	}
}

impl Time {
	fn elapsed_ms(&self) -> f64 {
		self.time_ms
	}

	fn update(&mut self) {
		self.time_ms = Instant::now().duration_since(self.start).as_millis() as f64;
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

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct Startup;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct PreUpdate;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct Update;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct Extract;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct Render;

fn main() -> anyhow::Result<()> {
	App::new()
		.add_systems(Startup, generate_scene)
		.add_systems(PreUpdate, |mut time: ResMut<Time>| {
			time.update();
		})
		.add_systems(Update, animate_spheres)
		.add_systems(Extract, extract_time)
		.add_systems(Extract, extract_spheres)
		.add_systems(Render, render)
		.run();

	Ok(())
}

fn generate_scene(mut commands: Commands) {
	commands.spawn_batch(
		[
			Sphere {
				radius: 1.0,
				position: Vec3::new(-1.5, 0.0, 0.5),
				color: Vec4::new(0.0, 0.0, 0.0, 1.0),
			},
			Sphere {
				radius: 0.5,
				position: Vec3::new(-0.5, 0.0, 0.2),
				color: Vec4::new(0.0, 0.0, 0.0, 1.0),
			},
			Sphere {
				radius: 0.25,
				position: Vec3::new(0.0, 0.00, 0.0),
				color: Vec4::new(0.8, 0.6, 0.2, 1.0),
			},
			Sphere {
				radius: 0.5,
				position: Vec3::new(0.5, 0.0, 0.2),
				color: Vec4::new(0.0, 0.0, 0.0, 1.0),
			},
			Sphere {
				radius: 1.0,
				position: Vec3::new(1.5, 0.0, 0.5),
				color: Vec4::new(0.0, 0.0, 0.0, 1.0),
			},
		]
		.into_iter()
		.map(|sphere| (Animate, sphere)),
	);
	commands.spawn_batch([
		Sphere {
			radius: 3.0,
			position: Vec3::new(-2.5, 4.0, 1.5),
			color: Vec4::new(0.1, 0.005, 0.005, 1.0),
		},
		Sphere {
			radius: 3.0,
			position: Vec3::new(2.5, -4.0, 1.5),
			color: Vec4::new(0.007, 0.007, 0.1, 1.0),
		},
	]);
}

#[derive(Component)]
struct Animate;

fn animate_spheres(mut spheres: Query<(&mut Sphere, &Animate)>, time: Res<Time>) {
	for (mut sphere, _) in &mut spheres {
		let elapsed = time.elapsed_ms() as f32;
		let sin = f32::sin(elapsed / 1000.0 + sphere.position.x);
		sphere.position.y = sin;
	}
}

fn extract_spheres(spheres: Query<&Sphere>, mut targets: ResMut<RenderTargets>) {
	for target in targets.iter_mut() {
		target.renderer.update_spheres(spheres.iter());
	}
}

fn extract_time(time: Res<Time>, mut targets: ResMut<RenderTargets>) {
	for target in targets.iter_mut() {
		target.renderer.update_time(time.elapsed_ms() as f32);
	}
}

fn render(mut events: EventReader<WinitEvent>, mut targets: ResMut<RenderTargets>) {
	for event in events.read() {
		match event {
			WinitEvent::Resized(window_id, physical_size) => {
				targets.get_mut(*window_id).unwrap().resize(*physical_size);
			}
			_ => {}
		}
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
