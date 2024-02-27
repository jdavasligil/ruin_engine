#![allow(dead_code)]
#![allow(unused_variables)]

mod colors;

use glam::{Mat3, Vec3, Vec4};
use std::cmp::max;
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowBuilder;

const WIDTH: i16 = 800;
const HEIGHT: i16 = 800;

type VertexID = usize;
type Face = (VertexID, VertexID, VertexID);

// NOTES:
// Raster (Pixel) Space: The actual pixel dimensional space of the device.
// Normalized Device Coordinate Space (NDC): Points are in the range [-1, 1].
//       This is a 2D normalized screen-space for drawing.

// TODO:
// [] Complete raster pipeline to render model in screenspace.
// [] Implement hyperbolic & linear Gouraud Shading

/// A simple SIMD buffer containing vertex and face (vertex id triplet) data.
struct RasterBuffer {
    // TODO: Vertex Lighting
    vert_buf: [Vec4; 128],
    // TODO: Face Normals
    face_buf: [Face; 128],
    vert_count: usize,
    face_count: usize,
}

impl RasterBuffer {
    fn new() -> RasterBuffer {
        RasterBuffer {
            vert_buf: [Vec4::NAN; 128],
            face_buf: [(usize::MAX, usize::MAX, usize::MAX); 128],
            vert_count: 0,
            face_count: 0,
        }
    }

    fn flush(&mut self) {
        self.vert_buf = [Vec4::NAN; 128];
        self.face_buf = [(usize::MAX, usize::MAX, usize::MAX); 128];
        self.vert_count = 0;
        self.face_count = 0;
    }

    fn try_push_vert(&mut self, vert: (f32, f32, f32)) -> Result<(), &'static str> {
        if self.vert_count >= 128 {
            return Err("Buffer is full.");
        }

        self.vert_buf[self.vert_count] = Vec4::new(vert.0, vert.1, vert.2, 1.0);
        self.vert_count += 1;

        Ok(())
    }

    fn try_find_vert(&self, v: (f32, f32, f32)) -> Result<usize, &'static str> {
        let mut idx: usize = 0;

        while idx < self.vert_count {
            if self.vert_buf[idx].x == v.0
                && self.vert_buf[idx].y == v.1
                && self.vert_buf[idx].z == v.2
            {
                return Ok(idx);
            }

            idx += 1;
        }

        Err("Vertex not found.")
    }

    fn try_push_tri(
        &mut self,
        v1: (f32, f32, f32),
        v2: (f32, f32, f32),
        v3: (f32, f32, f32),
    ) -> Result<(), &'static str> {
        if self.face_count >= 128 {
            return Err("Buffer is full.");
        }

        let mut face: Face = (0, 0, 0);

        face.0 = match self.try_find_vert(v1) {
            Ok(idx) => idx,
            Err(str) => return Err(str),
        };
        face.1 = match self.try_find_vert(v2) {
            Ok(idx) => idx,
            Err(str) => return Err(str),
        };
        face.2 = match self.try_find_vert(v3) {
            Ok(idx) => idx,
            Err(str) => return Err(str),
        };

        self.face_buf[self.face_count] = face;
        self.face_count += 1;

        Ok(())
    }

    fn try_push_face(&mut self, face: Face) -> Result<(), &'static str> {
        if self.face_count >= 128 {
            return Err("Buffer is full.");
        }

        self.face_buf[self.face_count] = face;
        self.face_count += 1;

        Ok(())
    }

    fn try_face_to_mat(&self, face_idx: usize) -> Result<Mat3, &'static str> {
        if face_idx >= self.face_count {
            return Err("Index out of range.");
        }

        Ok(Mat3::from_cols(
            self.vert_buf[self.face_buf[face_idx].0].truncate(),
            self.vert_buf[self.face_buf[face_idx].1].truncate(),
            self.vert_buf[self.face_buf[face_idx].2].truncate(),
        ))
    }
}

fn redraw(
    buffer: &mut [u32],
    raster_buffer: &mut RasterBuffer,
    width: usize,
    height: usize
) {
    // Initialize vertex matrix
    let tri_mat = raster_buffer.try_face_to_mat(0).unwrap();

    // Store the inverse
    let tri_mat_inv = tri_mat.inverse();

    // Calculate edge functions (to test what side of each edge the test point exists)
    let edge_test_0 = tri_mat_inv * Vec3::new(1.0, 0.0, 0.0);
    let edge_test_1 = tri_mat_inv * Vec3::new(0.0, 1.0, 0.0);
    let edge_test_2 = tri_mat_inv * Vec3::new(0.0, 0.0, 1.0);

    // Begin rasterizing by looping over pixels
    for y in 0..height {
        for x in 0..width {
            // Sample location at the center of each pixel
            let sample: Vec3 = Vec3::new((x as f32) + 0.5, (y as f32) + 0.5, 1.0);

            let alpha = sample.dot(edge_test_0);
            let beta = sample.dot(edge_test_1);
            let gamma = sample.dot(edge_test_2);

            // let red = (x & 0xff) ^ (y & 0xff);
            // let green = (x & 0x7f) ^ (y & 0x7f);
            // let blue = (x & 0x3f) ^ (y & 0x3f);
            // let value = (blue | (green << 8) | (red << 16)) as u32;
            let value = (0xff << 16) as u32;

            if (alpha >= 0.0) && (beta >= 0.0) && (gamma >= 0.0) {
                buffer[y * width + x] = value;
            } else {
                buffer[y * width + x] = 0;
            }
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let window = Rc::new(
        WindowBuilder::new()
            .with_title("Press space to show/hide a rectangle")
            .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
            .build(&event_loop)
            .unwrap(),
    );

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_child(&window.canvas().unwrap())
            .unwrap();
    }

    let context = softbuffer::Context::new(window.clone()).unwrap();
    let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

    let mut buf = RasterBuffer::new();

    buf.try_push_vert((1.0, 1.0, 1.0)).unwrap(); // 0
    buf.try_push_vert((-1.0, 1.0, 1.0)).unwrap(); // 1
    buf.try_push_vert((1.0, -1.0, 1.0)).unwrap(); // 2
    buf.try_push_vert((1.0, 1.0, -1.0)).unwrap(); // 3
    buf.try_push_vert((-1.0, -1.0, 1.0)).unwrap(); // 4
    buf.try_push_vert((-1.0, 1.0, -1.0)).unwrap(); // 5
    buf.try_push_vert((1.0, -1.0, -1.0)).unwrap(); // 6
    buf.try_push_vert((-1.0, -1.0, -1.0)).unwrap(); // 7

    buf.try_push_face((0, 1, 2)).unwrap();
    buf.try_push_face((1, 4, 2)).unwrap();
    buf.try_push_face((0, 2, 6)).unwrap();
    buf.try_push_face((0, 6, 3)).unwrap();
    buf.try_push_face((3, 6, 7)).unwrap();
    buf.try_push_face((3, 7, 5)).unwrap();
    buf.try_push_face((5, 7, 4)).unwrap();
    buf.try_push_face((5, 4, 1)).unwrap();
    buf.try_push_face((0, 5, 1)).unwrap();
    buf.try_push_face((0, 3, 5)).unwrap();
    buf.try_push_face((2, 7, 6)).unwrap();
    buf.try_push_face((2, 4, 7)).unwrap();

    let mut flag = false;

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::RedrawRequested,
            } if window_id == window.id() => {
                // Grab the window's client area dimensions
                let (width, height) = {
                    let size = window.inner_size();
                    (size.width, size.height)
                };

                // Resize surface if needed
                // HACK: max used to handle WASM error with window.inner_size().
                // Error happens because size.width and size.height are default zero.
                surface
                    .resize(
                        NonZeroU32::new(max(WIDTH as u32, width)).unwrap(),
                        NonZeroU32::new(max(HEIGHT as u32, height)).unwrap(),
                        )
                    .unwrap();

                // Draw something in the window
                let mut buffer = surface.buffer_mut().unwrap();
                redraw(&mut buffer, &mut buf, width as usize, height as usize);
                buffer.present().unwrap();
            }

            Event::WindowEvent {
                window_id,
                event: WindowEvent::CloseRequested,
            } if window_id == window.id() => {
                elwt.exit();
            }

            //Event::WindowEvent {
            //    window_id,
            //    event:
            //        WindowEvent::KeyboardInput {
            //            event:
            //                KeyEvent {
            //                    state: ElementState::Pressed,
            //                    logical_key: Key::Named(NamedKey::Space),
            //                    ..
            //                },
            //                ..
            //        },
            //} if window_id == window.id() => {
            //    // Flip the rectangle flag and request a redraw to show the changed image
            //    flag = !flag;
            //    window.request_redraw();
            //}
            _ => {}
        }
    })
    .unwrap();
}

// DEAD CODE
//
// #[derive(Default)]
// struct Vertex {
//     position: Vec3,
//     normal: Vec3,
//     color: Color,
//     lux: u8, // Luminous Flux (0-255)
//     adjacents: Vec<VertexID>,
// }
//
// impl Vertex {
//     fn new(position: Vec3) -> Vertex {
//         Vertex {
//             position,
//             normal: Vec3::ZERO,
//             color: Colors::BLUE,
//             lux: 255,
//             adjacents: Vec::<VertexID>::new(),
//         }
//     }
// }
//
// // Adjacency list is a dynamic flat array.
// // This structure stores a list of adjacency arrays in a flat structure.
// // The jth adjacent vertex id for the ith vertex is obtained by:
// // idx = i * chunk_size + j
// // Diagram: [[v1, v2, ... , v32][w1, w2, ... , w32] ...]
// struct AdjacencyList {
//     data: Vec<VertexID>,
//     chunk_size: u32,
// }
//
// impl AdjacencyList {
//     fn new() -> AdjacencyList {
//         AdjacencyList {
//             data: Vec::<VertexID>::new(),
//             chunk_size: 8,
//         }
//     }
//     fn insert(idx: usize,) {
//     }
// }
//
// struct VertexData {
//     positions: Vec<Vec3A>,
//     normals: Vec<Vec3A>,
//     colors: Vec<Color>,
//     intensities: Vec<u8>,
//     adjacents: AdjacencyList,
// }
//
// struct FaceData {
//     triangles: Vec<Face>,
// //    normals: Vec<Vec3A>,
// }
//
// #[derive(Default)]
// struct Mesh {
//     vertices: Vec<VertexData>,
//     faces: Vec<Face>,
// }
//
// impl Mesh {
//     fn build_cube(origin: Vec3, scale: f32, color: Color) {
//     }
// }
