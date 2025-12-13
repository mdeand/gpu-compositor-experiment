use std::sync::Arc;

struct State {
  window: Arc<winit::window::Window>,
  device: wgpu::Device,
  queue: wgpu::Queue,
  size: winit::dpi::PhysicalSize<u32>,
  surface: wgpu::Surface<'static>,
  surface_format: wgpu::TextureFormat,
}

impl State {
  async fn new(window: Arc<winit::window::Window>) -> Self {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
      // NOTE(mdeand): We're gonna use explicit Vulkan to experiment with wgpu-hal.
      backends: wgpu::Backends::VULKAN,
      ..wgpu::InstanceDescriptor::default()
    });

    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions::default())
      .await
      .expect("Failed to find a suitable GPU adapter");

    let (device, queue) = adapter
      .request_device(&wgpu::DeviceDescriptor::default())
      .await
      .expect("Failed to create device");

    let size = window.inner_size();

    let surface = instance
      .create_surface(window.clone())
      .expect("Failed to create surface");

    let surface_capabilities = surface.get_capabilities(&adapter);

    let surface_format = surface_capabilities.formats[0];

    let state = Self {
      window,
      device,
      queue,
      size,
      surface,
      surface_format,
    };

    state.configure_surface();

    state
  }

  fn configure_surface(&self) {
    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: self.surface_format,
      width: self.size.width,
      height: self.size.height,
      present_mode: wgpu::PresentMode::AutoVsync,
      alpha_mode: wgpu::CompositeAlphaMode::Auto,
      view_formats: vec![self.surface_format.add_srgb_suffix()],
      desired_maximum_frame_latency: 2,
    };

    self.surface.configure(&self.device, &config);
  }

  fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    self.size = new_size;

    self.configure_surface();
  }

  fn render(&mut self) {
    let surface_texture = self
      .surface
      .get_current_texture()
      .expect("Failed to acquire next swapchain texture");

    let texture_view = surface_texture
      .texture
      .create_view(&wgpu::TextureViewDescriptor {
        format: Some(self.surface_format.add_srgb_suffix()),
        ..Default::default()
      });

    let mut encoder = self.device.create_command_encoder(&Default::default());

    unsafe {
      // TODO(mdeand): Utilize the headless vulkan renderer here.
      encoder.as_hal_mut::<wgpu_hal::api::Vulkan, _, _>(|_api| {});
    }

    {
      let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &texture_view,
          resolve_target: None,
          depth_slice: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: wgpu::StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
      });
    }

    self.queue.submit([encoder.finish()]);
    self.window.pre_present_notify();
    surface_texture.present();
  }
}

#[derive(Default)]
struct App {
  state: Option<State>,
}

impl winit::application::ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
    let window = Arc::new(
      event_loop
        .create_window(winit::window::Window::default_attributes())
        .expect("Failed to create window"),
    );

    self.state = Some(pollster::block_on(State::new(window.clone())));

    window.request_redraw();
  }

  fn window_event(
    &mut self,
    event_loop: &winit::event_loop::ActiveEventLoop,
    _window_id: winit::window::WindowId,
    event: winit::event::WindowEvent,
  ) {
    let state = self.state.as_mut().expect("State is not initialized");

    match event {
      winit::event::WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      winit::event::WindowEvent::RedrawRequested => {
        state.render();
        state.window.request_redraw();
      }
      winit::event::WindowEvent::Resized(size) => {
        state.resize(size);
      }
      _ => (),
    }
  }
}

fn main() -> anyhow::Result<()> {
  pretty_env_logger::init();

  let event_loop = winit::event_loop::EventLoop::new().unwrap();

  event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

  let mut app = App::default();

  event_loop.run_app(&mut app).unwrap();

  Ok(())
}
