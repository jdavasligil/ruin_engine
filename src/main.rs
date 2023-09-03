#![allow(dead_code)]
#![allow(unused_variables)]

mod colors;

use crate::colors::Colors::*;
use crate::colors::Colors;

use std::num::NonZeroU32;
use glam::Vec3;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const WIDTH: i16 = 800;
const HEIGHT: i16 = 800;

type VertexID = usize;
type Face = (VertexID, VertexID, VertexID);

// TODO: Implement hyperbolic & linear Gouraud Shading
#[derive(Default)]
struct Vertex {
    position: Vec3,
    normal: Vec3,
    color: Color,
    lux: u8, // Luminous Flux (0-255)
    adjacents: Vec<VertexID>,
}

impl Vertex {
    fn new(position: Vec3) -> Vertex {
        Vertex {
            position,
            normal: Vec3::ZERO,
            color: Colors::BLUE,
            lux: 255,
            adjacents: Vec::<VertexID>::new(),
        }
    }
}

#[derive(Default)]
struct MeshData {
    vertices: Vec<Vertex>,
    faces: Vec<Face>,
}

impl MeshData {
    fn build_cube(origin: Vec3, scale: f32, color: Color) {
    }
}

fn redraw(buffer: &mut [u32], width: usize, height: usize, flag: bool) {
    for y in 0..height {
        for x in 0..width {
            let value = if flag && x >= 100 && x < width - 100 && y >= 100 && y < height - 100 {
                0x00ffffff
            } else {
                let red = (x & 0xff) ^ (y & 0xff);
                let green = (x & 0x7f) ^ (y & 0x7f);
                let blue = (x & 0x3f) ^ (y & 0x3f);
                (blue | (green << 8) | (red << 16)) as u32
            };
            buffer[y * width + x] = value;
        }
    }
}

fn main() {
    let mut vertex_list = Vec::<Vertex>::new();
    let mut face_list = Vec::<Face>::new();

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Press space to show/hide a rectangle")
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_child(&window.canvas())
            .unwrap();
    }

    let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

    let mut flag = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                // Grab the window's client area dimensions
                let (width, height) = {
                    let size = window.inner_size();
                    (size.width, size.height)
                };

                // Resize surface if needed
                surface
                    .resize(
                        NonZeroU32::new(width).unwrap(),
                        NonZeroU32::new(height).unwrap(),
                    )
                    .unwrap();

                // Draw something in the window
                let mut buffer = surface.buffer_mut().unwrap();
                redraw(&mut buffer, width as usize, height as usize, flag);
                buffer.present().unwrap();
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                *control_flow = ControlFlow::Exit;
            }

            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Space),
                                ..
                            },
                        ..
                    },
                window_id,
            } if window_id == window.id() => {
                // Flip the rectangle flag and request a redraw to show the changed image
                flag = !flag;
                window.request_redraw();
            }

            _ => {}
        }
    });
}
