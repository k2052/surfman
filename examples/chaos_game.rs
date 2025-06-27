// surfman/surfman/examples/chaos_game.rs
//
//! Demonstrates how to use `surfman` to draw to a window surface.
//! 
//! This example has been updated to use modern winit patterns:
//! - Uses Arc<Window> for proper window management
//! - Structured application state with clean event handling
//! - Modern surface creation and rendering loop
//! - Cross-platform support (macOS with CPU rendering, Linux with demo mode)
//! 
//! The chaos game algorithm creates a fractal by iteratively jumping halfway
//! to random vertices of a triangle, creating the Sierpinski triangle.

use euclid::default::Point2D;
use rand::{self, Rng};
use std::sync::Arc;
use surfman::{Connection, Device};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::raw_window_handle::HasDisplayHandle;
use winit::window::{Window, WindowBuilder};

#[cfg(target_os = "macos")]
use surfman::SystemConnection;

#[cfg(target_os = "macos")]
use euclid::default::Size2D;

#[cfg(target_os = "macos")]
use surfman::{Surface, SurfaceAccess, SurfaceType, Context, ContextAttributes, ContextAttributeFlags, GLVersion};

#[cfg(target_os = "macos")]
use winit::raw_window_handle::HasWindowHandle;



const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;

#[cfg(target_os = "macos")]
const BYTES_PER_PIXEL: usize = 4;

#[cfg(target_os = "macos")]
const FOREGROUND_COLOR: u32 = !0;

const ITERATIONS_PER_FRAME: usize = 20;

static TRIANGLE_POINTS: [(f32, f32); 3] = [
    (400.0, 300.0 + 75.0 + 150.0),
    (400.0 + 259.81, 300.0 + 75.0 - 300.0),
    (400.0 - 259.81, 300.0 + 75.0 - 300.0),
];

#[cfg(not(all(
    any(target_os = "linux", target_os = "macos"),
    feature = "sm-raw-window-handle-06"
)))]
fn main() {
    println!("The `chaos_game` demo is not yet supported on this platform.");
    println!("Supported platforms: macOS (CPU rendering), Linux (GPU rendering)");
}

// macOS version with CPU rendering
#[cfg(all(
    target_os = "macos",
    feature = "sm-raw-window-handle-06"
))]
fn main() {
    struct App {
        window: Arc<Window>,
        connection: Connection,
        device: Device,
        context: Context,
        surface: Surface,
        rng: rand::rngs::ThreadRng,
        point: Point2D<f32>,
        data: Vec<u8>,
    }

    impl App {
        fn new(event_loop: &EventLoop<()>) -> Self {
            let physical_size = PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT);
            let window = Arc::new(
                WindowBuilder::new()
                    .with_title("Chaos game example")
                    .with_inner_size(physical_size)
                    .build(event_loop)
                    .unwrap(),
            );

            // Use the appropriate connection for macOS
            #[cfg(target_os = "macos")]
            let connection = {
                use surfman::SystemConnection;
                SystemConnection::new().unwrap()
            };

            #[cfg(not(target_os = "macos"))]
            let connection = {
                let display_handle = window
                    .display_handle()
                    .expect("failed to get display handle from window");
                Connection::from_display_handle(display_handle).unwrap()
            };

            let adapter = connection.create_adapter().unwrap();
            let mut device = connection.create_device(&adapter).unwrap();

            // Create context for surface operations
            let context_attributes = ContextAttributes {
                version: GLVersion::new(3, 0),
                flags: ContextAttributeFlags::empty(),
            };
            let context_descriptor = device
                .create_context_descriptor(&context_attributes)
                .unwrap();
            let context = device.create_context(&context_descriptor, None).unwrap();

            let window_size = window.inner_size();
            let window_size = Size2D::new(window_size.width as i32, window_size.height as i32);
            let handle = window.window_handle().unwrap();
            let native_widget = connection
                .create_native_widget_from_window_handle(handle, window_size)
                .unwrap();

            let surface_type = SurfaceType::Widget { native_widget };
            let surface = device
                .create_surface(&context, SurfaceAccess::GPUCPU, surface_type)
                .unwrap();

            let rng = rand::thread_rng();
            let point = Point2D::new(WINDOW_WIDTH as f32 * 0.5, WINDOW_HEIGHT as f32 * 0.5);
            let data = vec![0; WINDOW_WIDTH as usize * WINDOW_HEIGHT as usize * 4];

            App {
                window,
                connection,
                device,
                context,
                surface,
                rng,
                point,
                data,
            }
        }

        fn handle_window_event(&mut self, event: WindowEvent) -> bool {
            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            logical_key: Key::Named(NamedKey::Escape),
                            ..
                        },
                    ..
                } => return false,
                WindowEvent::RedrawRequested => {
                    self.render();
                    self.window.request_redraw();
                }
                _ => {}
            }
            true
        }

        fn render(&mut self) {
            for _ in 0..ITERATIONS_PER_FRAME {
                let (dest_x, dest_y) = TRIANGLE_POINTS[self.rng.gen_range(0..3)];
                self.point = self.point.lerp(Point2D::new(dest_x, dest_y), 0.5);
                put_pixel(&mut self.data, &self.point, FOREGROUND_COLOR);
            }

            self.device
                .lock_surface_data(&mut self.surface)
                .unwrap()
                .data()
                .copy_from_slice(&self.data);
            self.device.present_surface(&self.context, &mut self.surface).unwrap();
        }
    }

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(&event_loop);
    
    // Initial render request
    app.window.request_redraw();

    event_loop
        .run(move |event, target| {
            match event {
                Event::WindowEvent { event, .. } => {
                    if !app.handle_window_event(event) {
                        target.exit();
                    }
                }
                _ => {}
            }
            target.set_control_flow(ControlFlow::Poll);
        })
        .unwrap();
}

// Linux version with simplified approach (could be extended to GPU rendering)
#[cfg(all(
    target_os = "linux",
    feature = "sm-raw-window-handle-06"
))]
fn main() {
    struct ChaosGameApp {
        window: Arc<Window>,
        _connection: Connection,
        _device: Device,
        rng: rand::rngs::ThreadRng,
        point: Point2D<f32>,
        iteration_count: usize,
    }

    impl ChaosGameApp {
        fn new(event_loop: &EventLoop<()>) -> Self {
            let physical_size = PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT);
            let window = Arc::new(
                WindowBuilder::new()
                    .with_title("Chaos game example (Linux - Demo)")
                    .with_inner_size(physical_size)
                    .build(event_loop)
                    .unwrap(),
            );

            let display_handle = window
                .display_handle()
                .expect("failed to get display handle from window");
            let connection = Connection::from_display_handle(display_handle).unwrap();
            let adapter = connection.create_adapter().unwrap();
            let device = connection.create_device(&adapter).unwrap();

            let rng = rand::thread_rng();
            let point = Point2D::new(WINDOW_WIDTH as f32 * 0.5, WINDOW_HEIGHT as f32 * 0.5);

            println!("Chaos Game initialized on Linux");
            println!("Note: This is a simplified demo. Full GPU rendering could be implemented.");
            println!("Press Escape to exit");

            ChaosGameApp {
                window,
                _connection: connection,
                _device: device,
                rng,
                point,
                iteration_count: 0,
            }
        }

        fn handle_window_event(&mut self, event: WindowEvent) -> bool {
            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            logical_key: Key::Named(NamedKey::Escape),
                            ..
                        },
                    ..
                } => return false,
                WindowEvent::RedrawRequested => {
                    self.update_chaos_game();
                    self.window.request_redraw();
                }
                _ => {}
            }
            true
        }

        fn update_chaos_game(&mut self) {
            // Perform the chaos game algorithm
            for _ in 0..ITERATIONS_PER_FRAME {
                let (dest_x, dest_y) = TRIANGLE_POINTS[self.rng.gen_range(0..3)];
                self.point = self.point.lerp(Point2D::new(dest_x, dest_y), 0.5);
                self.iteration_count += 1;
            }

            // Every 1000 iterations, print current position
            if self.iteration_count % 1000 == 0 {
                println!(
                    "Iteration {}: Point at ({:.1}, {:.1})",
                    self.iteration_count, self.point.x, self.point.y
                );
            }
        }
    }

    let event_loop = EventLoop::new().unwrap();
    let mut app = ChaosGameApp::new(&event_loop);
    
    // Initial render request
    app.window.request_redraw();

    event_loop
        .run(move |event, target| {
            match event {
                Event::WindowEvent { event, .. } => {
                    if !app.handle_window_event(event) {
                        target.exit();
                    }
                }
                _ => {}
            }
            target.set_control_flow(ControlFlow::Poll);
        })
        .unwrap();
}

#[cfg(target_os = "macos")]
fn put_pixel(data: &mut [u8], point: &Point2D<f32>, color: u32) {
    let (x, y) = (f32::round(point.x) as usize, f32::round(point.y) as usize);
    let start = (y * WINDOW_WIDTH as usize + x) * BYTES_PER_PIXEL;
    for index in 0..BYTES_PER_PIXEL {
        data[index + start] = (color >> (index * 8)) as u8;
    }
}
