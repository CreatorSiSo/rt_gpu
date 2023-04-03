use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

mod renderer;
use renderer::Renderer;

struct App {
	window: Window,
	renderer: Renderer,
}

impl App {
	async fn new(event_loop: &EventLoop<()>) -> anyhow::Result<Self> {
		let window = Window::new(&event_loop)?;
		let renderer = Renderer::new(&window).await?;

		Ok(Self { window, renderer })
	}

	fn run(mut self, event_loop: EventLoop<()>) -> anyhow::Result<()> {
		event_loop.run(move |event, _, control_flow| {
			control_flow.set_wait();

			match event {
				Event::WindowEvent { event, window_id } => {
					self.handle_window_event(window_id, event, control_flow)
				}
				Event::RedrawRequested(window_id) => self.handle_redraw(window_id),
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
			WindowEvent::Resized(size) => {
				self.renderer.resize(size);
				// On macos the window needs to be redrawn manually after resizing
				self.window.request_redraw();
			}
			WindowEvent::CloseRequested => control_flow.set_exit(),
			_ => {}
		}
	}

	fn handle_redraw(&mut self, _window_id: WindowId) {
		self.renderer.draw()
	}
}

#[pollster::main]
async fn main() -> anyhow::Result<()> {
	env_logger::init();

	let event_loop = EventLoop::new();
	App::new(&event_loop).await?.run(event_loop)?;

	Ok(())
}
